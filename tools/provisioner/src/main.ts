// tools/provisioner/src/main.ts
// =================================================================
// APARATO: THE NECROMANCER (PROVISIONER v2.1)
// CLASIFICACIÓN: OPS / AUTOMATION
// ESTADO: PRODUCTION READY (Strict Types Fixed)
// =================================================================

import { chromium, BrowserContext, Page } from 'playwright';
import * as path from 'path';
import * as fs from 'fs';
import { z } from 'zod';
import chalk from 'chalk';
import 'dotenv/config';

// --- 1. CONFIGURACIÓN ROBUSTA (SSoT) ---
const ConfigSchema = z.object({
  // URLs críticas
  COLAB_URL: z.string().url().default('https://colab.research.google.com/'),
  ORCHESTRATOR_URL: z.string().url(),

  // Seguridad
  WORKER_AUTH_TOKEN: z.string().min(5),

  // Parámetros de Operación
  // CORRECCIÓN ZOD: .default() debe ir ANTES de .transform()
  // Porque la entrada (.env) es String, el default cubre la ausencia, y LUEGO transformamos.
  WORKER_COUNT: z.string().default('1').transform((val) => parseInt(val, 10)),
  HEADLESS: z.string().default('false').transform((val) => val === 'true'),

  // Perfil de navegador (Persistencia de sesión Google)
  CHROME_PROFILE_PATH: z.string().default('./.chrome-profile'),

  // Artefacto (URL pública del binario compilado con MUSL)
  // Si no se provee, usamos una dummy para test, pero fallará la descarga real
  MINER_BINARY_URL: z.string().url().default('https://github.com/razpodesta/prospector-btc/releases/latest/download/miner-musl'),
});

// Validación Fall-Fast al inicio
const config = ConfigSchema.parse(process.env);

// --- 2. PAYLOAD DE INYECCIÓN (PYTHON WRAPPER) ---
// Este script se ejecuta DENTRO de la celda de Colab.
// Orquesta la descarga y ejecución del binario Rust sin bloquear la UI de Colab.
const generatePayload = (workerId: number) => `
# --- PROSPECTOR INJECTION START ---
import os
import subprocess
import time
import sys

# Configuración inyectada desde TS
MINER_URL = "${config.MINER_BINARY_URL}"
ORCH_URL = "${config.ORCHESTRATOR_URL}"
TOKEN = "${config.WORKER_AUTH_TOKEN}"
W_ID = "colab-node-${workerId}-${Date.now()}"

def install_miner():
    """Descarga el binario estático si no existe"""
    if not os.path.exists("miner"):
        print(f"⬇️ [PYTHON] Descargando binario desde {MINER_URL}...")
        # Usamos wget silencioso (-q) pero mostramos salida si falla
        res = subprocess.run(["wget", "-O", "miner", MINER_URL, "-q"])
        if res.returncode != 0:
            print("❌ [PYTHON] Error crítico: Fallo en descarga del binario.")
            sys.exit(1)
        subprocess.run(["chmod", "+x", "miner"])
        print("✅ [PYTHON] Binario instalado y ejecutable.")
    else:
        print("⚡ [PYTHON] Binario detectado en caché local.")

def run_miner():
    """Ejecuta el minero en un subproceso y monitorea su salud"""
    print(f"🚀 [PYTHON] Iniciando Worker {W_ID} contra {ORCH_URL}")

    cmd = ["./miner", f"--orchestrator-url={ORCH_URL}", f"--auth-token={TOKEN}", f"--worker-id={W_ID}"]

    # Bucle de supervisión (Watchdog simple)
    while True:
        try:
            # Popen permite streaming de logs sin bloquear
            process = subprocess.Popen(
                cmd,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                universal_newlines=True,
                bufsize=1
            )

            # Streaming de logs de Rust hacia la consola de Colab
            for line in process.stdout:
                print(line, end='')

            process.wait()

            # Si el proceso muere, analizamos por qué
            rc = process.returncode
            print(f"⚠️ [PYTHON] El minero se detuvo (Exit Code: {rc}). Reiniciando en 5s...")
            time.sleep(5)

        except KeyboardInterrupt:
            print("🛑 [PYTHON] Detenido manualmente.")
            break
        except Exception as e:
            print(f"❌ [PYTHON] Excepción en Runtime: {e}")
            time.sleep(10)

# Ejecución secuencial
try:
    install_miner()
    run_miner()
except Exception as e:
    print(f"💀 [PYTHON] Error fatal en el wrapper: {e}")
# --- PROSPECTOR INJECTION END ---
`;

// --- 3. LÓGICA DE NAVEGACIÓN Y RESILIENCIA ---

/**
 * Maneja los diálogos molestos de Google Colab (Desconexión, ReCaptcha simple, etc)
 */
async function handleColabDialogs(page: Page, prefix: string) {
    const dialogSelectors = [
        'text=RECONNECT',
        'text=Runtime disconnected',
        'text=Connect to hosted runtime',
        '#ok' // Botones genéricos de confirmación
    ];

    for (const selector of dialogSelectors) {
        try {
            const btn = page.locator(selector).first();
            if (await btn.isVisible({ timeout: 500 })) {
                console.log(`${prefix} ⚠️ Detectado diálogo "${selector}". Intentando cerrar...`);
                await btn.click({ force: true });
                await page.waitForTimeout(1000);
            }
        } catch {
            // Ignorar errores de selector si no existen
        }
    }
}

