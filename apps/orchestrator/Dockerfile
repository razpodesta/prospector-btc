
# =================================================================
# APARATO: DOCKER BUILDER (ZERO-COST STRATEGY)
# MISIÓN: Compilar código + Inyectar Filtro UTXO externo
# =================================================================

# --- ETAPA 1: BUILDER (Compilación Rust) ---
FROM rust:1.75-slim-bookworm as builder

WORKDIR /usr/src/app

# Instalar dependencias del sistema para compilar crates criptográficos
RUN apt-get update && apt-get install -y pkg-config libssl-dev musl-tools

# Copiar todo el contexto del Monorepo (Libs + Apps)
# Es necesario copiar todo porque 'orchestrator' depende de 'libs/core/...'
COPY . .

# Compilar en modo Release optimizado
# Usamos -p para especificar el paquete del orquestador dentro del workspace
RUN cargo build --release -p prospector-orchestrator

# --- ETAPA 2: RUNTIME (Imagen Final) ---
FROM debian:bookworm-slim

# Instalar:
# - ca-certificates: Para poder hacer peticiones HTTPS (Vital para descargar filtro y para Turso)
# - libssl3: Runtime de OpenSSL
# - curl: Para descargar el filtro durante el build
RUN apt-get update && \
    apt-get install -y ca-certificates libssl3 curl && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

# 1. Crear directorio para recursos estáticos
RUN mkdir -p public

# 2. INYECCIÓN DEL FILTRO (Estrategia Zero-Cost)
# Argumento de construcción con URL por defecto (CAMBIAR ESTO POR TU URL REAL)
# Ejemplo: https://github.com/TU_USUARIO/prospector-btc/releases/download/v0.1/utxo_filter.bin
ARG FILTER_URL="https://github.com/CHANGE_THIS_TO_YOUR_REAL_GITHUB_RELEASE_URL/utxo_filter.bin"

# Descarga y verificación simple.
# El flag -f hace que curl falle si el servidor da error (ej. 404), deteniendo el build.
RUN echo "⬇️ INICIANDO PROTOCOLO DE HIDRATACIÓN..." && \
    echo "🔗 Origen: $FILTER_URL" && \
    curl -L -f -o public/utxo_filter.bin "$FILTER_URL" && \
    echo "✅ Filtro UTXO inyectado en /app/public/utxo_filter.bin" || \
    (echo "❌ ERROR FATAL: No se pudo descargar el filtro. Verifica la URL." && exit 1)

# 3. Copiar el binario compilado desde la etapa anterior
COPY --from=builder /usr/src/app/target/release/orchestrator /app/orchestrator

# 4. Configurar Entorno
ENV RUST_LOG=info
ENV PORT=3000
# Valor por defecto para DB (Render inyectará la real)
ENV DATABASE_URL="file:prospector.db"

EXPOSE 3000

# 5. Ejecutar
CMD ["./orchestrator"]
