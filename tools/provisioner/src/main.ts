// tools/provisioner/src/main.ts
// =================================================================
// APARATO: THE NECROMANCER (PROVISIONER)
// RESPONSABILIDAD: GESTIÓN DE FLOTA EN GOOGLE COLAB
// =================================================================

import { chromium, BrowserContext, Page } from 'playwright';
import * as path from 'path';
import * as fs from 'fs';
import { z } from 'zod';
import chalk from 'chalk';
import 'dotenv/config';

// 1. VALIDACIÓN DE ENTORNO (FAIL FAST)
const ConfigSchema = z.object({
  COLAB_URL: z.string().url(),
  ORCHESTRATOR_URL: z.string().url(),
  WORKER_AUTH_TOKEN: z.string().min(10),
  WORKER_COUNT: z.string().transform(Number).default('1'),
  HEADLESS: z.string().transform((val) => val === 'true').default('false'),
  CHROME_PROFILE_PATH: z.string().default('./.chrome-profile'),
});

const config = ConfigSchema.parse(process.env);

// 2. GENERADOR DE PAYLOAD (INYECCIÓN DE CÓDIGO)
// Este script bash se escribirá en la celda de Colab y se ejecutará.
const generatePayload = (workerId: number) => `
# --- PROSPECTOR INJECTION START ---
import os
import subprocess
import time

# Configuración
MINER_URL = "https://github.com/TU_USUARIO/prospector-btc/releases/download/v0.1-alpha/miner-musl"
ORCH_URL = "${config.ORCHESTRATOR_URL}"
TOKEN = "${config.WORKER_AUTH_TOKEN}"
W_ID = "colab-worker-${workerId}-${Date.now()}"

print(f"🚀 INICIANDO PROTOCOLO PROSPECTOR: {W_ID}")

# 1. Descargar Binario Estático (Si no existe)
if not os.path.exists("miner"):
    print("⬇️ Descargando binario minero...")
    subprocess.run(["wget", "-O", "miner", MINER_URL, "-q"])
    subprocess.run(["chmod", "+x", "miner"])

# 2. Ejecutar Minero (Bucle Infinito gestionado por Rust)
print(f"🔗 Conectando a {ORCH_URL}...")
cmd = f"./miner --orchestrator-url={ORCH_URL} --auth-token={TOKEN} --worker-id={W_ID}"
os.system(cmd)
# --- PROSPECTOR INJECTION END ---
`;

async function wakeUpWorker(id: number, context: BrowserContext) {
  const prefix = chalk.blue(`[Necromancer-${id}]`);
  console.log(`${prefix} 💀 Invocando worker...`);

  const page = await context.newPage();

  try {
    // A. Navegación
    await page.goto(config.COLAB_URL, { waitUntil: 'domcontentloaded' });

    // B. Verificación de Sesión
    // Esperamos un elemento típico de Google logueado (Avatar)
    try {
        await page.waitForSelector('img[src*="googleusercontent.com"]', { timeout: 5000 });
        console.log(`${prefix} ✅ Sesión de Google detectada.`);
    } catch {
        console.error(`${prefix} ❌ ERROR: No parece haber sesión iniciada. Revisa tu perfil de Chrome.`);
        throw new Error("No session");
    }

    // C. Conexión al Runtime
    console.log(`${prefix} 🔌 Conectando al Runtime T4...`);
    // Truco: Hacemos click en "Connect" si existe, o "Reconnect"
    const connectBtn = page.getByText(/Connect|Reconnect/i).first();
    if (await connectBtn.isVisible()) {
        await connectBtn.click();
        await page.waitForTimeout(3000); // Esperar asignación
    }

    // D. Inyección de Código
    // Buscamos la primera celda de código (Monaco Editor)
    console.log(`${prefix} 💉 Inyectando Payload...`);
    await page.locator('.monaco-editor').first().click();

    // Limpiamos celda (CTRL+A, Backspace)
    await page.keyboard.press('Control+A');
    await page.keyboard.press('Backspace');

    // Escribimos el script Python
    const payload = generatePayload(id);
    await page.keyboard.insertText(payload);

    // E. Ejecución
    console.log(`${prefix} ⚡ EJECUTANDO (CTRL+Enter)...`);
    await page.keyboard.press('Control+Enter');

    console.log(`${prefix} ✅ Worker desplegado. Monitoreando logs por 10s...`);
    await page.waitForTimeout(10000);

    // Opcional: Tomar screenshot de éxito
    await page.screenshot({ path: `logs/success-${id}.png` });

  } catch (error) {
    console.error(`${prefix} ❌ FALLO CRÍTICO:`, error);
    await page.screenshot({ path: `logs/error-${id}.png` });
  }
}

async function main() {
  console.log(chalk.green.bold('🔥 PROSPECTOR PROVISIONER INICIADO'));
  console.log(`🎯 Objetivo: ${config.WORKER_COUNT} nodos en ${config.COLAB_URL}`);

  // Verificar perfil de Chrome
  const userDataDir = path.resolve(config.CHROME_PROFILE_PATH);
  if (!fs.existsSync(userDataDir)) {
      console.log(chalk.red('⚠️ ALERTA: No existe el perfil de Chrome.'));
      console.log('   Ejecuta chrome con: --user-data-dir="./.chrome-profile" y loguéate en Google primero.');
      process.exit(1);
  }

  const context = await chromium.launchPersistentContext(userDataDir, {
    headless: config.HEADLESS,
    channel: 'chrome',
    args: ['--disable-blink-features=AutomationControlled'],
    viewport: { width: 1280, height: 720 }
  });

  // Ejecución en serie (para no saturar RAM local)
  const count = Number(config.WORKER_COUNT);
  for (let i = 0; i < count; i++) {
    await wakeUpWorker(i, context);
    await new Promise(r => setTimeout(r, 5000)); // Delay táctico
  }

  console.log(chalk.green('✨ Despliegue completado. Manteniendo navegador abierto...'));
  // No cerramos el navegador, o los workers de Colab morirán.
}

main().catch(console.error);
