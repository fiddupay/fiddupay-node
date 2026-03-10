import axios from 'axios';

// Admin API client
const api = axios.create({
  baseURL: import.meta.env.VITE_API_URL || 'http://localhost:5000',
  withCredentials: true,
});

export const adminAPI = {
  // Fee Sweep Settings
  getFeeSweepSettings: () => api.get('/api/v1/admin/fee-sweep/settings'),
  updateFeeSweepSettings: (data: any) => api.put('/api/v1/admin/fee-sweep/settings', data),
  triggerManualSweep: (network: string) => api.post(`/api/v1/admin/fee-sweep/trigger/${network}`),
};

export default api;
