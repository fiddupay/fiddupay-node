// API Service - Centralized API calls
import api from '@/utils/api'

export const authAPI = {
  register: (data: { business_name: string; email: string; password: string }) =>
    api.post('/api/v1/merchants/register', data),
  
  login: (data: { email: string; password: string }) =>
    api.post('/api/v1/merchants/login', data),
}

export const merchantAPI = {
  getProfile: () => api.get('/api/v1/merchants/profile'),
  getBalance: () => api.get('/api/v1/merchants/balance'),
  getAnalytics: () => api.get('/api/v1/merchants/analytics'),
  getFeeSetting: () => api.get('/api/v1/merchants/fee-setting'),
  updateFeeSetting: (data: any) => api.put('/api/v1/merchants/fee-setting', data),
  switchEnvironment: (environment: string) => 
    api.post('/api/v1/merchants/environment/switch', { environment }),
  generateApiKey: () => api.post('/api/v1/merchants/api-keys/generate'),
  rotateApiKey: () => api.post('/api/v1/merchants/api-keys/rotate'),
  setWallet: (data: any) => api.put('/api/v1/merchants/wallets', data),
  setWebhook: (data: any) => api.put('/api/v1/merchants/webhook', data),
}

export const paymentAPI = {
  create: (data: any) => api.post('/api/v1/payments', data),
  getStatus: (paymentId: string) => api.get(`/api/v1/payments/${paymentId}/status`),
  getHistory: (params?: any) => api.get('/api/v1/payments', { params }),
}

export const withdrawalAPI = {
  create: (data: any) => api.post('/api/v1/withdrawals', data),
  process: (id: string, password: string) => api.post(`/api/v1/withdrawals/${id}/process`, { encryption_password: password }),
  getHistory: (params?: any) => api.get('/api/v1/withdrawals', { params }),
  validateGas: (cryptoType: string, amount: number) => api.get(`/api/v1/wallets/gas-check?crypto_type=${cryptoType}&amount=${amount}`),
}

export const walletAPI = {
  configure: (data: any) => api.post('/api/v1/wallets/configure-address', data),
  generate: (network: string, password?: string) => api.post('/api/v1/wallets/generate', { network, encryption_password: password }),
  import: (data: any) => api.post('/api/v1/wallets/import', data),
  getAll: () => api.get('/api/v1/wallets'),
}

export const securityAPI = {
  getEvents: (params?: any) => api.get('/api/v1/security/events', { params }),
  getAlerts: (params?: any) => api.get('/api/v1/security/alerts', { params }),
  getBalanceAlerts: (params?: any) => api.get('/api/v1/security/balance-alerts', { params }),
  checkGas: () => api.get('/api/v1/security/gas-check'),
  acknowledgeAlert: (alertId: string) => 
    api.post(`/api/v1/security/alerts/${alertId}/acknowledge`),
  resolveBalanceAlert: (alertId: string) => 
    api.post(`/api/v1/security/balance-alerts/${alertId}/resolve`),
}

export const publicAPI = {
  contact: (data: any) => api.post('/api/v1/contact', data),
  getSupportedCurrencies: () => api.get('/api/v1/currencies/supported'),
  getStatus: () => api.get('/api/status'),
  getPricing: () => api.get('/api/v1/pricing'),
}

export default {
  auth: authAPI,
  merchant: merchantAPI,
  payment: paymentAPI,
  withdrawal: withdrawalAPI,
  wallet: walletAPI,
  security: securityAPI,
  public: publicAPI,
}
