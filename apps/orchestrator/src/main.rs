// apps/orchestrator/src/main.rs
// =================================================================
// PROSPECTOR SYSTEM // APARATO: ORCHESTRATOR
// CLASIFICACIÓN: CORE INFRASTRUCTURE
// ESTÁNDARES: RUST 2021, TOKIO ASYNC, AXUM
// =================================================================

use dotenvy::dotenv;
use std::net::SocketAddr;
use std::process;
use tracing::{info, error, warn};
use prospector_infra_db::TursoClient;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;

// Módulos internos de la arquitectura hexagonal
mod state;
mod handlers;
mod routes;
mod middleware;

use crate::state::AppState;

/// Punto de entrada principal del Orquestador (API Server).
///
/// # Responsabilidades
/// 1. **Bootstrap del Entorno:** Carga variables y configura logging.
/// 2. **Conexión a Infraestructura:** Establece el enlace persistente con Turso DB.
/// 3. **Logística Táctica:** Sirve el archivo `utxo_filter.bin` a los workers.
/// 4. **Enrutamiento:** Expone la API REST para el enjambre de mineros.
///
/// # Panics
/// Este binario entrará en pánico (Crash intencional) si:
/// - No puede conectarse a la Base de Datos (integridad crítica).
/// - No puede vincularse al puerto TCP asignado.
#[tokio::main]
async fn main() {
    // -----------------------------------------------------------------
    // FASE 1: INICIALIZACIÓN DEL ENTORNO
    // -----------------------------------------------------------------

    // Intentamos cargar .env para desarrollo local.
    // En producción (Render), las variables son inyectadas por el orquestador del contenedor.
    dotenv().ok();

    // Configuración del Sistema de Observabilidad (Protocolo Heimdall)
    // Filtramos para ver logs de INFO de nuestra app y logs de tráfico HTTP.
    tracing_subscriber::fmt()
        .with_env_filter("prospector_orchestrator=info,tower_http=info")
        .with_target(false) // Limpia el output eliminando rutas de módulos ruidosas
        .compact()          // Formato optimizado para logs de contenedores
        .init();

    info!("🚀 SYSTEM STARTUP: INICIANDO PROSPECTOR ORCHESTRATOR [ELITE MODE]");

    // -----------------------------------------------------------------
    // FASE 2: CONEXIÓN A INFRAESTRUCTURA DE DATOS
    // -----------------------------------------------------------------

    // Obtención de credenciales de forma segura
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "file:prospector.db".to_string());

    let db_token = std::env::var("TURSO_AUTH_TOKEN").ok();

    info!("🔌 Estableciendo enlace neuronal con Base de Datos: {}", db_url);

    // Conexión Estricta (Fail Fast)
    // Si la DB no responde, el orquestador es inútil. Abortamos inicio.
    let db_client = match TursoClient::connect(&db_url, db_token).await {
        Ok(client) => {
            info!("✅ Conexión a Turso/libSQL establecida y verificada.");
            client
        },
        Err(e) => {
            error!("❌ ERROR CRÍTICO DE INFRAESTRUCTURA: Fallo al conectar DB.");
            error!("🔎 Diagnóstico: {}", e);
            error!("💀 Abortando inicio del sistema.");
            process::exit(1);
        }
    };

    // -----------------------------------------------------------------
    // FASE 3: CONSTRUCCIÓN DEL ESTADO Y SEGURIDAD
    // -----------------------------------------------------------------

    // Inicialización de memoria volátil compartida (Arc<RwLock>)
    let state = AppState::new(db_client);

    // Configuración de CORS (Cross-Origin Resource Sharing)
    // Se mantiene permisivo ("permissive") intencionalmente para permitir:
    // 1. Conexiones desde Dashboards en Vercel (distintos dominios).
    // 2. Reportes desde Mineros en Colab (IPs dinámicas/Google Cloud).
    let cors = CorsLayer::permissive();

    // Configuración de Logística de Archivos (Hydra-Zero)
    // Mapeamos el directorio local "public" (inyectado por Docker) a la web.
    // Esto permite la auto-hidratación de los mineros.
    let public_path = "public";
    let static_files = ServeDir::new(public_path);

    // Verificación de existencia del directorio público (Warn si no existe)
    if !std::path::Path::new(public_path).exists() {
        warn!("⚠️ ADVERTENCIA DE LOGÍSTICA: El directorio '{}' no existe.", public_path);
        warn!("   Los mineros NO podrán descargar el filtro UTXO.");
    } else {
        info!("📦 Sistema de Logística activo. Sirviendo desde: ./{}", public_path);
    }

    // -----------------------------------------------------------------
    // FASE 4: ENRUTAMIENTO Y LANZAMIENTO
    // -----------------------------------------------------------------

    // Composición del Router (Axum)
    let app = routes::create_router(state)
        // Ruta de Abastecimiento: GET /resources/utxo_filter.bin
        .nest_service("/resources", static_files)
        // Capa de Seguridad Perimetral
        .layer(cors);

    // Configuración del Puerto (Render inyecta PORT)
    let port_str = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let port: u16 = port_str.parse().unwrap_or_else(|_| {
        warn!("⚠️ Variable PORT malformada, usando puerto fallback 3000");
        3000
    });

    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    info!("✅ ORCHESTRATOR OPERATIVO");
    info!("📡 Escuchando tráfico HTTP en: {}", addr);
    info!("🛡️  Ruta de API: /api/v1");
    info!("💊 Ruta de Recursos: /resources");

    // Vinculación TCP y Bucle de Eventos
    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(l) => l,
        Err(e) => {
            error!("❌ ERROR DE RED: No se pudo vincular al puerto {}", port);
            error!("🔎 Diagnóstico: {}", e);
            process::exit(1);
        }
    };

    // Inicio del Servidor
    if let Err(e) = axum::serve(listener, app).await {
        error!("❌ FALLO CATASTRÓFICO DEL SERVIDOR EN EJECUCIÓN: {}", e);
        process::exit(1);
    }
}
