import axios, { AxiosInstance, AxiosResponse } from 'axios'
import { 
  LoginCredentials, 
  RegisterData, 
  User, 
  Payment, 
  PaymentData, 
  PaymentFilters,
  WalletConfig,
  Wallet,
  Analytics,
  Balance,
  Withdrawal,
  WithdrawalData,
  ApiResponse,
  PaginatedResponse
} from '@/types'

class ApiService {
  private api: AxiosInstance

  constructor() {
    const baseURL = import.meta.env.VITE_API_URL 
      ? `${import.meta.env.VITE_API_URL}/api/v1`
      : '/api/v1'
    
    this.api = axios.create({
      baseURL,
      timeout: 30000,
    })

    // Request interceptor to add auth token
    this.api.interceptors.request.use((config) => {
      const token = localStorage.getItem('payflow_token')
      if (token) {
        config.headers.Authorization = `Bearer ${token}`
      }
      return config
    })

    // Response interceptor for error handling
    this.api.interceptors.response.use(
      (response) => response,
      (error) => {
        if (error.response?.status === 401) {
          localStorage.removeItem('payflow_token')
          window.location.href = '/login'
        }
        return Promise.reject(error)
      }
    )
  }

  // Authentication
  async login(credentials: LoginCredentials): Promise<{ user: User; api_key: string }> {
    const response = await this.api.post('/merchants/login', credentials)
    return response.data
  }

  async register(data: RegisterData): Promise<{ user: User; api_key: string }> {
    const response = await this.api.post('/merchants/register', data)
    return response.data
  }

  async getProfile(): Promise<User> {
    const response = await this.api.get('/merchants/profile')
    return response.data
  }

  // Payments
  async createPayment(data: PaymentData): Promise<Payment> {
    const response = await this.api.post('/payments', data)
    return response.data
  }

  async getPayments(filters?: PaymentFilters): Promise<PaginatedResponse<Payment>> {
    const response = await this.api.get('/payments', { params: filters })
    return response.data
  }

  async getPayment(id: string): Promise<Payment> {
    const response = await this.api.get(`/payments/${id}`)
    return response.data
  }

  // Wallets
  async configureWallet(data: WalletConfig): Promise<ApiResponse<void>> {
    const response = await this.api.put('/merchants/wallets', data)
    return response.data
  }

  async getWallets(): Promise<Wallet[]> {
    const response = await this.api.get('/merchants/wallets')
    return response.data
  }

  // Analytics
  async getAnalytics(timeRange?: { start_date?: string; end_date?: string }): Promise<Analytics> {
    const response = await this.api.get('/merchants/analytics', { params: timeRange })
    return response.data
  }

  async getBalance(): Promise<Balance> {
    const response = await this.api.get('/merchants/balance')
    return response.data
  }

  // Withdrawals
  async createWithdrawal(data: WithdrawalData): Promise<Withdrawal> {
    const response = await this.api.post('/withdrawals', data)
    return response.data
  }

  async getWithdrawals(page = 1, pageSize = 10): Promise<PaginatedResponse<Withdrawal>> {
    const response = await this.api.get('/withdrawals', {
      params: { page, page_size: pageSize }
    })
    return response.data
  }

  async getWithdrawal(id: string): Promise<Withdrawal> {
    const response = await this.api.get(`/withdrawals/${id}`)
    return response.data
  }

  // Settings
  async updateWebhook(url: string): Promise<ApiResponse<void>> {
    const response = await this.api.put('/merchants/webhook', { url })
    return response.data
  }

  async regenerateApiKey(): Promise<{ api_key: string }> {
    const response = await this.api.post('/merchants/api-key/regenerate')
    return response.data
  }

  // Supported currencies
  async getSupportedCurrencies(): Promise<any> {
    const response = await this.api.get('/currencies/supported')
    return response.data
  }

  // Health check
  async healthCheck(): Promise<{ status: string }> {
    const response = await this.api.get('/health')
    return response.data
  }
}

export const apiService = new ApiService()
export default apiService
