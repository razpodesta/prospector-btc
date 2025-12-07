// apps/miner-worker/src/main.rs
// =================================================================
// APARATO: MINER WORKER (STREAMING EDITION)
// ESTADO: PRODUCTION READY
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
use std::io::{BufRead, BufReader};
use std::fs::File;
use tokio::time::sleep;
use anyhow::{Context, Result, anyhow};
use reqwest::Client;
use futures_util::StreamExt; // Necesario para download streaming
use std::io::Write;

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

    // ... (Métodos hydrate_filter y get_job se mantienen igual que en el snapshot original)
    // Solo reescribo lo necesario para brevedad, asume que hydrate_filter, get_job y report_finding están aquí.

    async fn hydrate_filter(&self, path: &PathBuf) -> Result<()> {
        if path.exists() {
            println!("✅ [CACHE] Filtro detectado localmente: {:?}", path);
            return Ok(());
        }
        println!("⬇️ [NET] Iniciando descarga de UTXO Filter...");
        let url = format!("{}/resources/utxo_filter.bin", self.base_url);
        let res = self.client.get(&url).header("Authorization", format!("Bearer {}", self.token)).send().await?;
        if !res.status().is_success() { return Err(anyhow!("HTTP Error {}", res.status())); }
        let bytes = res.bytes().await?;
        std::fs::write(path, bytes)?;
        Ok(())
    }

    async fn get_job(&self) -> Result<WorkOrder> {
        let url = format!("{}/api/v1/job", self.base_url);
        let res = self.client.post(&url).header("Authorization", format!("Bearer {}", self.token)).send().await?;
        if res.status() == 404 { return Err(anyhow!("Idle")); }
        res.json::<WorkOrder>().await.context("Error deserializando WorkOrder")
    }

    async fn report_finding(&self, finding: &Finding) -> Result<()> {
        let url = format!("{}/api/v1/finding", self.base_url);
        self.client.post(&url).header("Authorization", format!("Bearer {}", self.token)).json(finding).send().await?;
        Ok(())
    }

    /// Descarga un dataset (diccionario) a un archivo temporal.
    async fn download_dataset(&self, url: &str, destination: &PathBuf) -> Result<()> {
        if destination.exists() {
             println!("📚 Dataset ya existe en caché: {:?}", destination);
             return Ok(());
        }

        println!("⬇️ Descargando Dataset: {}", url);
        let res = self.client.get(url).send().await?;

        if !res.status().is_success() {
            return Err(anyhow!("Fallo descarga dataset: {}", res.status()));
        }

        let mut file = File::create(destination)?;
        let mut stream = res.bytes_stream();

        while let Some(item) = stream.next().await {
            let chunk = item?;
            file.write_all(&chunk)?;
        }

        println!("✅ Dataset descargado: {:?}", destination);
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let args = Args::parse();
    println!("🚀 PROSPECTOR WORKER {} ONLINE (v2.0 Streaming)", args.worker_id);

    let client = Arc::new(WorkerClient::new(args.orchestrator_url.clone(), args.auth_token.clone()));
    let filter_path = PathBuf::from("utxo_filter.bin");

    // 1. Hidratación
    let mut retry = 0;
    while let Err(e) = client.hydrate_filter(&filter_path).await {
        retry += 1;
        eprintln!("⚠️ Error filtro: {}. Reintento {}/10", e, retry);
        if retry > 10 { std::process::exit(1); }
        sleep(Duration::from_secs(5)).await;
    }

    // 2. Carga Bloom Filter
    println!("🧠 Cargando Bloom Filter...");
    let filter = Arc::new(tokio::task::spawn_blocking(move || {
        RichListFilter::load_from_file(&filter_path).unwrap()
    }).await?);

    println!("✅ Motor Listo. Objetivos: {}", filter.count());

    // 3. Loop Minería
    let running = Arc::new(AtomicBool::new(true));
    while running.load(Ordering::Relaxed) {
        match client.get_job().await {
            Ok(job) => {
                println!("🔨 JOB: {}", job.id);
                let f_ref = filter.clone();
                let c_ref = client.clone();

                // Ejecutar job (bloqueante para CPU, pero en hilo separado)
                tokio::task::spawn_blocking(move || {
                    let rt = tokio::runtime::Handle::current();
                    rt.block_on(async {
                        process_job(job, &f_ref, &c_ref).await.unwrap_or_else(|e| eprintln!("❌ Error Job: {}", e));
                    });
                }).await?;
            },
            Err(_) => sleep(Duration::from_secs(5)).await,
        }
    }
    Ok(())
}

async fn process_job(job: WorkOrder, filter: &RichListFilter, client: &WorkerClient) -> Result<()> {
    // Definimos la closure de chequeo (lógica pura)
    let check = |source: String, pk: SafePrivateKey| {
        let pub_key = SafePublicKey::from_private(&pk);
        let addr = pubkey_to_address(&pub_key, false);
        if filter.contains(&addr) {
            println!("🚨 HALLAZGO: {}", addr);
            let f = Finding {
                address: addr,
                private_key_wif: private_to_wif(&pk, false),
                source_entropy: source,
                wallet_type: "p2pkh".to_string(),
            };
            // Hack para llamar async desde rayon context si fuera necesario,
            // pero aquí estamos bajo control de tokio
             let rt = tokio::runtime::Handle::current();
             rt.block_on(async {
                 let _ = client.report_finding(&f).await;
             });
        }
    };

    match job.strategy {
        SearchStrategy::Dictionary { dataset_url, limit: _ } => {
            // 1. Descargar dataset si no existe
            let filename = dataset_url.split('/').last().unwrap_or("dict.txt");
            let path = PathBuf::from("/tmp").join(filename); // Usar /tmp en Linux (Colab)

            client.download_dataset(&dataset_url, &path).await?;

            // 2. Leer archivo y procesar en paralelo
            // Hacemos streaming leyendo líneas en un vector buffer para alimentar a Rayon
            // No cargamos todo el archivo en RAM. Cargamos chunks.
            let file = File::open(&path)?;
            let reader = BufReader::new(file);

            // Estrategia de chunks para Rayon
            let mut chunk = Vec::with_capacity(10_000);

            for line in reader.lines() {
                if let Ok(l) = line {
                    chunk.push(l);
                    if chunk.len() >= 10_000 {
                        // Procesar chunk en paralelo
                        let batch = std::mem::replace(&mut chunk, Vec::with_capacity(10_000));
                        batch.par_iter().for_each(|phrase| {
                            let pk = prospector_domain_strategy::brainwallet::phrase_to_private_key(phrase);
                            check(format!("brain:{}", phrase), pk);
                        });
                    }
                }
            }
            // Procesar remanente
            if !chunk.is_empty() {
                chunk.par_iter().for_each(|phrase| {
                    let pk = prospector_domain_strategy::brainwallet::phrase_to_private_key(phrase);
                    check(format!("brain:{}", phrase), pk);
                });
            }
        },
        SearchStrategy::Combinatoric { prefix, suffix, start_index, end_index } => {
            let iter = CombinatoricIterator::new(start_index, end_index, prefix, suffix);
            iter.par_bridge().for_each(|(phrase, pk)| check(format!("comb:{}", phrase), pk));
        },
        _ => {}
    }
    Ok(())
}
