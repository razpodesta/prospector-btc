# apps/orchestrator/Dockerfile
# =================================================================
# APARATO: BACKEND CONTAINER (RENDER PRODUCTION)
# ESTADO: BLINDADO & PARCHEADO
# =================================================================

# --- 1. BUILDER (Compilador) ---
FROM rust:1.83-slim-bookworm as builder

WORKDIR /usr/src/app

# 1.1. Actualización de Seguridad del OS (Elimina vulnerabilidades críticas)
RUN apt-get update && \
    apt-get upgrade -y && \
    apt-get install -y pkg-config libssl-dev musl-tools curl git && \
    rm -rf /var/lib/apt/lists/*

# 1.2. Copiar Contexto
COPY . .

# 1.3. Diagnóstico
RUN echo "🏗️  Iniciando compilación segura..." && \
    rustc --version

# 1.4. Compilación Release
# Usamos --locked para asegurar que se use tu Cargo.lock con el fix de 'home'
RUN cargo build --release --locked -p prospector-orchestrator

# --- 2. RUNTIME (Ejecución Ligera) ---
FROM debian:bookworm-slim

WORKDIR /app

# 2.1. Actualización de Seguridad en Runtime
RUN apt-get update && \
    apt-get upgrade -y && \
    apt-get install -y ca-certificates libssl3 curl && \
    rm -rf /var/lib/apt/lists/*

# 2.2. Estructura Pública
RUN mkdir -p public

# 2.3. Inyección de Filtro (Con Fallback de Seguridad)
ARG FILTER_URL=""
RUN if [ -n "$FILTER_URL" ]; then \
      echo "⬇️ Descargando filtro desde CDN..."; \
      curl -L -f -o public/utxo_filter.bin "$FILTER_URL"; \
    else \
      echo "⚠️  FILTER_URL no provista. Generando placeholder para arranque."; \
      touch public/utxo_filter.bin; \
    fi

# 2.4. Copiar Binario
COPY --from=builder /usr/src/app/target/release/orchestrator /usr/local/bin/orchestrator

# 2.5. Configuración
ENV RUST_LOG=info
ENV PORT=3000

EXPOSE 3000

# 2.6. Healthcheck
HEALTHCHECK --interval=30s --timeout=5s --start-period=5s \
  CMD curl -f http://localhost:3000/api/v1/status || exit 1

CMD ["orchestrator"]
