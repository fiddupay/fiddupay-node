// API Service - Centralized API calls
import api from '@/utils/api'
import { LoginCredentials } from '@/types'

export const authAPI = {
  register: (data: { business_name: string; email: string; password: string }) =>
    api.post('/api/v1/merchants/register', data),

  login: (data: LoginCredentials) =>
    api.post('/api/v1/merchants/login', data),
}

export const merchantAPI = {
  getProfile: () => api.get('/api/v1/merchants/profile'),
  getBalance: () => api.get('/api/v1/merchants/balance'),
  getAnalytics: (params?: {
    granularity?: string;
    start_date?: string;
    end_date?: string;
  }) => {
    const query = params ? `?${new URLSearchParams(params as any).toString()}` : '';
    return api.get(`/api/v1/merchants/analytics${query}`);
  },

  // Invoice Management
  createInvoice: (data: { amount_usd: string; description: string; due_date: string }) =>
    api.post('/api/v1/merchants/invoices', data),

  getInvoices: (params?: { limit?: number; offset?: number }) => {
    const query = params ? `?${new URLSearchParams(params as any).toString()}` : '';
    return api.get(`/api/v1/merchants/invoices${query}`);
  },

  getInvoice: (invoiceId: string) => api.get(`/api/v1/merchants/invoices/${invoiceId}`),
  getFeeSetting: () => api.get('/api/v1/merchants/fee-setting'),
  updateFeeSetting: (data: { customer_pays_fee?: boolean; fee_percentage?: number }) => api.put('/api/v1/merchants/fee-setting', data),
  switchEnvironment: (toLive: boolean) =>
    api.post('/api/v1/merchants/environment/switch', { to_live: toLive }),
  updateSettlementMode: (mode: string) =>
    api.put('/api/v1/merchants/settlement-mode', { mode }), // DEPRECATED: Use updateSettings
  generateApiKey: (isLive: boolean) => api.post('/api/v1/merchants/api-keys/generate', { is_live: isLive }),
  rotateApiKey: (isLive: boolean) => api.post('/api/v1/merchants/api-keys/rotate', { is_live: isLive }),
  setWallet: (data: { crypto_type: string; address: string }) => api.put('/api/v1/merchants/wallets', data), // DEPRECATED: Use walletAPI.setup
  setWebhook: (data: { url: string }) => api.put('/api/v1/merchants/webhook', data), // DEPRECATED: Use updateSettings

  // Unified Settings & Status
  getMerchantSettings: () => api.get('/api/v1/merchants/settings'),
  updateSettings: (data: {
    webhook_url?: string;
    settlement_mode?: string;
    customer_pays_fee?: boolean;
    ip_whitelist?: string[];
    sandbox_mode?: boolean;
    rotate_webhook_secret?: boolean;
  }) => api.patch('/api/v1/merchants/settings', data),
  sendTestWebhook: () => api.post('/api/v1/merchants/webhook/test'),
  getReadinessStatus: () => api.get('/api/v1/merchants/status'),
}

export const paymentAPI = {
  create: (data: { amount_usd: string; crypto_type?: string; description?: string; metadata?: Record<string, string> }) => api.post('/api/v1/merchants/payments', data),
  getStatus: (paymentId: string) => api.get(`/api/v1/merchants/payments/${paymentId}/status`),
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
    return api.get(`/api/v1/merchants/payments${query}`);
  },
  get: (paymentId: string) => api.get(`/api/v1/merchants/payments/${paymentId}`),
  verify: (paymentId: string, data: { transaction_hash: string }) => api.post(`/api/v1/merchants/payments/${paymentId}/verify`, data),
  finalizeSelection: (payment_id: string, cryptoType: string) =>
    api.post(`/api/v1/merchants/payments/${payment_id}/select`, { crypto_type: cryptoType }),
  cancel: (payment_id: string) => api.post(`/api/v1/merchants/payments/${payment_id}/cancel`),

  // Unified Transactions
  getUnifiedTransactions: (params?: any) => {
    const query = params ? `?${new URLSearchParams(params as any).toString()}` : '';
    return api.get(`/api/v1/merchants/transactions${query}`);
  },
}

export const withdrawalAPI = {
  create: (data: { crypto_type: string; amount: number; to_address: string }) => api.post('/api/v1/merchants/withdrawals', data),
  process: (id: string, password: string) => api.post(`/api/v1/merchants/withdrawals/${id}/process`, { encryption_password: password }),
  getHistory: (params?: any) => api.get('/api/v1/merchants/withdrawals', { params }),
  validateGas: (cryptoType: string, amount: number) => api.get(`/api/v1/merchants/wallets/gas-check?crypto_type=${cryptoType}&amount=${amount}`),
}

export const walletAPI = {
  setup: (data: {
    crypto_type: string;
    mode: 'address' | 'generate' | 'import';
    address?: string;
    private_key?: string;
    is_active?: boolean;
  }) => api.post('/api/v1/merchants/wallets', data),
  getAll: () => api.get('/api/v1/merchants/wallets'),
  getBalances: () => api.get('/api/v1/merchants/wallets/balances'),
  revoke: (cryptoType: string) => api.delete(`/api/v1/merchants/wallets/${cryptoType}`),
}

export const securityAPI = {
  getEvents: (params?: any) => api.get('/api/v1/merchants/security/events', { params }),
  getAlerts: (params?: any) => api.get('/api/v1/merchants/security/alerts', { params }),
  getBalanceAlerts: (params?: any) => api.get('/api/v1/merchants/security/balance-alerts', { params }),
  checkGas: () => api.get('/api/v1/merchants/security/gas-check'),
  acknowledgeAlert: (alertId: string) =>
    api.post(`/api/v1/merchants/security/alerts/${alertId}/acknowledge`),
  resolveBalanceAlert: (alertId: string) =>
    api.post(`/api/v1/merchants/security/balance-alerts/${alertId}/resolve`),
}

export const publicAPI = {
  contact: (data: { name: string; email: string; message: string }) => api.post('/api/v1/contact', data),
  getSupportedCurrencies: (merchantId?: number) => {
    const query = merchantId ? `?merchant_id=${merchantId}` : '';
    return api.get(`/api/v1/currencies/supported${query}`);
  },
  getStatus: () => api.get('/api/v1/status'),
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
