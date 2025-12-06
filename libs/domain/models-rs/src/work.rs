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
    /// Búsqueda secuencial combinatoria (ej: "pass0" ... "pass1000000")
    /// Ideal para fuerza bruta inteligente.
    Combinatoric {
        prefix: String,
        suffix: String,
        start_index: u64,
        end_index: u64,
    },

    /// Búsqueda basada en diccionario puro (lista de palabras).
    /// Menos eficiente para distribución masiva, pero útil para ataques dirigidos.
    Dictionary {
        dataset_url: String, // URL para descargar el diccionario
        limit: usize,
    },

    /// Búsqueda aleatoria pura (Monos escribiendo Shakespeare).
    /// Baja probabilidad, pero cobertura infinita.
    Random {
        seed: u64,
    },
}
