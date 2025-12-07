// libs/infra/api-client-ts/src/lib/client.ts
import axios from 'axios';

// ESTRATEGIA RELATIVA:
// Al usar el proxy de Next.js, no necesitamos saber la URL del backend en el cliente.
// Simplemente llamamos a nuestro propio dominio.
const BASE_URL = '/api/v1';

// Token opcional (si se inyecta desde el servidor o env)
const API_TOKEN = process.env['NEXT_PUBLIC_API_TOKEN'] || '';

export const apiClient = axios.create({
  baseURL: BASE_URL,
  headers: {
    'Content-Type': 'application/json',
  },
  timeout: 10000, // 10s para dar tiempo al proxy
});

// Interceptor de Seguridad
apiClient.interceptors.request.use((config) => {
  if (API_TOKEN) {
    config.headers.Authorization = `Bearer ${API_TOKEN}`;
  }
  return config;
});

// Interceptor de Errores
apiClient.interceptors.response.use(
  (response) => response,
  (error) => {
    if (error.code !== 'ERR_CANCELED') {
      // Log discreto para no saturar consola
      console.warn('API Warning:', {
        endpoint: error.config?.url,
        status: error.response?.status
      });
    }
    return Promise.reject(error);
  }
);
