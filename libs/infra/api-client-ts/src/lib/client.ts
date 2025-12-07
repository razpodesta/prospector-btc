// libs/infra/api-client-ts/src/lib/client.ts
import axios from 'axios';

// 1. Configuración dinámica desde el entorno
const BASE_URL = process.env['NEXT_PUBLIC_API_URL'] || 'http://localhost:3000/api/v1';
const API_TOKEN = process.env['NEXT_PUBLIC_API_TOKEN'] || '';

export const apiClient = axios.create({
  baseURL: BASE_URL,
  headers: {
    'Content-Type': 'application/json',
  },
  timeout: 5000,
});

// 2. Interceptor de Seguridad (Inyección de Credenciales)
apiClient.interceptors.request.use((config) => {
  if (API_TOKEN) {
    config.headers.Authorization = `Bearer ${API_TOKEN}`;
  }
  return config;
});

// 3. Interceptor de Errores (Logging)
apiClient.interceptors.response.use(
  (response) => response,
  (error) => {
    // Evitamos ensuciar la consola en caso de cancelación o error esperado
    if (error.code !== 'ERR_CANCELED') {
      console.error('🔥 API Error:', {
        url: error.config?.url,
        status: error.response?.status,
        data: error.response?.data
      });
    }
    return Promise.reject(error);
  }
);
