// API Service - Centralized API calls
import api from '@/utils/api'
import { LoginCredentials } from '@/types'

export const authAPI = {
  register: (data: any) =>
    api.post('/api/v1/merchants/register', data),

  registerP2P: (data: any) =>
    api.post('/api/v1/p2p/register', data),

  login: (data: LoginCredentials) =>
    api.post('/api/v1/merchants/login', data),
}

const cleanParams = (params: any) => {
  const cleaned: any = {};
  Object.keys(params).forEach(key => {
    if (params[key] !== undefined && params[key] !== null && params[key] !== '') {
      cleaned[key] = params[key];
    }
  });
  return cleaned;
};

export const merchantAPI = {
  getProfile: () => api.get('/api/v1/merchants/profile'),
  getBalance: () => api.get('/api/v1/merchants/balance'),
  getAnalytics: (params?: {
    granularity?: string;
    from_date?: string;
    to_date?: string;
    status?: string;
    blockchain?: string;
  }) => {
    const query = params ? `?${new URLSearchParams(cleanParams(params)).toString()}` : '';
    return api.get(`/api/v1/merchants/analytics${query}`);
  },
  getBalanceHistory: (params?: { limit?: number }) => {
    const query = params ? `?${new URLSearchParams(cleanParams(params as any)).toString()}` : '';
    return api.get(`/api/v1/merchants/balance/history${query}`);
  },

  // Invoice Management
  createInvoice: (data: { customer_email?: string; customer_name?: string; items: any[]; tax?: string; notes?: string; due_date?: string; currency?: string }) =>
    api.post('/api/v1/merchants/invoices', data),

  getInvoices: (params?: { limit?: number; offset?: number }) => {
    const query = params ? `?${new URLSearchParams(cleanParams(params)).toString()}` : '';
    return api.get(`/api/v1/merchants/invoices${query}`);
  },

  getInvoice: (invoiceId: string) => api.get(`/api/v1/merchants/invoices/${invoiceId}`),
  getFeeSetting: () => api.get('/api/v1/merchants/fee-setting'),
  switchEnvironment: (toLive: boolean) =>
    api.post('/api/v1/merchants/environment/switch', { to_live: toLive }),
  generateApiKey: (isLive: boolean) => api.post('/api/v1/merchants/api-keys/generate', { is_live: isLive }),
  rotateApiKey: (isLive: boolean) => api.post('/api/v1/merchants/api-keys/rotate', { is_live: isLive }),

  // Unified Settings & Status
  getMerchantSettings: () => api.get('/api/v1/merchants/settings'),
  updateSettings: (data: {
    webhook_url?: string;
    webhook_format?: string;
    redirect_url?: string;
    settlement_mode?: string;
    customer_pays_fee?: boolean;
    fee_percentage?: number;
    ip_whitelist?: string[];
    sandbox_mode?: boolean;
    rotate_webhook_secret?: boolean;
    low_balance_alerts_enabled?: boolean;
    low_balance_threshold_usd?: string;
    transaction_pin?: string;
  }) => api.patch('/api/v1/merchants/settings', data),
  setTransactionPin: (pin: string) => api.post('/api/v1/merchants/security/pin', { pin }),
  verifyTransactionPin: (pin: string) => api.post('/api/v1/merchants/security/pin/verify', { pin }),
  sendTestWebhook: () => api.post('/api/v1/merchants/webhook/test'),
  getReadinessStatus: () => api.get('/api/v1/merchants/status'),
  getAuditLogs: (params?: { limit?: number; from?: string; to?: string; action_type?: string }) => {
    const query = params ? `?${new URLSearchParams(cleanParams(params)).toString()}` : '';
    return api.get(`/api/v1/merchants/audit-logs${query}`);
  },
}

export const paymentAPI = {
  create: (data: any) => api.post('/api/v1/merchants/payments', data),
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
    const query = params ? `?${new URLSearchParams(cleanParams(params)).toString()}` : '';
    return api.get(`/api/v1/merchants/payments${query}`);
  },
  get: (paymentId: string) => api.get(`/api/v1/merchants/payments/${paymentId}`),
  verify: (paymentId: string, data: { transaction_hash: string }) => api.post(`/api/v1/merchants/payments/${paymentId}/verify`, data),
  finalizeSelection: (payment_id: string, cryptoType: string) =>
    api.post(`/api/v1/merchants/payments/${payment_id}/select`, { crypto_type: cryptoType }),
  cancel: (payment_id: string) => api.post(`/api/v1/merchants/payments/${payment_id}/cancel`),

  // Unified Transactions
  getUnifiedTransactions: (params?: any) => {
    const query = params ? `?${new URLSearchParams(cleanParams(params)).toString()}` : '';
    return api.get(`/api/v1/merchants/transactions${query}`);
  },
}

