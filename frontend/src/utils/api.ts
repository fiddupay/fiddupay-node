import { useAuthStore } from '@/stores/authStore'
import axios from 'axios'

// Flag to temporarily suppress the 401 interceptor during environment switches
export let suppressAuthRedirect = false
export function setSuppressAuthRedirect(value: boolean) {
  suppressAuthRedirect = value
}

// Create axios instance with base configuration
const api = axios.create({
  baseURL: import.meta.env.VITE_API_URL || 'https://api.fiddupay.com',
  timeout: 30000,
  withCredentials: true, // Required for HttpOnly cookies (Fortress Layer)
  headers: {
    'Content-Type': 'application/json',
  },
})

// Request interceptor: The browser will automatically include HttpOnly cookies, 
// but we also include the Authorization header as a fallback (Hybrid Auth).
api.interceptors.request.use(
  (config) => {
    // Check sessionStorage for the fallback token
    const dashboardToken = sessionStorage.getItem('fiddupay_dashboard_token');

    if (dashboardToken) {
      config.headers.Authorization = `Bearer ${dashboardToken}`;
    }

    return config;
  },
  (error) => Promise.reject(error)
);

// Response interceptor for error handling
api.interceptors.response.use(
  (response) => response,
  (error) => {
    if (error.response?.status === 401) {
      // Only redirect to login if we are in the protected dashboard area
      const isAppPath = window.location.pathname.startsWith('/app');
      const isAuthPage = window.location.pathname === '/login' || window.location.pathname === '/register';

      if (isAppPath && !isAuthPage && !suppressAuthRedirect) {
        // Use the auth store's logout to clear state cleanly across the app
        useAuthStore.getState().logout();

        // Redirect to login only if accessing a protected route
        window.location.href = '/login'
      } else if (!isAuthPage) {
        // Silently clear state on public pages to avoid infinite redirect loops
        // and allow viewing the landing page.
        useAuthStore.getState().logout();
      }
    }
    return Promise.reject(error)
  }
)

export default api
