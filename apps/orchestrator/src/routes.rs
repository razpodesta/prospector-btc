// =================================================================
// APARATO: ORCHESTRATOR ROUTER
// RESPONSABILIDAD: ENRUTAMIENTO, MIDDLEWARE Y SEGURIDAD
// NIVEL: ELITE PRODUCTION
// =================================================================

use axum::{
    routing::{get, post},
    Router,
    middleware,
};

// Importamos los módulos internos necesarios
use crate::handlers;
use crate::state::AppState;
use crate::middleware::auth_guard;

/// Construye el enrutador principal de la aplicación.
///
/// # Arquitectura
/// - Define rutas protegidas por token (Mineros y Admin).
/// - Asigna el estado global (DB, Memoria).
/// - Anida todo bajo el prefijo `/api/v1` para versionado futuro.
pub fn create_router(state: AppState) -> Router {

    // Definición de Rutas Protegidas
    // Todas estas rutas requieren el header 'Authorization: Bearer <TOKEN>'
    let api_routes = Router::new()
        // -------------------------------------------------------------
        // SECCIÓN 1: OPERACIONES DE ENJAMBRE (WORKERS)
        // -------------------------------------------------------------

        // Recepción de latidos (Heartbeats) para monitoreo
        .route("/heartbeat", post(handlers::receive_heartbeat))

        // Asignación de rangos de trabajo (Jobs)
        .route("/job", post(handlers::assign_job))

        // Recepción de hallazgos criptográficos (Findings)
        .route("/finding", post(handlers::report_finding))

        // Consulta de estado del sistema (Dashboard Público)
        .route("/status", get(handlers::get_system_status))

        // -------------------------------------------------------------
        // SECCIÓN 2: THE IRON VAULT (ADMINISTRACIÓN DE IDENTIDAD)
        // -------------------------------------------------------------

        // Gestión de Inventario (Dashboard)
        // POST: Subir nueva identidad | GET: Listar identidades existentes
        .route("/admin/identities", post(handlers::upload_identity).get(handlers::list_identities))

        // Arrendamiento de Identidad (Provisioner Automático)
        // GET: Solicita una identidad activa para usar en un nodo
        .route("/admin/identities/lease", get(handlers::lease_identity))

        // -------------------------------------------------------------
        // CAPA DE SEGURIDAD
        // -------------------------------------------------------------
        // Aplicamos el middleware de autenticación a todas las rutas anteriores
        .route_layer(middleware::from_fn(auth_guard));

    // Ensamblaje final del Router
    // Anidamos las rutas protegidas bajo "/api/v1" y adjuntamos el estado
    Router::new()
        .nest("/api/v1", api_routes)
        .with_state(state)
}
