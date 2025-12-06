// apps/miner-worker/src/main.rs
// =================================================================
// APARATO: MINER WORKER (SMART AUTONOMOUS UNIT)
// NIVEL: ELITE
// CAPACIDADES: AUTO-HIDRATACIÓN, REPORTE RESILIENTE, LOOP INFINITO
// =================================================================

use clap::Parser;
use prospector_core_probabilistic::filter_wrapper::RichListFilter;
use prospector_domain_strategy::BrainwalletIterator;
use prospector_core_gen::address_legacy::pubkey_to_address;
use prospector_core_gen::wif::private_to_wif;
use prospector_core_math::public_key::SafePublicKey;
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
    /// URL del Orquestador (Ej: https://prospector-api.onrender.com/api/v1)
    #[arg(long, env = "ORCHESTRATOR_URL")]
    orchestrator_url: String,

    /// Token de Autenticación
    #[arg(long, env = "WORKER_AUTH_TOKEN")]
    auth_token: String,

    /// ID del Worker (Generado o asignado)
    #[arg(long, default_value = "anonymous-drone")]
    worker_id: String,
}

/// Cliente HTTP reusado
struct WorkerClient {
    client: Client,
    base_url: String,
    token: String,
}

impl WorkerClient {
    fn new(base_url: String, token: String) -> Self {
        Self {
            client: Client::builder().timeout(Duration::from_secs(10)).build().unwrap(),
            base_url,
            token,
        }
    }

    /// Descarga el Filtro de Bloom si no existe localmente
    async fn hydrate_filter(&self, path: &PathBuf) -> Result<()> {
        if path.exists() {
            println!("✅ Filtro encontrado en caché local: {:?}", path);
            return Ok(());
        }

        println!("⬇️ Descargando UTXO Filter desde el Comandante...");
        let url = format!("{}/resources/utxo_filter.bin", self.base_url);

        let response = self.client.get(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .send()
            .await
            .context("Fallo al conectar para descargar filtro")?;

        if !response.status().is_success() {
            return Err(anyhow!("El servidor rechazó la descarga: {}", response.status()));
        }

        let bytes = response.bytes().await.context("Fallo al bajar bytes")?;
        std::fs::write(path, bytes).context("Fallo al escribir filtro en disco")?;

        println!("✅ Filtro descargado e hidratado exitosamente ({:?})", path);
        Ok(())
    }

    /// Pide trabajo al Orquestador
    async fn get_job(&self) -> Result<WorkOrder> {
        let url = format!("{}/job", self.base_url);
        let res = self.client.post(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .send()
            .await
            .context("Fallo de red al pedir trabajo")?;

        if res.status() == 404 {
            return Err(anyhow!("No hay trabajo disponible (Idle)"));
        }

        res.json::<WorkOrder>().await.context("Error deserializando Job")
    }

    /// Reporta un hallazgo
    async fn report_finding(&self, finding: &Finding) -> Result<()> {
        let url = format!("{}/finding", self.base_url);
        self.client.post(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .json(finding)
            .send()
            .await
            .context("Fallo al enviar reporte")?;
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let args = Args::parse();

    println!("🤖 PROSPECTOR DRONE v2.0 (Elite Edition)");
    println!("🔗 Uplink: {}", args.orchestrator_url);

    let client = Arc::new(WorkerClient::new(args.orchestrator_url.clone(), args.auth_token.clone()));

    // 1. AUTO-HIDRATACIÓN
    let filter_path = PathBuf::from("utxo_filter.bin");
    // Reintentar descarga hasta el infinito si falla (el nodo no sirve sin filtro)
    loop {
        match client.hydrate_filter(&filter_path).await {
            Ok(_) => break,
            Err(e) => {
                eprintln!("⚠️ Fallo de hidratación: {}. Reintentando en 5s...", e);
                sleep(Duration::from_secs(5)).await;
            }
        }
    }

    // 2. CARGA EN MEMORIA (Pesada)
    println!("🧠 Cargando red neuronal (Bloom Filter)...");
    let filter = Arc::new(RichListFilter::load_from_file(&filter_path)
        .context("Filtro corrupto o inválido")?);
    println!("✅ Sistema listo. {} direcciones indexadas.", filter.count());

    // 3. BUCLE DE TRABAJO INFINITO
    let running = Arc::new(AtomicBool::new(true)); // Control para graceful shutdown

    while running.load(Ordering::Relaxed) {
        println!("📡 Solicitando misión...");

        match client.get_job().await {
            Ok(job) => {
                println!("🔨 Ejecutando Misión ID: {}", job.id);

                // Procesamiento CPU INTENSIVO (Blocking)
                // Usamos spawn_blocking para no congelar el runtime asíncrono de Tokio
                let filter_ref = filter.clone();
                let client_ref = client.clone();
                let job_id = job.id;

                let _result = tokio::task::spawn_blocking(move || {
                    execute_strategy(job.strategy, &filter_ref, &client_ref)
                }).await?;
            }
            Err(e) => {
                eprintln!("💤 Esperando instrucciones ({})", e);
                sleep(Duration::from_secs(10)).await;
            }
        }
    }

    Ok(())
}

// Función síncrona pura para Rayon (SIMD)
fn execute_strategy(strategy: SearchStrategy, filter: &RichListFilter, client: &WorkerClient) {
    // Aquí implementamos la lógica según la estrategia recibida
    // Para esta fase de nivelación, asumimos 'Dictionary' como en el snapshot
    // Pero preparado para recibir rangos numéricos.

    let dictionary = match strategy {
        SearchStrategy::Dictionary { category } => {
            // En producción real, el diccionario vendría DENTRO del Job o descargado
            // Aquí usamos un mock para validar el flujo
            vec!["satoshi".to_string(), "bitcoin".to_string(), "123456".to_string()]
        },
        _ => vec![], // Placeholder para Sequential/Random
    };

    // Usamos par_iter de Rayon
    dictionary.par_iter().for_each(|phrase| {
        let private_key = prospector_domain_strategy::brainwallet::phrase_to_private_key(phrase);
        let pub_key = SafePublicKey::from_private(&private_key);
        let address = pubkey_to_address(&pub_key, false);

        if filter.contains(&address) {
            println!("🚨 HALLAZGO: {}", address);

            let finding = Finding {
                address: address.clone(),
                private_key_wif: private_to_wif(&private_key, false),
                source_entropy: format!("brainwallet:{}", phrase),
                wallet_type: "p2pkh".to_string(),
            };

            // Bridge al mundo async para reportar
            let rt = tokio::runtime::Handle::current();
            rt.block_on(async {
                let _ = client.report_finding(&finding).await;
            });
        }
    });
}
