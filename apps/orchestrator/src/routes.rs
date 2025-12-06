# apps/orchestrator/src/routes.rs
use axum::{
    routing::{get, post},
    Router,
    middleware,
};
use crate::handlers;
use crate::state::AppState;
use crate::middleware::auth_guard; // <-- IMPORTAR

pub fn create_router(state: AppState) -> Router {
    // Rutas protegidas (Mineros)
    let api_routes = Router::new()
        .route("/heartbeat", post(handlers::receive_heartbeat))
        .route("/job", post(handlers::assign_job))
        .route("/finding", post(handlers::report_finding))
        // Protegemos el status también por seguridad (El dashboard necesitará el token)
        .route("/status", get(handlers::get_system_status))
        .route_layer(middleware::from_fn(auth_guard)); // <-- APLICAR MIDDLEWARE

    Router::new()
        .nest("/api/v1", api_routes) // Prefijo /api/v1
        .with_state(state)
}
