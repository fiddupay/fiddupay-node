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
  headers: {
    'Content-Type': 'application/json',
  },
})

// Request interceptor to add auth token
api.interceptors.request.use(
  (config) => {
    const token = localStorage.getItem('fiddupay_token') || sessionStorage.getItem('fiddupay_token')
    if (token) {
      config.headers.Authorization = `Bearer ${token}`
    }
    return config
  },
  (error) => {
    return Promise.reject(error)
  }
)

// Response interceptor for error handling
api.interceptors.response.use(
  (response) => response,
  (error) => {
    if (error.response?.status === 401) {
      const isAuthPage = window.location.pathname === '/login' || window.location.pathname === '/register';

      if (!isAuthPage && !suppressAuthRedirect) {
        // Clear auth tokens from both storages
        localStorage.removeItem('fiddupay_token')
        sessionStorage.removeItem('fiddupay_token')

        // Clear the Zustand auth state from both storages
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