export const refundAPI = {
  create: (data: { payment_id: string; amount: string; reason: string }) =>
    api.post('/api/v1/merchants/refunds', data),
  get: (refundId: string) =>
    api.get(`/api/v1/merchants/refunds/${refundId}`),
  complete: (refundId: string) =>
    api.post(`/api/v1/merchants/refunds/${refundId}/complete`),
}

export const withdrawalAPI = {
  create: (data: { crypto_type: string; amount: string | number; to_address?: string; destination_address?: string; description?: string; pin: string }) => api.post('/api/v1/merchants/withdrawals', data),
  get: (id: string) => api.get(`/api/v1/merchants/withdrawals/${id}`),
  cancel: (id: string) => api.post(`/api/v1/merchants/withdrawals/${id}/cancel`),
  process: (id: string, password: string) => api.post(`/api/v1/merchants/withdrawals/${id}/process`, { encryption_password: password }),
  getHistory: (params?: any) => api.get('/api/v1/merchants/withdrawals', { params }),
  validateGas: (cryptoType: string, amount: number) => api.get(`/api/v1/merchants/wallets/gas-check?crypto_type=${cryptoType}&amount=${amount}`),
}

export const walletAPI = {
  setup: (data: {
    crypto_type: string;
    mode: 'address' | 'generate';
    address?: string;
    is_active?: boolean;
    enable_all_evm?: boolean;
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
  getSettings: () => api.get('/api/v1/merchants/security/settings'),
  acknowledgeAlert: (alertId: string) =>
    api.post(`/api/v1/merchants/security/alerts/${alertId}/acknowledge`),
  resolveBalanceAlert: (alertId: string) =>
    api.post(`/api/v1/merchants/security/balance-alerts/${alertId}/resolve`),
  toggleWalletLock: (locked: boolean, password: string) =>
    api.post('/api/v1/merchants/security/wallets/lock', { locked, password }),
  toggleCustomerWalletLock: (locked: boolean, password: string) =>
    api.post('/api/v1/merchants/security/customers/wallets/lock', { locked, password }),
}

export const sandboxAPI = {
  enable: () => api.post('/api/v1/merchants/sandbox/enable'),
  simulate: (paymentId: string, data: { success: boolean; transaction_hash?: string; from_address?: string }) =>
    api.post(`/api/v1/merchants/sandbox/payments/${paymentId}/simulate`, data),
}

export const customerAPI = {
  list: (params?: any) => api.get('/api/v1/merchants/customers', { params }),
  create: (data: { external_id: string; email?: string; first_name?: string; last_name?: string }) => api.post('/api/v1/merchants/customers', data),
  getWallets: (externalId: string) => api.get(`/api/v1/merchants/customers/${externalId}/wallets`),
  createWallets: (externalId: string, data: { networks: string[] }) => api.post(`/api/v1/merchants/customers/${externalId}/wallets`, data),
  bulkProvisionWallets: (data: { customer_ids?: string[], all_customers: boolean }) => api.post('/api/v1/merchants/customers/bulk-provision', data),
  getBalances: (externalId: string) => api.get(`/api/v1/merchants/customers/${externalId}/balances`),
  getDepositAddress: (externalId: string, cryptoType: string) => api.get(`/api/v1/merchants/customers/${externalId}/deposit-address/${cryptoType}`),
  getTransactions: (externalId: string, params?: any) => api.get(`/api/v1/merchants/customers/${externalId}/transactions`, { params }),
  payMerchant: (externalId: string, data: { crypto_type: string; amount: string; reference_id?: string; description?: string }) => api.post(`/api/v1/merchants/customers/${externalId}/pay-merchant`, data),
  updateStatus: (externalId: string, data: { status: string; reason?: string }) => api.patch(`/api/v1/merchants/customers/${externalId}/status`, data),
  updatePermissions: (externalId: string, data: { can_withdraw?: boolean; withdrawal_limit?: string }) => api.patch(`/api/v1/merchants/customers/${externalId}/permissions`, data),
  withdraw: (externalId: string, data: { crypto_type: string; amount: string; destination_address: string; pin?: string }) => api.post(`/api/v1/merchants/customers/${externalId}/withdraw`, data),
  sweep: (externalId: string, data: { crypto_type: string; amount: string; pin?: string }) => api.post(`/api/v1/merchants/customers/${externalId}/sweep`, data),
  deactivate: (externalId: string) => api.post(`/api/v1/merchants/customers/${externalId}/deactivate`),
}

export const publicAPI = {
  contact: (data: { name: string; email: string; subject: string; message: string }) => api.post('/api/v1/contact', data),
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
  customer: customerAPI,
  refund: refundAPI,
  sandbox: sandboxAPI,
  public: publicAPI,
}
