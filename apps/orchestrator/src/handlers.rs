// apps/orchestrator/src/handlers.rs
use axum::{extract::{State, Json}, http::StatusCode, response::IntoResponse};
use prospector_domain_models::{worker::WorkerHeartbeat, finding::Finding}; // <-- WorkOrder eliminado (se infiere)
use prospector_infra_db::repositories::{FindingRepository, JobRepository};
use tracing::{info, warn, error};
use crate::state::AppState;

pub async fn receive_heartbeat(
    State(state): State<AppState>,
    Json(heartbeat): Json<WorkerHeartbeat>,
) -> impl IntoResponse {
    state.update_worker(heartbeat);
    StatusCode::OK
}

pub async fn assign_job(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let repo = JobRepository::new(state.db.clone());

    match repo.assign_next_range().await {
        Ok(work_order) => {
            info!("💼 Trabajo asignado: ID {}", work_order.id);
            // Rust infiere que esto es un WorkOrder, no necesitamos importarlo explícitamente
            Json(work_order).into_response()
        },
        Err(e) => {
            error!("❌ Error asignando trabajo: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub async fn report_finding(
    State(state): State<AppState>,
    Json(finding): Json<Finding>,
) -> impl IntoResponse {
    warn!("🚨 HALLAZGO REPORTADO: {}", finding.address);

    let repo = FindingRepository::new(state.db.clone());

    match repo.save(&finding).await {
        Ok(_) => {
            info!("💾 Hallazgo persistido en Turso exitosamente.");
            StatusCode::CREATED
        },
        Err(e) => {
            error!("❌ ERROR CRÍTICO: No se guardó el hallazgo: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

pub async fn get_system_status(State(state): State<AppState>) -> Json<Vec<WorkerHeartbeat>> {
    Json(state.get_active_workers())
}
