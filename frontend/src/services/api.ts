import axios, { AxiosInstance } from 'axios'
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
    this.api.interceptors.request.use((config: any) => {
      const token = localStorage.getItem('fiddupay_token')
      if (token) {
        config.headers.Authorization = `Bearer ${token}`
      }
      return config
    })

    // Response interceptor for error handling
    this.api.interceptors.response.use(
      (response: any) => response,
      (error: any) => {
        if (error.response?.status === 401) {
          localStorage.removeItem('fiddupay_token')
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

  async verifyPayment(id: string): Promise<Payment> {
    const response = await this.api.post(`/payments/${id}/verify`)
    return response.data
  }

  // Wallets
  async configureWallet(data: WalletConfig): Promise<ApiResponse<void>> {
    const response = await this.api.put('/merchants/wallets', data)
    return response.data
  }

  async getWallets(): Promise<Wallet[]> {
    const response = await this.api.get('/wallets')
    return response.data
  }

  async generateWallet(cryptoType: string): Promise<Wallet> {
    const response = await this.api.post('/wallets/generate', { crypto_type: cryptoType })
    return response.data
  }

  async importWallet(cryptoType: string, privateKey: string): Promise<Wallet> {
    const response = await this.api.post('/wallets/import', { 
      crypto_type: cryptoType, 
      private_key: privateKey 
    })
    return response.data
  }

  async exportPrivateKey(cryptoType: string): Promise<{ private_key: string }> {
    const response = await this.api.post('/wallets/export-key', { crypto_type: cryptoType })
    return response.data
  }

  async checkGasRequirements(): Promise<any> {
    const response = await this.api.get('/wallets/gas-check')
    return response.data
  }

  async getGasEstimates(): Promise<any[]> {
    const response = await this.api.get('/wallets/gas-estimates')
    return response.data
  }

  // Analytics
  async getAnalytics(timeRange?: { start_date?: string; end_date?: string }): Promise<Analytics> {
    const response = await this.api.get('/analytics', { params: timeRange })
    return response.data
  }

  async exportAnalytics(format: string, timeRange?: { start_date?: string; end_date?: string }): Promise<any> {
    const response = await this.api.get('/analytics/export', { 
      params: { format, ...timeRange }
    })
    return response.data
  }

  // Fee Settings (placeholder methods for compatibility)
  async getFeeSetting(): Promise<{ fee_percentage: number }> {
    // Return default fee setting since this is now handled by daily volume limits
    return { fee_percentage: 0.75 }
  }

  async updateFeeSetting(_data: { fee_percentage: number }): Promise<void> {
    // Placeholder - fee settings are now managed by admin
    console.log('Fee settings are managed by admin configuration')
  }

  // Address-only payments (placeholder for compatibility)
  async createAddressOnlyPayment(data: any): Promise<Payment> {
    // Use regular payment creation
    return this.createPayment(data)
  }

  async getBalance(): Promise<Balance> {
    const response = await this.api.get('/merchants/balance')
    return response.data
  }

  async getBalanceHistory(params?: any): Promise<PaginatedResponse<any>> {
    const response = await this.api.get('/merchants/balance/history', { params })
    return response.data
  }

  // Withdrawals
  async createWithdrawal(data: WithdrawalData): Promise<Withdrawal> {
    const response = await this.api.post('/withdrawals', data)
    return response.data
  }

  async getWithdrawals(params?: any): Promise<PaginatedResponse<Withdrawal>> {
    const response = await this.api.get('/withdrawals', { params })
    return response.data
  }

  async getWithdrawal(id: string): Promise<Withdrawal> {
    const response = await this.api.get(`/withdrawals/${id}`)
    return response.data
  }

  async cancelWithdrawal(id: string): Promise<Withdrawal> {
    const response = await this.api.post(`/withdrawals/${id}/cancel`)
    return response.data
  }

  // Refunds
  async createRefund(data: any): Promise<any> {
    const response = await this.api.post('/refunds', data)
    return response.data
  }

  async getRefund(id: string): Promise<any> {
    const response = await this.api.get(`/refunds/${id}`)
    return response.data
  }

  async completeRefund(id: string): Promise<any> {
    const response = await this.api.post(`/refunds/${id}/complete`)
    return response.data
  }

  // Security
  async getSecurityEvents(params?: any): Promise<PaginatedResponse<any>> {
    const response = await this.api.get('/security/events', { params })
    return response.data
  }

  async getSecurityAlerts(params?: any): Promise<PaginatedResponse<any>> {
    const response = await this.api.get('/security/alerts', { params })
    return response.data
  }

  async acknowledgeSecurityAlert(id: string): Promise<any> {
    const response = await this.api.post(`/security/alerts/${id}/acknowledge`)
    return response.data
  }

  async getSecuritySettings(): Promise<any> {
    const response = await this.api.get('/security/settings')
    return response.data
  }

  async updateSecuritySettings(data: any): Promise<any> {
    const response = await this.api.put('/security/settings', data)
    return response.data
  }

  // IP Whitelist
  async setIpWhitelist(ipAddresses: string[]): Promise<ApiResponse<void>> {
    const response = await this.api.put('/merchants/ip-whitelist', { ip_addresses: ipAddresses })
    return response.data
  }

  async getIpWhitelist(): Promise<{ ip_addresses: string[] }> {
    const response = await this.api.get('/merchants/ip-whitelist')
    return response.data
  }

  // Audit Logs
  async getAuditLogs(params?: any): Promise<PaginatedResponse<any>> {
    const response = await this.api.get('/audit-logs', { params })
    return response.data
  }

  // Sandbox
  async enableSandbox(): Promise<{ success: boolean }> {
    const response = await this.api.post('/sandbox/enable')
    return response.data
  }

  async simulatePayment(paymentId: string, data: any): Promise<any> {
    const response = await this.api.post(`/sandbox/payments/${paymentId}/simulate`, data)
    return response.data
  }

  // Settings
  async updateWebhook(url: string): Promise<ApiResponse<void>> {
    const response = await this.api.put('/merchants/webhook', { webhook_url: url })
    return response.data
  }

  async generateApiKey(environment?: string): Promise<{ api_key: string }> {
    const response = await this.api.post('/merchants/api-keys/generate', { environment })
    return response.data
  }

  async rotateApiKey(environment?: string): Promise<{ api_key: string }> {
    const response = await this.api.post('/merchants/api-keys/rotate', { environment })
    return response.data
  }

  async switchEnvironment(environment: string): Promise<{ message: string; environment: string }> {
    const response = await this.api.post('/merchants/environment/switch', { environment })
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
