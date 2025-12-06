# libs/domain/mining-strategy/src/brainwallet.rs
use prospector_core_math::private_key::SafePrivateKey;
use prospector_core_math::hashing::double_sha256; // O sha256 simple según el caso
use sha2::{Sha256, Digest};

/// Convierte una frase de texto (passphrase) en una Clave Privada.
///
/// # Vulnerabilidad Histórica
/// Muchos usuarios usaron `SHA256("texto")` directamente como clave privada.
/// Esta función replica ese error humano.
pub fn phrase_to_private_key(phrase: &str) -> SafePrivateKey {
    // 1. Hashear la frase
    let mut hasher = Sha256::new();
    hasher.update(phrase.as_bytes());
    let result = hasher.finalize();

    // 2. Convertir bytes a Clave Privada Segura
    // SafePrivateKey maneja internamente la verificación de curva válida.
    // En el rarísimo caso de que el hash no sea un escalar válido,
    // podríamos paniquear o retornar Result (simplificado aquí para la demo).
    SafePrivateKey::from_bytes(&result).expect("Hash SHA256 inválido para curva secp256k1")
}

/// Un iterador que toma una lista de palabras y genera claves candidatas.
pub struct BrainwalletIterator<'a> {
    dictionary: &'a [String],
    index: usize,
}

impl<'a> BrainwalletIterator<'a> {
    pub fn new(dictionary: &'a [String]) -> Self {
        Self { dictionary, index: 0 }
    }
}

impl<'a> Iterator for BrainwalletIterator<'a> {
    type Item = (String, SafePrivateKey);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.dictionary.len() {
            return None;
        }

        let phrase = &self.dictionary[self.index];
        self.index += 1;

        let private_key = phrase_to_private_key(phrase);

        Some((phrase.clone(), private_key))
    }
}
