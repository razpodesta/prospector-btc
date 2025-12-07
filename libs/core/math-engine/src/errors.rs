use thiserror::Error;

/// Errores posibles dentro del motor matemático.
#[derive(Error, Debug)]
pub enum MathError {
    #[error("Formato de clave inválido: {0}")]
    InvalidKeyFormat(String),

    #[error("Error en operación criptográfica de curva elíptica")]
    EllipticCurveError(#[from] secp256k1::Error),

    #[error("Longitud de datos incorrecta. Esperado {expected}, recibido {got}")]
    InvalidLength { expected: usize, got: usize },
}
