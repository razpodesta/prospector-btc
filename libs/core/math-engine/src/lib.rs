# libs/core/math-engine/src/lib.rs
// =================================================================
// APARATO: CORE MATH ENGINE
// ESTÁNDARES: RUST 2021, CLIPPY STRICT, SAFE MEMORY
// =================================================================

// REGLAS DE SEGURIDAD Y LINTING
// Prohibimos código inseguro (unsafe) a menos que esté estrictamente auditado.
#![deny(unsafe_code)]
// Forzamos documentación para mantener el estándar doctoral.
#![warn(missing_docs)]
// Activamos lints de Clippy para código idiomático y performante.
#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

//! # Core Math Engine
//!
//! Este es el núcleo criptográfico de Prospector.
//!
//! ## Responsabilidades:
//! 1. Wrapper seguro sobre `libsecp256k1`.
//! 2. Implementación de Hashing específico de Bitcoin (Hash160).
//! 3. Gestión de tipos base para Claves Privadas y Públicas.
//!
//! ## Rendimiento:
//! Utiliza bindings C optimizados con ensamblador para la curva elíptica.

/// Módulo para operaciones de Hashing (SHA256, RIPEMD160).
pub mod hashing;

/// Módulo para gestión de Claves Privadas.
pub mod private_key;

/// Módulo para gestión de Claves Públicas.
pub mod public_key;

/// Tipos de error unificados para el módulo matemático.
pub mod errors;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
