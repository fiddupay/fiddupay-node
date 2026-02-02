// API Service - Centralized API calls
import api from '@/utils/api'

export const authAPI = {
  register: (data: { business_name: string; email: string; password: string }) =>
    api.post('/api/v1/merchant/register', data),
  
  login: (data: { email: string; password: string }) =>
    api.post('/api/v1/merchant/login', data),
}

export const merchantAPI = {
  getProfile: () => api.get('/api/v1/merchant/profile'),
  getBalance: () => api.get('/api/v1/merchant/balance'),
  getAnalytics: (params?: { 
    granularity?: string; 
    start_date?: string; 
    end_date?: string; 
  }) => {
    const query = params ? `?${new URLSearchParams(params as any).toString()}` : '';
    return api.get(`/api/v1/merchant/analytics${query}`);
  },

  // Invoice Management
  createInvoice: (data: { amount_usd: string; description: string; due_date: string }) => 
    api.post('/api/v1/merchant/invoices', data),
  
  getInvoices: (params?: { limit?: number; offset?: number }) => {
    const query = params ? `?${new URLSearchParams(params as any).toString()}` : '';
    return api.get(`/api/v1/merchant/invoices${query}`);
  },
  
  getInvoice: (invoiceId: string) => api.get(`/api/v1/merchant/invoices/${invoiceId}`),
  getFeeSetting: () => api.get('/api/v1/merchant/fee-setting'),
  updateFeeSetting: (data: any) => api.put('/api/v1/merchant/fee-setting', data),
  switchEnvironment: (environment: string) => 
    api.post('/api/v1/merchant/environment/switch', { environment }),
  generateApiKey: () => api.post('/api/v1/merchant/api-keys/generate'),
  rotateApiKey: () => api.post('/api/v1/merchant/api-keys/rotate'),
  setWallet: (data: any) => api.put('/api/v1/merchant/wallets', data),
  setWebhook: (data: any) => api.put('/api/v1/merchant/webhook', data),
}

export const paymentAPI = {
  create: (data: any) => api.post('/api/v1/merchant/payments', data),
  getStatus: (paymentId: string) => api.get(`/api/v1/merchant/payments/${paymentId}/status`),
  getHistory: (params?: {
    status?: string;
    crypto_type?: string;
    blockchain?: string;
    start_date?: string;
    end_date?: string;
    min_amount?: number;
    max_amount?: number;
    limit?: number;
    offset?: number;
  }) => {
    const query = params ? `?${new URLSearchParams(params as any).toString()}` : '';
    return api.get(`/api/v1/merchant/payments${query}`);
  },
  get: (paymentId: string) => api.get(`/api/v1/merchant/payments/${paymentId}`),
  verify: (paymentId: string, data: any) => api.post(`/api/v1/merchant/payments/${paymentId}/verify`, data),
}

export const withdrawalAPI = {
  create: (data: any) => api.post('/api/v1/merchant/withdrawals', data),
  process: (id: string, password: string) => api.post(`/api/v1/merchant/withdrawals/${id}/process`, { encryption_password: password }),
  getHistory: (params?: any) => api.get('/api/v1/merchant/withdrawals', { params }),
  validateGas: (cryptoType: string, amount: number) => api.get(`/api/v1/merchant/wallets/gas-check?crypto_type=${cryptoType}&amount=${amount}`),
}

export const walletAPI = {
  configure: (data: any) => api.post('/api/v1/merchant/wallets/configure-address', data),
  generate: (network: string, password?: string) => api.post('/api/v1/merchant/wallets/generate', { network, encryption_password: password }),
  import: (data: any) => api.post('/api/v1/merchant/wallets/import', data),
  getAll: () => api.get('/api/v1/merchant/wallets'),
}

export const securityAPI = {
  getEvents: (params?: any) => api.get('/api/v1/merchant/security/events', { params }),
  getAlerts: (params?: any) => api.get('/api/v1/merchant/security/alerts', { params }),
  getBalanceAlerts: (params?: any) => api.get('/api/v1/merchant/security/balance-alerts', { params }),
  checkGas: () => api.get('/api/v1/merchant/security/gas-check'),
  acknowledgeAlert: (alertId: string) => 
    api.post(`/api/v1/merchant/security/alerts/${alertId}/acknowledge`),
  resolveBalanceAlert: (alertId: string) => 
    api.post(`/api/v1/merchant/security/balance-alerts/${alertId}/resolve`),
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
