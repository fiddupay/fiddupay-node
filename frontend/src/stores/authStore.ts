import { create } from 'zustand'
import { persist, createJSONStorage, StateStorage } from 'zustand/middleware'
import { User, LoginCredentials, RegisterData } from '@/types'
import { authAPI, merchantAPI } from '@/services/apiService'

interface AuthState {
  user: User | null
  token: string | null
  dashboard_token: string | null // NEW: Persistent session token
  isAuthenticated: boolean
  loading: boolean
  error: string | null
  rememberMe: boolean
}

interface AuthActions {
  login: (credentials: LoginCredentials) => Promise<void>
  register: (data: RegisterData) => Promise<void>
  logout: () => void
  clearError: () => void
  loadUser: (silent?: boolean) => Promise<void>
}

// Custom storage to handle "Remember Me" logic
const customStateStorage: StateStorage = {
  getItem: (name: string): string | null => {
    return localStorage.getItem(name) || sessionStorage.getItem(name)
  },
  setItem: (name: string, value: string): void => {
    try {
      const parsed = JSON.parse(value)
      // If rememberMe is true, store in localStorage, otherwise sessionStorage
      if (parsed.state?.rememberMe) {
        localStorage.setItem(name, value)
        sessionStorage.removeItem(name)
      } else {
        sessionStorage.setItem(name, value)
        localStorage.removeItem(name)
      }
    } catch (error) {
      // Fallback to localStorage if parsing fails
      localStorage.setItem(name, value)
    }
  },
  removeItem: (name: string): void => {
    localStorage.removeItem(name)
    sessionStorage.removeItem(name)
  }
}

export const useAuthStore = create<AuthState & AuthActions>()(
  persist(
    (set, _get) => ({
      // State
      user: null,
      token: null,
      dashboard_token: null,
      isAuthenticated: false,
      loading: false,
      error: null,
      rememberMe: false,

      // Actions
      login: async (credentials: LoginCredentials) => {
        try {
          const rememberMe = !!credentials.remember_me
          set({ loading: true, error: null })
          const response = await authAPI.login(credentials)

          // Dashboard token is now handled via HttpOnly cookies (Fortress Layer)
          // We no longer store it in localStorage/sessionStorage

          set({
            user: response.data.user,
            token: response.data.dashboard_token, // JWT is our primary session token
            dashboard_token: response.data.dashboard_token,
            isAuthenticated: true,
            loading: false,
            rememberMe: rememberMe
          })
        } catch (error: any) {
          set({
            error: error.response?.data?.error?.message || error.response?.data?.message || error.response?.data?.error || 'Login failed',
            loading: false,
          })
          throw error
        }
      },

      register: async (data: RegisterData) => {
        try {
          set({ loading: true, error: null })
          const response = await authAPI.register(data)

          // DASHBOARD_TOKEN handled via HttpOnly Cookies

          set({
            user: response.data.user,
            token: response.data.dashboard_token, // JWT is our primary session token
            dashboard_token: response.data.dashboard_token,
            isAuthenticated: true,
            loading: false,
            rememberMe: true // Default to true for registration
          })
        } catch (error: any) {
          set({
            error: error.response?.data?.error || 'Registration failed',
            loading: false,
          })
          throw error
        }
      },

      logout: () => {
        // Hit backend logout to clear HttpOnly cookies
        authAPI.logout().catch((err: any) => console.error("Logout failed:", err));

        set({
          user: null,
          token: null,
          dashboard_token: null,
          isAuthenticated: false,
          error: null,
          rememberMe: false
        })
      },

      clearError: () => {
        set({ error: null })
      },

      loadUser: async (silent: boolean = false) => {
        // Since HttpOnly cookies cannot be read by JS, we assume auth exists if state.isAuthenticated is true.
        // We'll verify this by attempting to fetch the profile.

        try {
          if (!silent) set({ loading: true })
          const response = await merchantAPI.getProfile()
          const profileUser = response.data.user

          set({
            user: profileUser,
            isAuthenticated: true,
            loading: false,
          })
        } catch (error: any) {
          // Only log out if it's an authentication error (401)
          if (error.response && error.response.status === 401) {
            _get().logout()
          } else {
            // For other errors (like 500), keep the session but stop loading
            set({ loading: false })
            console.error('Failed to load user profile:', error)
          }
        }
      },
    }),
    {
      name: 'fiddupay-auth',
      storage: createJSONStorage(() => customStateStorage),
      partialize: (state) => ({
        user: state.user,
        isAuthenticated: state.isAuthenticated,
        rememberMe: state.rememberMe
      } as any),
    }
  )
)
