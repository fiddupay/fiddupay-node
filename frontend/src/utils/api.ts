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
      const isAuthPage = window.location.pathname === '/login' || window.location.pathname === '/register';

      if (!isAuthPage && !suppressAuthRedirect) {
        // Clear auth tokens from both storages

        localStorage.removeItem('fiddupay_dashboard_token')
        sessionStorage.removeItem('fiddupay_dashboard_token')

        // Clear the Zustand auth state
        localStorage.removeItem('fiddupay-auth')
        sessionStorage.removeItem('fiddupay-auth')

        // Redirect to login
        window.location.href = '/login'
      }
    }
    return Promise.reject(error)
  }
)

export default api
