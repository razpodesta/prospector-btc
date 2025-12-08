// libs/domain/models-rs/src/work.rs

use serde::{Serialize, Deserialize};
use uuid::Uuid;

/// Define una unidad de trabajo asignada a un Minero.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkOrder {
    pub id: Uuid,
    pub strategy: SearchStrategy,
    pub target_duration_sec: u64,
}

/// Tipos de estrategias de búsqueda soportadas.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "params")]
pub enum SearchStrategy {
    /// Búsqueda secuencial numérica (Fuerza Bruta Inteligente).
    Combinatoric {
        prefix: String,
        suffix: String,
        start_index: u64,
        end_index: u64,
    },

    /// Búsqueda basada en diccionario (Brainwallets).
    Dictionary {
        dataset_url: String, // URL para descargar el diccionario si no está en caché
        limit: usize,
    },

    /// Búsqueda de vulnerabilidades históricas específicas (Arqueología).
    ForensicScan {
        target: ForensicTarget,
        range_start: u64,
        range_end: u64,
    },

    /// Búsqueda aleatoria pura (Cobertura infinita).
    Random {
        seed: u64,
    },
}

/// Objetivos forenses conocidos.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ForensicTarget {
    /// CVE-2008-0166: Fallo en el generador de números aleatorios de OpenSSL en Debian.
    /// El espacio de claves se redujo al PID del proceso (max 32,767).
    DebianOpenSSL,

    /// Patrones de PRNG débiles en implementaciones antiguas de Android/Java.
    AndroidSecureRandom,
}
