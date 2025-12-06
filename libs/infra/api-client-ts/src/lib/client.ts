import axios from 'axios';

// En producción, esto vendría de process.env.NEXT_PUBLIC_API_URL
// Por defecto apuntamos al Orquestador local
const BASE_URL = 'http://localhost:3000/api/v1';

export const apiClient = axios.create({
  baseURL: BASE_URL,
  headers: {
    'Content-Type': 'application/json',
  },
  timeout: 5000, // Fail fast
});

// Interceptor para manejo de errores global
apiClient.interceptors.response.use(
  (response) => response,
  (error) => {
    console.error('🔥 API Error:', error.response?.data || error.message);
    return Promise.reject(error);
  }
);
