// =================================================================
// APARATO: ORCHESTRATOR HANDLERS
// RESPONSABILIDAD: CONTROLADOR LÓGICO DE LA API (MVC)
// NIVEL: ELITE PRODUCTION
// =================================================================

use axum::{
    extract::{State, Json, Query},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Deserialize;
use tracing::{info, warn, error};

// IMPORTACIONES DE DOMINIO E INFRAESTRUCTURA
use crate::state::AppState;
use prospector_domain_models::{
    worker::WorkerHeartbeat,
    finding::Finding,
    identity::{CreateIdentityPayload, Identity}
};
use prospector_infra_db::repositories::{
    FindingRepository,
    JobRepository,
    IdentityRepository
};

// --- DTOs LOCALES ---

/// Parámetros para solicitar una identidad (Lease).
#[derive(Deserialize)]
pub struct LeaseParams {
    pub platform: String,
}

// =================================================================
// SECCIÓN 1: OPERACIONES DE MINERÍA (WORKERS)
// =================================================================

/// Recibe el latido de un minero para actualizar el mapa de calor del sistema.
pub async fn receive_heartbeat(
    State(state): State<AppState>,
    Json(heartbeat): Json<WorkerHeartbeat>,
) -> impl IntoResponse {
    // Actualización en memoria (rápida)
    state.update_worker(heartbeat);
    StatusCode::OK
}

/// Asigna un nuevo rango de búsqueda a un minero solicitante.
/// Utiliza Optimistic Concurrency Control en la DB.
pub async fn assign_job(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let repo = JobRepository::new(state.db.clone());

    match repo.assign_next_range().await {
        Ok(work_order) => {
            info!("💼 Trabajo asignado: ID {} [{:?}]", work_order.id, work_order.strategy);
            Json(work_order).into_response()
        },
        Err(e) => {
            error!("❌ Error crítico asignando trabajo: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

/// Recibe un hallazgo exitoso (Colisión/Wallet) y lo persiste.
pub async fn report_finding(
    State(state): State<AppState>,
    Json(finding): Json<Finding>,
) -> impl IntoResponse {
    warn!("🚨 HALLAZGO REPORTADO: {} | Source: {}", finding.address, finding.source_entropy);

    let repo = FindingRepository::new(state.db.clone());

    match repo.save(&finding).await {
        Ok(_) => {
            info!("💾 Hallazgo persistido en Turso exitosamente.");
            StatusCode::CREATED
        },
        Err(e) => {
            error!("❌ ERROR CRÍTICO DB: No se guardó el hallazgo: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

/// Retorna el estado actual de la flota de workers (para el Dashboard).
pub async fn get_system_status(State(state): State<AppState>) -> Json<Vec<WorkerHeartbeat>> {
    Json(state.get_active_workers())
}

// =================================================================
// SECCIÓN 2: THE IRON VAULT (GESTIÓN DE IDENTIDADES)
// =================================================================

/// ADMIN: Carga o actualiza una identidad (Cookies/Credenciales).
pub async fn upload_identity(
    State(state): State<AppState>,
    Json(payload): Json<CreateIdentityPayload>,
) -> impl IntoResponse {
    let repo = IdentityRepository::new(state.db.clone());

    match repo.upsert(&payload).await {
        Ok(_) => {
            info!("🔐 Identidad asegurada en Bóveda: {} ({})", payload.email, payload.platform);
            StatusCode::CREATED
        },
        Err(e) => {
            error!("❌ Error guardando identidad: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

/// ADMIN: Lista todas las identidades disponibles y su estado.
pub async fn list_identities(
    State(state): State<AppState>,
) -> Json<Vec<Identity>> {
    let repo = IdentityRepository::new(state.db.clone());
    match repo.list_all().await {
        Ok(list) => Json(list),
        Err(e) => {
            error!("❌ Error listando inventario de identidades: {}", e);
            Json(vec![])
        }
    }
}

/// PROVISIONER: Solicita ("Arrienda") una identidad activa para usarla.
/// Implementa lógica de rotación (Least Used).
pub async fn lease_identity(
    State(state): State<AppState>,
    Query(params): Query<LeaseParams>,
) -> impl IntoResponse {
    let repo = IdentityRepository::new(state.db.clone());

    match repo.lease_active(&params.platform).await {
        Ok(Some(identity)) => {
            info!("🎟️ Identidad arrendada a nodo: {} -> {}", identity.platform, identity.email);
            // Entregamos la identidad completa. El cliente extraerá el JSON de cookies.
            Json(Some(identity)).into_response()
        },
        Ok(None) => {
            warn!("⚠️ Stock agotado: No hay identidades activas para '{}'", params.platform);
            StatusCode::NOT_FOUND.into_response()
        },
        Err(e) => {
            error!("❌ Error transaccional al arrendar identidad: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
