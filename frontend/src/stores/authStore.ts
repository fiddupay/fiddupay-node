import { create } from 'zustand'
import { persist, createJSONStorage, StateStorage } from 'zustand/middleware'
import { User, LoginCredentials, RegisterData } from '@/types'
import { authAPI, merchantAPI } from '@/services/apiService'

interface AuthState {
  user: User | null
  token: string | null
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
// This ensures that the Zustand state (user, isAuthenticated) is stored 
// in the same place as the token.
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

          // Clear both storages before setting the new token to avoid conflicts
          localStorage.removeItem('fiddupay_token')
          sessionStorage.removeItem('fiddupay_token')

          const storage = rememberMe ? localStorage : sessionStorage
          storage.setItem('fiddupay_token', response.data.api_key)

          set({
            user: response.data.user,
            token: response.data.api_key,
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

          // Registration always defaults to persistent for convenience, 
          // but we follow current pattern of using localStorage.
          localStorage.setItem('fiddupay_token', response.data.api_key)

          set({
            user: response.data.user,
            token: response.data.api_key,
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
        localStorage.removeItem('fiddupay_token')
        sessionStorage.removeItem('fiddupay_token')
        set({
          user: null,
          token: null,
          isAuthenticated: false,
          error: null,
          rememberMe: false
        })
      },

      clearError: () => {
        set({ error: null })
      },

      loadUser: async (silent: boolean = false) => {
        const token = localStorage.getItem('fiddupay_token') || sessionStorage.getItem('fiddupay_token')
        if (!token) {
          set({
            user: null,
            token: null,
            isAuthenticated: false,
            loading: false
          })
          return
        }

        try {
          if (!silent) set({ loading: true })
          const response = await merchantAPI.getProfile()
          const profileUser = response.data.user
          const newApiToken = profileUser.api_key || token

          // Update storage with the latest token from profile if it changed
          if (profileUser.api_key && profileUser.api_key !== token) {
            if (localStorage.getItem('fiddupay_token')) {
              localStorage.setItem('fiddupay_token', profileUser.api_key)
            }
            if (sessionStorage.getItem('fiddupay_token')) {
              sessionStorage.setItem('fiddupay_token', profileUser.api_key)
            }
          }

          set({
            user: profileUser,
            token: newApiToken,
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