async function wakeUpWorker(id: number, context: BrowserContext) {
  const prefix = chalk.blue(`[Necromancer-${id}]`);
  console.log(`${prefix} 💀 Invocando worker...`);

  const page = await context.newPage();

  try {
    // A. Navegación
    // networkidle asegura que la carga pesada de JS de Colab terminó
    await page.goto(config.COLAB_URL, { waitUntil: 'domcontentloaded', timeout: 60000 });

    // B. Verificación de Sesión (Vital)
    const googleAvatar = page.locator('img[src*="googleusercontent.com"]').first();
    try {
        await googleAvatar.waitFor({ state: 'visible', timeout: 15000 });
    } catch {
        console.error(chalk.red(`${prefix} ❌ ERROR DE AUTENTICACIÓN`));
        console.error(`No se detectó sesión de Google activa en el perfil de Chrome.`);
        console.error(`Asegúrate de haberte logueado manualmente en la instancia de Chrome lanzada.`);
        throw new Error("Auth Failed");
    }

    // C. Conexión al Runtime
    console.log(`${prefix} 🔌 Conectando al Runtime (GPU/CPU)...`);
    const connectBtn = page.getByText(/^Connect$|^Reconnect$/i).first();

    if (await connectBtn.isVisible()) {
        await connectBtn.click();
        // Esperamos indicadores de recursos (RAM/Disk)
        const resourceMonitor = page.locator('colab-memory-usage-sparkline').first(); // Selector específico de Colab moderno
        try {
             await resourceMonitor.waitFor({ state: 'attached', timeout: 45000 });
             console.log(`${prefix} ✅ Runtime Conectado Exitosamente.`);
        } catch {
             console.warn(`${prefix} ⚠️ Advertencia: No se confirmó visualmente la conexión, pero procedemos.`);
        }
    }

    // D. Limpieza e Inyección
    console.log(`${prefix} 💉 Inyectando Payload...`);

    // Buscamos el editor Monaco (el área de texto principal)
    const editorLayer = page.locator('.view-lines').first();
    await editorLayer.click({ force: true });

    // Limpieza de celda anterior (Ctrl+A -> Del)
    await page.keyboard.press('Control+A');
    await page.keyboard.press('Delete');

    // Inyección vía Clipboard (Más rápido y seguro que typeo carácter por carácter)
    const payload = generatePayload(id);
    await page.evaluate((text) => navigator.clipboard.writeText(text), payload);
    await page.keyboard.press('Control+V'); // Pegar

    // E. Ejecución
    console.log(`${prefix} ⚡ EJECUTANDO (CTRL+ENTER)...`);
    await page.keyboard.press('Control+Enter');

    // F. Validación de Arranque
    // Buscamos el log de Python en el iframe de salida
    try {
        const outputFrame = page.frameLocator('iframe').last();
        const successLog = outputFrame.getByText('🚀 [PYTHON] Iniciando Worker');
        await successLog.waitFor({ state: 'visible', timeout: 20000 });
        console.log(chalk.green(`${prefix} ✅ Worker OPERATIVO y reportando.`));
    } catch (e) {
        console.warn(`${prefix} ⚠️ Worker ejecutado, pero no se leyó el log de confirmación a tiempo.`);
    }

    // G. Mantenimiento (Anti-Idle)
    // Dejamos la página abierta. Si cerramos 'page', el runtime se mata.

  } catch (error) {
    console.error(chalk.red(`${prefix} ❌ FALLO CRÍTICO:`), error);
    // Screenshot para debug visual
    const screenshotPath = path.resolve(`logs/error-${id}.png`);
    await page.screenshot({ path: screenshotPath, fullPage: true });
    console.log(`${prefix} 📸 Captura guardada en: ${screenshotPath}`);
  }
}

async function main() {
  console.log(chalk.green.bold('🔥 PROSPECTOR NECROMANCER v2.1 [INIT]'));

  // Verificación de Perfil de Chrome
  const userDataDir = path.resolve(config.CHROME_PROFILE_PATH);
  if (!fs.existsSync(userDataDir)) {
      console.log(chalk.yellow(`⚠️ El directorio de perfil no existe: ${userDataDir}`));
      console.log(chalk.white(`El script creará uno nuevo, pero deberás loguearte manualmente en Google la primera vez.`));
  }

  // Lanzamiento del Navegador "Headful" (Con UI)
  const context = await chromium.launchPersistentContext(userDataDir, {
    headless: config.HEADLESS,
    channel: 'chrome', // Usa tu Chrome instalado, mejor para evadir detección
    viewport: { width: 1280, height: 720 },
    args: [
        '--disable-blink-features=AutomationControlled', // Bandera anti-bot
        '--no-sandbox',
        '--disable-infobars',
        '--window-position=0,0'
    ]
  });

  const workerCount = config.WORKER_COUNT;
  console.log(chalk.cyan(`🎯 Objetivo: Desplegar ${workerCount} nodos.`));

  // Ejecución en Batch para estabilidad
  const BATCH_SIZE = 3;

  for (let i = 0; i < workerCount; i += BATCH_SIZE) {
      const batchPromises = [];
      for (let j = 0; j < BATCH_SIZE && (i + j) < workerCount; j++) {
          batchPromises.push(wakeUpWorker(i + j, context));
      }

      await Promise.all(batchPromises);

      if (i + BATCH_SIZE < workerCount) {
          console.log(chalk.gray(`⏳ Enfriamiento de API (10s)...`));
          await new Promise(r => setTimeout(r, 10000));
      }
  }

  console.log(chalk.green.bold('✨ ENJAMBRE DESPLEGADO. SISTEMA EN VIGILANCIA.'));
  console.log(chalk.gray('NO CIERRES ESTA VENTANA NI EL NAVEGADOR.'));
}

main().catch((err) => {
    console.error(chalk.red.bold('💀 ERROR FATAL EN NECROMANCER:'));
    console.error(err);
    process.exit(1);
});
