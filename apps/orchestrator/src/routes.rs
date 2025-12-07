use axum::{
    routing::{get, post},
    Router,
    middleware,
};
use crate::handlers;
use crate::state::AppState;
use crate::middleware::auth_guard;

pub fn create_router(state: AppState) -> Router {
    // 1. Rutas Protegidas (Solo Mineros con Token)
    let protected_routes = Router::new()
        .route("/heartbeat", post(handlers::receive_heartbeat))
        .route("/job", post(handlers::assign_job))
        .route("/finding", post(handlers::report_finding))
        .route_layer(middleware::from_fn(auth_guard));

    // 2. Rutas Públicas (Healthchecks de Render/Docker y Dashboard sin Auth)
    // CRÍTICO: Esto debe estar fuera del auth_guard para evitar bucles de error 401 en los logs
    let public_routes = Router::new()
        .route("/status", get(handlers::get_system_status));

    // 3. Fusión de Rutas
    Router::new()
        .nest("/api/v1", protected_routes)
        .nest("/api/v1", public_routes)
        .with_state(state)
}
