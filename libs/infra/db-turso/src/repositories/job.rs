// libs/infra/db-turso/src/repositories/job.rs
use crate::TursoClient;
use crate::errors::DbError;
use prospector_domain_models::work::{WorkOrder, SearchStrategy};
use uuid::Uuid;
use libsql::params;

pub struct JobRepository {
    client: TursoClient,
}

impl JobRepository {
    pub fn new(client: TursoClient) -> Self {
        Self { client }
    }

    /// Asigna rango usando Optimistic Concurrency Control (OCC).
    pub async fn assign_next_range(&self) -> Result<WorkOrder, DbError> {
        let conn = self.client.get_connection()?;
        const CHUNK_SIZE: u64 = 10_000_000;
        const CATEGORY: &str = "combinatoric_v1";

        // BUCLE DE REINTENTO OPTIMISTA (Max 5 intentos)
        for _ in 0..5 {
            // 1. Leer estado actual
            let mut rows = conn.query(
                "SELECT last_index FROM range_cursor WHERE category = ?1",
                params![CATEGORY]
            ).await?;

            let current_last: u64 = if let Some(row) = rows.next().await? {
                row.get(0)?
            } else {
                // Auto-seed inicial si la tabla está vacía
                conn.execute(
                    "INSERT OR IGNORE INTO range_cursor (category, last_index) VALUES (?1, 0)",
                    params![CATEGORY]
                ).await?;
                0
            };

            let new_last = current_last + CHUNK_SIZE;

            // 2. Actualización Atómica Condicional
            // "Actualiza SOLO SI el valor en DB sigue siendo el que leí hace un milisegundo"
            let changed = conn.execute(
                "UPDATE range_cursor SET last_index = ?1 WHERE category = ?2 AND last_index = ?3",
                params![new_last, CATEGORY, current_last]
            ).await?;

            if changed > 0 {
                // ✅ Éxito: Ganamos la carrera y reservamos el rango [current_last, new_last)
                return Ok(WorkOrder {
                    id: Uuid::new_v4(),
                    target_duration_sec: 60,
                    strategy: SearchStrategy::Combinatoric {
                        prefix: "".to_string(),
                        suffix: "".to_string(),
                        start_index: current_last,
                        end_index: new_last,
                    },
                });
            }
            // ❌ Fallo: Alguien más actualizó. El bucle se repite y lee el nuevo valor.
        }

        Err(DbError::TransactionError) // Alta contención (demasiados workers a la vez)
    }
}
