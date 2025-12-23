/**
 * =================================================================
 * APARATO: SOVEREIGN EDGE PROXY (V18.0 - NEXT.JS 16 COMPLIANT)
 * CLASIFICACIÓN: INFRASTRUCTURE GATEWAY (ESTRATO L4)
 * RESPONSABILIDAD: ORQUESTACIÓN DE PETICIONES Y FRONTERA DE RED
 *
 * VISION HIPER-HOLÍSTICA:
 * Implementa el interceptor de peticiones de alta velocidad.
 * Actúa como un proxy inverso ligero para manejar la localización (i18n)
 * y la redirección de tráfico no identificado, protegiendo al motor
 * de renderizado de ráfagas no autorizadas.
 * =================================================================
 */

import { NextRequest, NextResponse } from "next/server";
import { authHandler } from "@/lib/handlers/auth";
import { i18nHandler } from "@/lib/handlers/i18n";
import { visitorHandler } from "@/lib/handlers/visitor";

/**
 * Función Proxy Soberana.
 * Ejecuta el ciclo de vida de la petición antes de la resolución de rutas.
 */
export async function proxy(request: NextRequest): Promise<NextResponse> {
  const { pathname } = request.nextUrl;

  // 1. PROTOCOLO DE EXCLUSIÓN (Static Assets Bypass)
  if (
    pathname.startsWith("/api/") ||
    pathname.startsWith("/_next") ||
    pathname.includes(".")
  ) {
    return NextResponse.next();
  }

  // 2. EXTRACCIÓN DE CONTEXTO DE VISITANTE (L3 Telemetry)
  await visitorHandler(request);

  // 3. HANDSHAKE DE AUTORIZACIÓN (Optimistic Check)
  // Nota: La validación real ocurre en el DAL de cada página/acción.
  const authentication_response = await authHandler(request);
  if (authentication_response) {
    return authentication_response;
  }

  // 4. NORMALIZACIÓN DE IDIOMA Y ENRUTAMIENTO FINAL
  return i18nHandler(request);
}

/**
 * Configuración del Matcher de Frontera.
 */
export const config = {
  matcher: [
    "/((?!api|_next/static|_next/image|favicon.ico|robots.txt|sitemap.xml|.*\\..*).*)",
  ],
};
