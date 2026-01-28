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
  login: (credentials: LoginCredentials) => Promise<void>
  register: (data: RegisterData) => Promise<void>
  logout: () => void
  clearError: () => void
  loadUser: () => Promise<void>
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
      login: async (credentials: LoginCredentials) => {
        try {
          set({ loading: true, error: null })
          const response = await authAPI.login(credentials)
          
          localStorage.setItem('fiddupay_token', response.data.api_key)
          
          set({
            user: response.data.user,
            token: response.data.api_key,
            isAuthenticated: true,
            loading: false,
          })
        } catch (error: any) {
          set({
            error: error.response?.data?.error || 'Login failed',
            loading: false,
          })
          throw error
        }
      },

      register: async (data: RegisterData) => {
        try {
          set({ loading: true, error: null })
          const response = await authAPI.register(data)
          
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

      loadUser: async () => {
        const token = localStorage.getItem('fiddupay_token')
        if (!token) return

        try {
          set({ loading: true })
          const user = await merchantAPI.getProfile()
          set({
            user: user.data,
            token,
            isAuthenticated: true,
            loading: false,
          })
        } catch (error) {
          localStorage.removeItem('fiddupay_token')
          set({
            user: null,
            token: null,
            isAuthenticated: false,
            loading: false,
          })
        }
      },
    }),
    {
      name: 'fiddupay-auth',
      partialize: (state) => ({
        user: state.user,
        token: state.token,
        isAuthenticated: state.isAuthenticated,
      }),
    }
  )
)
