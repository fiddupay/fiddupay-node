import axios from 'axios';

// Admin API client
const api = axios.create({
  baseURL: import.meta.env.VITE_API_URL || 'http://localhost:5000',
  withCredentials: true,
});

export const adminAPI = {
  // Auth
  login: (data: any) => api.post('/api/v1/admin/login', data),
  logout: () => api.post('/api/v1/admin/logout'),

  // Dashboard & Stats
  getDashboardStats: () => api.get('/api/v1/admin/dashboard'),
  getMerchantsSummary: () => api.get('/api/v1/admin/merchants/summary'),

  // Merchants
  getMerchants: () => api.get('/api/v1/admin/merchants'),
  getMerchantDetails: (id: string) => api.get(`/api/v1/admin/merchants/${id}`),
  updateMerchantStatus: (id: string, status: string) => api.put(`/api/v1/admin/merchants/${id}/status`, { status }),

  // Payments
  getPayments: () => api.get('/api/v1/admin/payments'),
  getPaymentDetails: (id: string) => api.get(`/api/v1/admin/payments/${id}`),
  forceConfirmPayment: (id: string) => api.post(`/api/v1/admin/payments/${id}/confirm`),

  // Withdrawals
  getWithdrawals: () => api.get('/api/v1/admin/withdrawals'),
  approveWithdrawal: (id: string) => api.post(`/api/v1/admin/withdrawals/${id}/approve`),
  rejectWithdrawal: (id: string) => api.post(`/api/v1/admin/withdrawals/${id}/reject`),

  // Fee Sweep & Wallets
  getFeeSweepSettings: () => api.get('/api/v1/admin/fee-sweep/settings'),
  updateFeeSweepSettings: (data: any) => api.put('/api/v1/admin/fee-sweep/settings', data),
  triggerManualSweep: (network: string) => api.post(`/api/v1/admin/fee-sweep/trigger/${network}`),
  getWalletBalances: () => api.get('/api/v1/admin/wallets/balances'),

  // System
  getSystemHealth: () => api.get('/api/v1/admin/system/health'),
  getSystemLogs: () => api.get('/api/v1/admin/system/logs'),
  getAuditLogs: () => api.get('/api/v1/admin/system/audit'),

  // Rectification
  rectifyOnchain: (data: { 
    address: string, 
    crypto_type: string, 
    dry_run?: boolean, 
    signature_limit?: number, 
    sandbox_mode?: boolean,
    rectify_type?: string 
  }) => api.post('/api/v1/admin/rectify/onchain', data),
};

export default api;
