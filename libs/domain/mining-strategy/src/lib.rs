// libs/domain/mining-strategy/src/lib.rs
// =================================================================
// APARATO: MINING STRATEGY
// MISION: Generación de candidatos de alta velocidad
// =================================================================

pub mod brainwallet;
pub mod combinatoric; // <--- NUEVO MOTOR

// Re-exports
pub use brainwallet::BrainwalletIterator;
pub use combinatoric::CombinatoricIterator;
