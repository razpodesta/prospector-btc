// libs/domain/forensics/src/debian_rng.rs
// use prospector_core_math::private_key::SafePrivateKey; // <-- ELIMINADO

/// Verifica si una clave privada podría haber sido generada por el bug de Debian.
pub struct DebianWeakDetector;

impl DebianWeakDetector {
    /// En un escenario real, cargaríamos la lista negra de claves públicas conocidas.
    pub fn is_weak_pid_seed(seed: u64) -> bool {
        // En Debian OpenSSL bug, la semilla era el PID (0-32767)
        seed <= 32767
    }
}
