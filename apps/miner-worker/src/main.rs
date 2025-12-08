use clap::Parser;
use prospector_core_probabilistic::filter_wrapper::RichListFilter;
use prospector_core_gen::wif::private_to_wif;
use prospector_core_math::private_key::SafePrivateKey;
use prospector_domain_models::{finding::Finding, work::WorkOrder};
use prospector_domain_strategy::{StrategyExecutor, ExecutorContext, FindingHandler};
use std::path::PathBuf;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::sleep;
use anyhow::{Context, Result, anyhow};
use reqwest::Client;

// --- CONFIGURACIÓN & ARGS ---

#[derive(Parser, Debug)]
struct Args {
    #[arg(long, env = "ORCHESTRATOR_URL")]
    orchestrator_url: String,

    #[arg(long, env = "WORKER_AUTH_TOKEN")]
    auth_token: String,

    #[arg(long, default_value = "drone-unit")]
    worker_id: String,
}

// --- CLIENTE HTTP RESILIENTE ---

struct WorkerClient {
    client: Client,
    base_url: String,
    token: String,
}

impl WorkerClient {
    fn new(base_url: String, token: String) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(30))
                .build().unwrap(),
            base_url,
            token,
        }
    }

    async fn hydrate_filter(&self, path: &PathBuf) -> Result<()> {
        if path.exists() {
            println!("✅ [CACHE] Filtro local detectado: {:?}", path);
            return Ok(());
        }
        println!("⬇️ [NET] Descargando UTXO Filter...");
        let url = format!("{}/resources/utxo_filter.bin", self.base_url);
        let res = self.client.get(&url).header("Authorization", format!("Bearer {}", self.token)).send().await?;
        if !res.status().is_success() { return Err(anyhow!("HTTP {}", res.status())); }
        let content = res.bytes().await?;
        tokio::fs::write(path, content).await?;
        println!("✅ [IO] Filtro guardado.");
        Ok(())
    }

    async fn get_job(&self) -> Result<WorkOrder> {
        let url = format!("{}/api/v1/job", self.base_url);
        let res = self.client.post(&url).header("Authorization", format!("Bearer {}", self.token)).send().await?;
        if res.status() == 404 { return Err(anyhow!("Idle")); }
        res.json::<WorkOrder>().await.context("Error parseando Job")
    }

    async fn report_finding(&self, finding: &Finding) -> Result<()> {
        let url = format!("{}/api/v1/finding", self.base_url);
        self.client.post(&url).header("Authorization", format!("Bearer {}", self.token)).json(finding).send().await?;
        Ok(())
    }
}

// --- IMPLEMENTACIÓN DEL BRIDGE (FindingHandler) ---

/// Puente entre el mundo Sincrónico (Rayon) y Asincrónico (Tokio).
/// Envía los hallazgos a través de un canal MPSC.
struct ChannelReporter {
    sender: mpsc::UnboundedSender<Finding>,
}

impl FindingHandler for ChannelReporter {
    fn on_finding(&self, address: String, pk: SafePrivateKey, source: String) {
        println!("🚨 ¡COLISIÓN CONFIRMADA! Address: {}", address);

        let finding = Finding {
            address,
            private_key_wif: private_to_wif(&pk, false), // WIF Legacy
            source_entropy: source,
            wallet_type: "p2pkh_legacy".to_string(),
        };

        // Enviar al canal. Si falla (canal cerrado), logueamos error crítico.
        if let Err(e) = self.sender.send(finding) {
            eprintln!("💀 ERROR CRÍTICO: No se pudo encolar el hallazgo. Canal cerrado: {}", e);
        }
    }
}

// --- MAIN LOOP ---

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let args = Args::parse();
    println!("🚀 PROSPECTOR WORKER {} ONLINE (Architecture v3.0)", args.worker_id);

    let client = Arc::new(WorkerClient::new(args.orchestrator_url.clone(), args.auth_token.clone()));
    let filter_path = PathBuf::from("utxo_filter.bin");

    // 1. Hidratación (Loop de reintento simple)
    while let Err(e) = client.hydrate_filter(&filter_path).await {
        eprintln!("⚠️ Error hidratación: {}. Reintentando en 5s...", e);
        sleep(Duration::from_secs(5)).await;
    }

    // 2. Carga en Memoria (Bloqueante)
    println!("🧠 Cargando Filtro en RAM...");
    let filter = Arc::new(tokio::task::spawn_blocking(move || {
        RichListFilter::load_from_file(&filter_path).expect("Filtro corrupto o ilegible")
    }).await?);
    println!("✅ Motor Listo. Objetivos: {}", filter.count());

    // 3. Preparar Contexto de Ejecución (Caché de Diccionarios)
    // En el futuro, aquí descargaríamos 'rockyou.txt' si fuera necesario
    let context = Arc::new(ExecutorContext {
        dictionary_cache: None, // Por defecto vacío hasta implementar downloader
    });

    // 4. Configurar Canal de Reporte (MPSC)
    let (tx, mut rx) = mpsc::unbounded_channel();

    // TAREA DE FONDO: Uploader de Hallazgos
    // Este hilo vive aparte y solo se dedica a subir lo que encuentre el minero.
    let client_ref = client.clone();
    tokio::spawn(async move {
        while let Some(finding) = rx.recv().await {
            println!("📤 Subiendo hallazgo a la Bóveda...");
            loop {
                match client_ref.report_finding(&finding).await {
                    Ok(_) => { println!("✅ Hallazgo asegurado."); break; },
                    Err(e) => {
                        eprintln!("❌ Error red subiendo hallazgo: {}. Reintentando...", e);
                        sleep(Duration::from_secs(2)).await;
                    }
                }
            }
        }
    });

    // 5. Loop Principal de Trabajo
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || r.store(false, Ordering::SeqCst)).unwrap_or_default();

    while running.load(Ordering::Relaxed) {
        match client.get_job().await {
            Ok(job) => {
                println!("🔨 JOB: {} [{:?}]", job.id, job.strategy);

                let f_ref = filter.clone();
                let ctx_ref = context.clone();
                let reporter = ChannelReporter { sender: tx.clone() };

                // Ejecución CPU-Intensive en Thread separado
                tokio::task::spawn_blocking(move || {
                    StrategyExecutor::execute(&job, &f_ref, &ctx_ref, &reporter);
                }).await?;

                println!("🏁 JOB FINALIZADO");
            },
            Err(_) => {
                // Modo silencioso en espera
                sleep(Duration::from_secs(5)).await;
            }
        }
    }

    Ok(())
}
