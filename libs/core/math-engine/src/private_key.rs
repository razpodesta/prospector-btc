// libs/core/math-engine/src/private_key.rs
use secp256k1::{SecretKey, Secp256k1};
use secp256k1::rand::rngs::OsRng;
use crate::errors::MathError;

/// Wrapper seguro para una clave privada.
pub struct SafePrivateKey {
    inner: SecretKey,
}

impl SafePrivateKey {
    /// Genera una nueva clave privada usando entropía segura del SO.
    pub fn new_random() -> Self {
        let secp = Secp256k1::new();
        let (secret_key, _) = secp.generate_keypair(&mut OsRng);
        Self { inner: secret_key }
    }

    /// Crea una clave privada desde bytes crudos (32 bytes).
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, MathError> {
        if bytes.len() != 32 {
            return Err(MathError::InvalidLength { expected: 32, got: bytes.len() });
        }
        let sk = SecretKey::from_slice(bytes).map_err(MathError::EllipticCurveError)?;
        Ok(Self { inner: sk })
    }

    /// Obtiene los 32 bytes crudos de la clave privada.
    /// Vital para serialización WIF.
    pub fn to_bytes(&self) -> [u8; 32] {
        self.inner.secret_bytes()
    }

    /// Obtiene la referencia interna a la clave de secp256k1.
    /// (Solo para uso interno del crate o interop avanzada).
    pub fn as_inner(&self) -> &SecretKey {
        &self.inner
    }
}
