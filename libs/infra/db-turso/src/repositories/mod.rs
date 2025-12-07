// libs/infra/db-turso/src/repositories/mod.rs

// Declaramos los módulos hijos
pub mod finding;
pub mod job;

// CORRECCIÓN CRÍTICA: Re-exportamos las structs para que sean accesibles
// Esto permite usar repositories::FindingRepository en lugar de repositories::finding::FindingRepository
pub use finding::FindingRepository;
pub use job::JobRepository;
