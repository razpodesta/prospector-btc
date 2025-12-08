// =================================================================
// APARATO: DB SCHEMA DEFINITION
// RESPONSABILIDAD: DEFINICIÓN DEL ESTADO INICIAL DE LA BASE DE DATOS
// =================================================================

/// Script SQL de inicialización.
/// Se ejecuta automáticamente al conectar si las tablas no existen.
///
/// NOTA: Para aplicar cambios en la tabla 'identities' en una DB existente,
/// se debe hacer DROP manual o borrar el archivo de DB local.
pub const INITIAL_SCHEMA: &str = r#"
-- 1. TABLA DE HALLAZGOS (CORE VALUE)
-- Almacena las colisiones encontradas.
CREATE TABLE IF NOT EXISTS findings (
    id TEXT PRIMARY KEY,
    address TEXT NOT NULL,
    private_key_wif TEXT NOT NULL,
    source_entropy TEXT,
    wallet_type TEXT,
    found_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- 2. TABLA DE CURSOR DE RANGOS (ORCHESTRATION)
-- Controla qué partes del espacio de búsqueda han sido asignadas.
CREATE TABLE IF NOT EXISTS range_cursor (
    category TEXT PRIMARY KEY,
    last_index INTEGER NOT NULL
);

-- 3. TABLA DE WORKERS (TELEMETRY)
-- Estado efímero de los nodos de procesamiento.
CREATE TABLE IF NOT EXISTS workers (
    worker_id TEXT PRIMARY KEY,
    hostname TEXT,
    last_heartbeat DATETIME DEFAULT CURRENT_TIMESTAMP,
    current_hashrate INTEGER
);

-- 4. TABLA DE IDENTIDADES (THE IRON VAULT v2.0)
-- Gestión de credenciales para el Provisioner.
-- Actualizada para soportar rotación, email y contadores de uso.
CREATE TABLE IF NOT EXISTS identities (
    id TEXT PRIMARY KEY,
    platform TEXT NOT NULL,       -- ej: 'google_colab'
    email TEXT NOT NULL,          -- Identificador de cuenta
    credentials_json TEXT NOT NULL, -- Cookies (Sensitive)
    user_agent TEXT NOT NULL,     -- Fingerprint evasion
    usage_count INTEGER DEFAULT 0,
    last_used_at DATETIME,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    status TEXT DEFAULT 'active', -- active, expired, ratelimited

    -- Restricción: Un solo registro por email en cada plataforma
    UNIQUE(platform, email)
);

-- SEED DATA: Inicializar cursor combinatorio si no existe
INSERT OR IGNORE INTO range_cursor (category, last_index) VALUES ('combinatoric_v1', 0);
"#;
