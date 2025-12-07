use secp256k1::{PublicKey, Secp256k1};
use crate::private_key::SafePrivateKey;

/// Wrapper para Clave Pública.
pub struct SafePublicKey {
    inner: PublicKey,
}

impl SafePublicKey {
    pub fn from_private(private: &SafePrivateKey) -> Self {
        let secp = Secp256k1::new();
        let public_key = PublicKey::from_secret_key(&secp, private.as_inner());
        Self { inner: public_key }
    }

    pub fn to_bytes(&self, compressed: bool) -> Vec<u8> {
        if compressed {
            self.inner.serialize().to_vec()
        } else {
            self.inner.serialize_uncompressed().to_vec()
        }
    }
}
