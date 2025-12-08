// =================================================================
// APARATO: MINER WORKER (STEALTH EDITION v3.5)
// OBJETIVO: MINERÍA DISTRIBUIDA CON EVASIÓN DE DETECCIÓN (DPI)
// =================================================================

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
use reqwest::{Client, header};

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

// --- CLIENTE HTTP RESILIENTE (CHROME SPOOFING) ---

struct WorkerClient {
    client: Client,
    base_url: String,
    token: String,
}

impl WorkerClient {
    fn new(base_url: String, token: String) -> Self {
        // CONSTRUCCIÓN DE MÁSCARA DIGITAL
        // Imitamos la huella digital exacta de Chrome 120 en Windows 10
        let mut headers = header::HeaderMap::new();
        headers.insert("User-Agent", header::HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"));
        headers.insert("Accept", header::HeaderValue::from_static("application/json, text/plain, */*"));
        headers.insert("Accept-Language", header::HeaderValue::from_static("en-US,en;q=0.9"));
        headers.insert("Connection", header::HeaderValue::from_static("keep-alive"));
        headers.insert("Sec-Ch-Ua", header::HeaderValue::from_static("\"Not_A Brand\";v=\"8\", \"Chromium\";v=\"120\", \"Google Chrome\";v=\"120\""));
        headers.insert("Sec-Ch-Ua-Mobile", header::HeaderValue::from_static("?0"));
        headers.insert("Sec-Ch-Ua-Platform", header::HeaderValue::from_static("\"Windows\""));

        let client = Client::builder()
            .default_headers(headers)
            .timeout(Duration::from_secs(45))
            .pool_idle_timeout(Duration::from_secs(15)) // Rotación de sockets para evitar tracking
            .build()
            .expect("Fallo crítico inicializando stack de red");

        Self {
            client,
            base_url,
            token,
        }
    }

    /// Descarga el filtro de Bloom simulando una descarga de archivo estático normal.
    async fn hydrate_filter(&self, path: &PathBuf) -> Result<()> {
        if path.exists() {
            println!("✅ [CACHE] Filtro local verificado: {:?}", path);
            return Ok(());
        }
        println!("⬇️ [NET] Iniciando descarga de UTXO Filter (Payload Grande)...");

        let url = format!("{}/resources/utxo_filter.bin", self.base_url);

        let res = self.client.get(&url)
            // Auth va en header estándar Bearer
            .header("Authorization", format!("Bearer {}", self.token))
            .send().await?;

        if !res.status().is_success() {
            return Err(anyhow!("Servidor rechazó descarga: HTTP {}", res.status()));
        }

        // Descarga a memoria y volcado a disco
        let content = res.bytes().await?;
        tokio::fs::write(path, content).await?;

        println!("✅ [IO] Filtro hidratado correctamente ({} bytes).", path.metadata()?.len());
        Ok(())
    }

    async fn get_job(&self) -> Result<WorkOrder> {
        let url = format!("{}/api/v1/job", self.base_url);
        let res = self.client.post(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .send().await?;

        if res.status() == 404 { return Err(anyhow!("Idle (Sin trabajo asignado)")); }
        if !res.status().is_success() { return Err(anyhow!("Error API Job: {}", res.status())); }

        res.json::<WorkOrder>().await.context("Error deserializando WorkOrder")
    }

    async fn report_finding(&self, finding: &Finding) -> Result<()> {
        let url = format!("{}/api/v1/finding", self.base_url);
        self.client.post(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .json(finding)
            .send().await?;
        Ok(())
    }
}

// --- PUENTE DE COMUNICACIÓN (RAYON -> TOKIO) ---

struct ChannelReporter {
    sender: mpsc::UnboundedSender<Finding>,
}

impl FindingHandler for ChannelReporter {
    fn on_finding(&self, address: String, pk: SafePrivateKey, source: String) {
        println!("🚨 ¡COLISIÓN CONFIRMADA! Address: {} | Source: {}", address, source);

        let finding = Finding {
            address,
            private_key_wif: private_to_wif(&pk, false), // WIF Legacy (Uncompressed)
            source_entropy: source,
            wallet_type: "p2pkh_legacy".to_string(),
        };

        if let Err(e) = self.sender.send(finding) {
            eprintln!("💀 ERROR CRÍTICO: Fallo al encolar hallazgo. Canal cerrado: {}", e);
        }
    }
}

// --- NÚCLEO DE EJECUCIÓN ---

#[tokio::main]
async fn main() -> Result<()> {
    // Inicialización de logs
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }
    env_logger::init();

    let args = Args::parse();
    println!("🚀 PROSPECTOR WORKER {} ONLINE (Stealth Mode Active)", args.worker_id);

    // 1. Inicializar Cliente
    let client = Arc::new(WorkerClient::new(args.orchestrator_url.clone(), args.auth_token.clone()));
    let filter_path = PathBuf::from("utxo_filter.bin");

    // 2. Hidratación Resiliente (Retry Loop)
    let mut backoff = 1;
    loop {
        match client.hydrate_filter(&filter_path).await {
            Ok(_) => break,
            Err(e) => {
                eprintln!("⚠️ Error hidratación: {}. Reintentando en {}s...", e, backoff);
                sleep(Duration::from_secs(backoff)).await;
                backoff = std::cmp::min(backoff * 2, 60); // Max backoff 60s
            }
        }
    }

    // 3. Carga de Memoria (Bloqueante en hilo dedicado)
    println!("🧠 Cargando Estructura Probabilística en RAM...");
    let filter = Arc::new(tokio::task::spawn_blocking(move || {
        RichListFilter::load_from_file(&filter_path).expect("Filtro corrupto o ilegible")
    }).await?);

    println!("✅ Motor Listo. Objetivos Indexados: {}", filter.count());

    // 4. Preparar Contexto y Canales
    // (Futuro: Aquí descargaríamos diccionarios externos si la estrategia lo requiere)
    let context = Arc::new(ExecutorContext { dictionary_cache: None });

    let (tx, mut rx) = mpsc::unbounded_channel();

    // 5. Spawn Uploader Thread (Consume los hallazgos sin bloquear CPU)
    let client_ref = client.clone();
    tokio::spawn(async move {
        while let Some(finding) = rx.recv().await {
            println!("📤 Subiendo hallazgo a la Bóveda...");
            loop {
                match client_ref.report_finding(&finding).await {
                    Ok(_) => { println!("✅ Hallazgo asegurado en DB."); break; },
                    Err(e) => {
                        eprintln!("❌ Error red subiendo hallazgo: {}. Reintentando...", e);
                        sleep(Duration::from_secs(2)).await;
                    }
                }
            }
        }
    });

    // 6. Bucle Principal de Trabajo
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        println!("\n🛑 Señal de parada recibida.");
        r.store(false, Ordering::SeqCst);
    }).unwrap_or_default();

    while running.load(Ordering::Relaxed) {
        match client.get_job().await {
            Ok(job) => {
                println!("🔨 JOB: {} [{:?}]", job.id, job.strategy);

                let f_ref = filter.clone();
                let ctx_ref = context.clone();
                let reporter = ChannelReporter { sender: tx.clone() };

                // Ejecución CPU-Intensive en Thread Pool de Rayon (vía spawn_blocking)
                let result = tokio::task::spawn_blocking(move || {
                    StrategyExecutor::execute(&job, &f_ref, &ctx_ref, &reporter);
                }).await;

                if let Err(e) = result {
                    eprintln!("❌ Pánico en hilo de minería: {}", e);
                } else {
                    println!("🏁 JOB FINALIZADO");
                }
            },
            Err(e) => {
                // Si no hay trabajo o hay error, esperar un poco antes de volver a preguntar
                println!("💤 Esperando instrucciones... ({})", e);
                sleep(Duration::from_secs(10)).await;
            }
        }
    }

    println!("👋 Worker desconectado limpiamente.");
    Ok(())
}
