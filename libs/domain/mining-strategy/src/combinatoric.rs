// libs/domain/mining-strategy/src/combinatoric.rs
use prospector_core_math::private_key::SafePrivateKey;
use prospector_core_math::hashing::double_sha256; // O sha256 simple, depende del modelo de amenaza

/// Generador de entropía secuencial.
/// Convierte índices (u64) en frases y luego en claves privadas.
pub struct CombinatoricIterator {
    current: u64,
    end: u64,
    prefix: String,
    suffix: String,
    // Buffer reutilizable para evitar allocations constantes
    buffer: String,
}

impl CombinatoricIterator {
    pub fn new(start: u64, end: u64, prefix: String, suffix: String) -> Self {
        // Pre-reserva capacidad para evitar reallocs
        let capacity = prefix.len() + suffix.len() + 20;
        Self {
            current: start,
            end,
            prefix,
            suffix,
            buffer: String::with_capacity(capacity),
        }
    }
}

impl Iterator for CombinatoricIterator {
    type Item = (String, SafePrivateKey);

    fn next(&mut self) -> Option<Self::Item> {
        if self.current >= self.end {
            return None;
        }

        // Construcción de frase eficiente: "prefix" + "123" + "suffix"
        self.buffer.clear();
        self.buffer.push_str(&self.prefix);
        self.buffer.push_str(&self.current.to_string());
        self.buffer.push_str(&self.suffix);

        self.current += 1;

        // NOTA: Esta clonación es inevitable porque retornamos el String para el reporte.
        // En una optimización extrema futura, podríamos retornar solo la PK y regenerar el String solo si hay éxito.
        let phrase = self.buffer.clone();

        // Generación de PK (Simulando Brainwallet SHA256)
        // Usamos SHA256 directo sobre la frase
        // importamos lógica local o duplicamos para evitar dep circular compleja,
        // pero lo ideal es usar el brainwallet generator
        let pk = crate::brainwallet::phrase_to_private_key(&phrase);

        Some((phrase, pk))
    }
}
