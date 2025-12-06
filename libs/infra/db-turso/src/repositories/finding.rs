// libs/infra/db-turso/src/repositories/finding.rs
use crate::TursoClient;
use crate::errors::DbError;
use prospector_domain_models::finding::Finding;
use uuid::Uuid;

pub struct FindingRepository {
    client: TursoClient,
}

impl FindingRepository {
    pub fn new(client: TursoClient) -> Self {
        Self { client }
    }

    pub async fn save(&self, finding: &Finding) -> Result<(), DbError> {
        let conn = self.client.get_connection()?;

        // Usamos INSERT OR IGNORE para idempotencia (si el worker reenvía el mismo hallazgo)
        let query = "INSERT OR IGNORE INTO findings (id, address, private_key_wif, source_entropy, wallet_type) VALUES (?1, ?2, ?3, ?4, ?5)";

        conn.execute(query, (
            Uuid::new_v4().to_string(),
            &finding.address,
            &finding.private_key_wif,
            &finding.source_entropy,
            &finding.wallet_type
        )).await?;

        Ok(())
    }
}
