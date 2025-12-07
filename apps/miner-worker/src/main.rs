// apps/miner-worker/src/main.rs
// =================================================================
// APARATO: MINER WORKER (ELITE EDITION)
// ESTADO: REFACTORIZADO Y OPTIMIZADO PARA PRODUCCIÓN
// =================================================================

use clap::Parser;
use prospector_core_probabilistic::filter_wrapper::RichListFilter;
use prospector_domain_strategy::{BrainwalletIterator, CombinatoricIterator};
use prospector_core_gen::address_legacy::pubkey_to_address;
use prospector_core_gen::wif::private_to_wif;
use prospector_core_math::public_key::SafePublicKey;
use prospector_core_math::private_key::SafePrivateKey;
use prospector_domain_models::{finding::Finding, work::WorkOrder, work::SearchStrategy};
use rayon::prelude::*;
use std::path::PathBuf;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::time::Duration;
use tokio::time::sleep;
use anyhow::{Context, Result, anyhow};
use reqwest::Client;

#[derive(Parser, Debug)]
struct Args {
    #[arg(long, env = "ORCHESTRATOR_URL")]
    orchestrator_url: String,

    #[arg(long, env = "WORKER_AUTH_TOKEN")]
    auth_token: String,

    #[arg(long, default_value = "drone-unit")]
    worker_id: String,
}

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
                .pool_idle_timeout(Duration::from_secs(90))
                .build().unwrap(),
            base_url,
            token,
        }
    }

    async fn hydrate_filter(&self, path: &PathBuf) -> Result<()> {
        if path.exists() {
            println!("✅ [CACHE] Filtro detectado localmente: {:?}", path);
            return Ok(());
        }
        println!("⬇️ [NET] Iniciando descarga de UTXO Filter...");

        let url = format!("{}/resources/utxo_filter.bin", self.base_url);
        let res = self.client.get(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .send().await.context("Error de conexión al descargar filtro")?;

        if !res.status().is_success() {
            return Err(anyhow!("Servidor rechazó descarga: HTTP {}", res.status()));
        }

        let bytes = res.bytes().await.context("Error descargando bytes")?;
        std::fs::write(path, bytes).context("Error I/O escribiendo filtro")?;
        println!("✅ [IO] Filtro hidratado: {:?}", path);
        Ok(())
    }

    async fn get_job(&self) -> Result<WorkOrder> {
        let url = format!("{}/api/v1/job", self.base_url);
        let res = self.client.post(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .send().await.context("Error solicitando trabajo")?;

        if res.status() == 404 { return Err(anyhow!("Idle (Sin trabajo disponible)")); }
        if !res.status().is_success() { return Err(anyhow!("Error API: {}", res.status())); }

        res.json::<WorkOrder>().await.context("Error deserializando WorkOrder")
    }

    async fn report_finding(&self, finding: &Finding) -> Result<()> {
        let url = format!("{}/api/v1/finding", self.base_url);
        self.client.post(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .json(finding)
            .send().await.context("Error reportando hallazgo")?;
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let args = Args::parse();
    println!("🚀 PROSPECTOR WORKER {} ONLINE", args.worker_id);

    let client = Arc::new(WorkerClient::new(args.orchestrator_url.clone(), args.auth_token.clone()));
    let filter_path = PathBuf::from("utxo_filter.bin");

    // 1. Hidratación Resiliente (Retry Loop)
    let mut retry_count = 0;
    loop {
        match client.hydrate_filter(&filter_path).await {
            Ok(_) => break,
            Err(e) => {
                retry_count += 1;
                eprintln!("⚠️ Error hidratación ({}/10): {}. Reintentando en 5s...", retry_count, e);
                if retry_count >= 10 {
                    eprintln!("💀 Fallo crítico: No se pudo obtener el filtro.");
                    std::process::exit(1);
                }
                sleep(Duration::from_secs(5)).await;
            }
        }
    }

    // 2. Carga en Memoria (Blocking Operation)
    println!("🧠 Cargando Bloom Filter en RAM...");
    // Usamos spawn_blocking porque deserializar 200MB es pesado para el loop async
    let filter = tokio::task::spawn_blocking(move || {
        RichListFilter::load_from_file(&filter_path).context("Filtro corrupto")
    }).await??;

    let filter = Arc::new(filter);
    println!("✅ Motor Listo. Objetivos Indexados: {}", filter.count());

    // 3. Loop Principal de Minería
    let running = Arc::new(AtomicBool::new(true));

    // Captura de Ctrl+C para graceful shutdown (opcional)
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    }).unwrap_or_default();

    while running.load(Ordering::Relaxed) {
        match client.get_job().await {
            Ok(job) => {
                println!("🔨 JOB STARTED: {} [{:?}]", job.id, job.strategy);
                let f_ref = filter.clone();
                let c_ref = client.clone();

                // Ejecutamos la estrategia intensiva en un thread pool dedicado
                // Esto permite que el heartbeat siga funcionando si lo implementamos
                tokio::task::spawn_blocking(move || {
                    process_job(job, &f_ref, &c_ref);
                }).await?;

                println!("🏁 JOB FINISHED");
            },
            Err(e) => {
                println!("💤 Esperando instrucciones... ({})", e);
                sleep(Duration::from_secs(10)).await;
            }
        }
    }

    println!("🛑 Worker detenido.");
    Ok(())
}

/// Núcleo de procesamiento agnóstico a la estrategia
fn process_job(job: WorkOrder, filter: &RichListFilter, client: &WorkerClient) {
    // Closure para procesar cada candidato generado
    let check_candidate = |source: String, pk: SafePrivateKey| {
        let pub_key = SafePublicKey::from_private(&pk);
        let addr = pubkey_to_address(&pub_key, false); // Legacy Uncompressed (Satoshi Era)

        if filter.contains(&addr) {
            println!("🚨 MATCH FOUND: {} ({})", addr, source);

            let finding = Finding {
                address: addr,
                private_key_wif: private_to_wif(&pk, false),
                source_entropy: source,
                wallet_type: "p2pkh_legacy".to_string(),
            };

            // Bridge Sync (Rayon) -> Async (Tokio)
            let rt = tokio::runtime::Handle::current();
            rt.block_on(async {
                // Reintentos infinitos para reportar hallazgo (es crítico no perderlo)
                loop {
                    match client.report_finding(&finding).await {
                        Ok(_) => {
                            println!("✅ Hallazgo asegurado en DB");
                            break;
                        },
                        Err(e) => {
                            eprintln!("❌ ERROR CRÍTICO REPORTANDO: {}. Reintentando...", e);
                            tokio::time::sleep(Duration::from_secs(2)).await;
                        }
                    }
                }
            });
        }
    };

    // Despacho de Estrategia con Paralelismo Rayon
    match job.strategy {
        SearchStrategy::Dictionary { dataset_url: _, limit: _ } => {
            // En producción: Descargar dataset_url a /tmp y leerlo.
            // Mock para demostración:
            let dict = vec!["satoshi".to_string(), "bitcoin".to_string(), "123456".to_string(), "password".to_string()];
            let strategy = BrainwalletIterator::new(&dict);
            strategy.par_bridge().for_each(|(phrase, pk)| check_candidate(format!("brain:{}", phrase), pk));
        },
        SearchStrategy::Combinatoric { prefix, suffix, start_index, end_index } => {
            println!("🔢 Scan numérico: {}{}..{}{}", prefix, start_index, prefix, end_index);
            let strategy = CombinatoricIterator::new(start_index, end_index, prefix, suffix);
            strategy.par_bridge().for_each(|(phrase, pk)| check_candidate(format!("comb:{}", phrase), pk));
        },
        SearchStrategy::Random { seed: _ } => {
            println!("⚠️ Estrategia Random no implementada en este worker version.");
        }
    }
}
