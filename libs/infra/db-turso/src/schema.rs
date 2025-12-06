// libs/infra/db-turso/src/schema.rs

pub const INITIAL_SCHEMA: &str = r#"
-- Tabla de Hallazgos (El Tesoro)
CREATE TABLE IF NOT EXISTS findings (
    id TEXT PRIMARY KEY,
    address TEXT NOT NULL,
    private_key_wif TEXT NOT NULL,
    source_entropy TEXT,
    wallet_type TEXT,
    found_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Tabla de Control de Rangos (Para evitar colisiones de trabajo)
CREATE TABLE IF NOT EXISTS range_cursor (
    category TEXT PRIMARY KEY, -- ej: 'combinatoric_numeric'
    last_index INTEGER NOT NULL
);

-- Tabla de Workers (Telemetría)
CREATE TABLE IF NOT EXISTS workers (
    worker_id TEXT PRIMARY KEY,
    hostname TEXT,
    last_heartbeat DATETIME DEFAULT CURRENT_TIMESTAMP,
    current_hashrate INTEGER
);

-- Inicializar cursor si no existe
INSERT OR IGNORE INTO range_cursor (category, last_index) VALUES ('combinatoric_v1', 0);
"#;
