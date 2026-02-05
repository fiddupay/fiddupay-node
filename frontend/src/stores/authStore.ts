import { create } from 'zustand'
import { persist } from 'zustand/middleware'
import { User, LoginCredentials, RegisterData } from '@/types'
import { authAPI, merchantAPI } from '@/services/apiService'

interface AuthState {
  user: User | null
  token: string | null
  isAuthenticated: boolean
  loading: boolean
  error: string | null
}

interface AuthActions {
  login: (credentials: LoginCredentials, rememberMe?: boolean) => Promise<void>
  register: (data: RegisterData) => Promise<void>
  logout: () => void
  clearError: () => void
  loadUser: (silent?: boolean) => Promise<void>
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

      // Actions
      login: async (credentials: LoginCredentials, rememberMe: boolean = false) => {
        try {
          set({ loading: true, error: null })
          const response = await authAPI.login({ ...credentials, remember_me: rememberMe })

          const storage = rememberMe ? localStorage : sessionStorage
          storage.setItem('fiddupay_token', response.data.api_key)

          set({
            user: response.data.user,
            token: response.data.api_key,
            isAuthenticated: true,
            loading: false,
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
        })
      },

      clearError: () => {
        set({ error: null })
      },

      loadUser: async (silent: boolean = false) => {
        const token = localStorage.getItem('fiddupay_token') || sessionStorage.getItem('fiddupay_token')
        if (!token) {
          set({ loading: false })
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
            localStorage.removeItem('fiddupay_token')
            sessionStorage.removeItem('fiddupay_token')
            set({
              user: null,
              token: null,
              isAuthenticated: false,
              loading: false,
            })
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
      partialize: (state) => ({
        user: state.user,
        // We still partialize these for Zustand persistence, 
        // but loadUser and logout handle the manual token storage.
        isAuthenticated: state.isAuthenticated,
      }),
    }
  )
)
