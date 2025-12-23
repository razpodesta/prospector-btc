# ⚡ PROSPECTOR-BTC (V11.5)
### Sistema Distribuido de Auditoría Criptográfica y Arqueología de Entropía en secp256k1

![Estado](https://img.shields.io/badge/Estado-En_Desarrollo_Activo-emerald?style=for-the-badge)
![Core](https://img.shields.io/badge/Core-Rust_U256_Hardened-orange?style=for-the-badge&logo=rust)
![Interface](https://img.shields.io/badge/Interface-Next.js_16-black?style=for-the-badge&logo=nextdotjs)
![Licencia](https://img.shields.io/badge/Licencia-MIT_Academic-blue?style=for-the-badge)

---

## 🌌 1. Resumen Ejecutivo: La Arqueología del Azar

La seguridad de la red Bitcoin descansa sobre una premisa termodinámica: el espacio de búsqueda de una clave privada de 256 bits ($2^{256}$) es tan vasto que su resolución por fuerza bruta requeriría más energía que la emitida por el Sol durante su vida útil.

**Prospector BTC** desafía esta premisa no atacando la matemática de la curva elíptica, sino la **falibilidad de su implementación**. El proyecto opera bajo la hipótesis de la **Entropía Defectuosa**: históricamente, el azar utilizado para generar fortunas digitales ha sido predecible, repetitivo o corrupto debido a fallos de software y limitaciones humanas.

Este sistema actúa como un **Escáner Forense Distribuido** que mapea y audita los "puntos calientes" de baja entropía en el registro inmutable (UTXO Set), utilizando una arquitectura de cómputo efímero de costo cero.

---

## 🛠️ 2. Especificaciones Técnicas y Viabilidad

El éxito teórico de Prospector se basa en la eliminación del cuello de botella computacional tradicional mediante cuatro pilares de ingeniería:

### A. Aritmética Jacobiana Vectorizada (L1)
A diferencia de los métodos convencionales que utilizan Coordenadas Afines, Prospector opera en el **Espacio Proyectivo Jacobiano**. Esto permite realizar adiciones de puntos en la curva sin ejecutar la costosa operación de **Inverso Modular** en el bucle caliente, reduciendo la latencia por llave generada en un factor de 100x.

### B. Protocolo de Cargador Táctico (Montgomery's Trick)
Implementamos la **Inversión por Lotes de Montgomery**. El sistema acumula ráfagas (Magazines) de 256 puntos Jacobianos y realiza una única inversión modular para proyectarlos simultáneamente al plano afín. Esta técnica permite alcanzar velocidades de grado industrial en hardware doméstico/nube.

### C. Estructuras Probabilísticas O(1)
El censo de carteras con saldo (UTXO) se comprime en **Filtros de Bloom Particionados**. Esto permite que cada nodo del enjambre verifique colisiones en tiempo constante $O(1)$ sin realizar consultas a disco o bases de datos externas, manteniendo el material de búsqueda íntegramente en la memoria caché del procesador.

### D. Multi-Estrategia de Auditoría
El sistema no es un "miner" genérico; es un orquestador de motores forenses:
*   **Sequential Engine:** Barrido Jacobiano ultra-rápido de rangos escalares.
*   **Satoshi-XP Engine:** Reconstrucción de estados de memoria de Windows XP (2009-2010).
*   **Android LCG Engine:** Simulación de fallos en el PRNG de Java (CVE-2013-7372).

---

## 📡 3. Arquitectura del Enjambre (Topología Dinámica)

```mermaid
graph TD
    subgraph "NÚCLEO DE MANDO (ESTRATO ESTRATÉGICO)"
        ORCH[Orquestador Rust Axum]
        DBA[(Tactical Ledger - Motor A)]
        DBB[(Strategic Archive - Motor B)]
        ORCH <--> DBA
        ORCH -->|Chronos Sync| DBB
    end

    subgraph "ENJAMBRE HYDRA (ESTRATO TÁCTICO)"
        W1[Worker Node 01]
        W2[Worker Node 02]
        Wn[Worker Node N...]

        W1 -.->|Binary Pulse| ORCH
        W2 -.->|Visual Feed| ORCH
        Wn -.->|Audit Report| ORCH
    end

    subgraph "PUNTO DE ACCESO (ESTRATO VISUAL)"
        DASH[Dashboard Next.js 16]
        DASH <-->|Neural Link| ORCH
    end
    ```

🔬 4. Escenarios de Auditoría Forense
Escenario	Objetivo	Metodología
Satoshi Era	Billeteras 2009-2010	Reconstrucción de PERF_DATA_BLOCK y mezclado SHA-1 de OpenSSL 0.9.8h.
Android LCG	Billeteras 2013	Simulación de java.util.Random con semillas de 48 bits de baja entropía.
Brainwallet Sweep	Entropía Humana	Transformación SHA-256 de diccionarios masivos y patrones mnemotécnicos.
ECDLP Short Range	Claves Públicas Conocidas	Resolución mediante Algoritmo Pollard's Kangaroo para recuperación de escalares.

⚡ 5. Capacidad de Cómputo y Rendimiento
Prospector está diseñado para escalar horizontalmente de forma infinita. Gracias al uso de Rust SIMD (AVX-512) y afinidad de núcleos, el rendimiento proyectado es:
Throughput por Hilo: ~120,000,000 de verificaciones por segundo (Satoshi-XP).
Eficiencia de Enjambre: Reporte de telemetría asíncrono con impacto de performance del 0%.
Huella de Memoria: < 500MB RAM incluyendo el filtro de Bloom de 7.7M de direcciones.

🧰 6. Stack Tecnológico (The Elite Stack)
Core Logic: Rust (no_std ready) para el motor matemático y criptográfico.
Monorepo: Nx para la gestión de dependencias y límites arquitectónicos.
Persistence: Estrategia de Motores Gemelos (Transaccional de alta frecuencia + Archivo histórico).
Real-time: Server-Sent Events (SSE) con empaquetamiento binario MessagePack para el Neural Link.
Security: Bóveda de Identidad Zero-Knowledge con cifrado local AES-256-GCM.

🚀 7. Protocolo de Despliegue
El sistema está diseñado para una Ignición Agnóstica. El binario del minero se compila estáticamente utilizando la librería C MUSL, garantizando portabilidad absoluta hacia cualquier entorno Linux efímero.
Crystallization: Generación del censo binario desde fuentes de Big Data.
Orchestration: Despliegue del núcleo de mando para la gestión de misiones.
Swarm Ignition: Activación de nodos remotos mediante aprovisionamiento automatizado (C2).
Audit Pulse: Monitoreo en tiempo real de colisiones y eficiencia a través del Dashboard.

📜 8. Ética y Cumplimiento
Este proyecto es una herramienta de investigación académica doctoral. Su propósito es demostrar la fragilidad de la generación de entropía defectuosa y certificar la seguridad de los algoritmos criptográficos modernos frente a implementaciones legacy. El uso de este software para acceder a activos digitales de terceros es ilegal y contraviene los principios fundamentales de esta tesis.
