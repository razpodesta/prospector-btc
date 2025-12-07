
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GenError {
    #[error("Error codificando Base58: {0}")]
    Base58Error(#[from] bs58::encode::Error),

    #[error("Error matemático subyacente: {0}")]
    MathError(#[from] prospector_core_math::errors::MathError),
}
