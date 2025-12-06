# libs/domain/mining-strategy/src/lib.rs
// =================================================================
// APARATO: MINING STRATEGY
// ESTÁNDARES: RUST 2021
// =================================================================

//! # Mining Strategy
//!
//! Implementa los iteradores y generadores que ejecutan la búsqueda
//! real basándose en la configuración recibida.

pub mod brainwallet;
// pub mod sequential; // (Futuro)
// pub mod random;     // (Futuro)

// Re-exportamos para facilitar uso
pub use brainwallet::BrainwalletIterator;
