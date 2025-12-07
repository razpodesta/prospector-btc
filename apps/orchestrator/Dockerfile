# apps/orchestrator/Dockerfile
# =================================================================
# APARATO: BACKEND CONTAINER (RENDER PRODUCTION)
# =================================================================

# --- 1. BUILDER ---
# Usamos la última versión estable para evitar vulnerabilidades conocidas
FROM rust:1.83-slim-bookworm as builder

WORKDIR /usr/src/app

# Instalar dependencias de sistema mínimas para compilar
RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev musl-tools curl && \
    rm -rf /var/lib/apt/lists/*

# Copiar todo el contexto (Render lo hace desde la raíz)
COPY . .

# Diagnóstico de Build
RUN echo "🏗️  Iniciando compilación de Orchestrator en modo Release..."
RUN cargo --version

# Compilar en modo Release optimizado
RUN cargo build --release -p prospector-orchestrator

# --- 2. RUNTIME ---
FROM debian:bookworm-slim

# Instalar certificados y utilidades vitales
RUN apt-get update && \
    apt-get install -y ca-certificates libssl3 curl && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Crear estructura pública
RUN mkdir -p public

# INYECCIÓN DE FILTRO (Con fallback para que no falle el deploy inicial)
ARG FILTER_URL=""
RUN if [ -n "$FILTER_URL" ]; then \
      echo "⬇️ Descargando filtro desde: $FILTER_URL"; \
      curl -L -f -o public/utxo_filter.bin "$FILTER_URL"; \
    else \
      echo "⚠️  FILTER_URL no definida. Creando archivo dummy para inicio."; \
      touch public/utxo_filter.bin; \
    fi

# Copiar binario
COPY --from=builder /usr/src/app/target/release/orchestrator /usr/local/bin/orchestrator

# Configurar Entorno
ENV RUST_LOG=info
ENV PORT=3000

# Puerto expuesto (Documentación)
EXPOSE 3000

# Healthcheck nativo
HEALTHCHECK --interval=30s --timeout=3s \
  CMD curl -f http://localhost:3000/api/v1/health || exit 1

CMD ["orchestrator"]
