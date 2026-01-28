import { HttpClient } from '../client';
import { 
  Balance, 
  BalanceHistory,
  AuditLog,
  ListAuditLogsParams,
  ListBalanceHistoryParams,
  PaginatedResponse
} from '../types';

export class Balances {
  constructor(private client: HttpClient) {}

  /**
   * Get current balance
   */
  async get(): Promise<Balance> {
    return this.client.request<Balance>('GET', '/api/v1/merchant/balance');
  }

  /**
   * Get balance history
   */
  async getHistory(params?: ListBalanceHistoryParams): Promise<PaginatedResponse<BalanceHistory>> {
    const queryParams = new URLSearchParams();
    
    if (params?.limit) queryParams.append('limit', params.limit.toString());
    if (params?.offset) queryParams.append('offset', params.offset.toString());
    if (params?.crypto_type) queryParams.append('crypto_type', params.crypto_type);

    const url = `/api/v1/merchant/balance/history${queryParams.toString() ? `?${queryParams.toString()}` : ''}`;
    return this.client.request<PaginatedResponse<BalanceHistory>>('GET', url);
  }
}

export class AuditLogs {
  constructor(private client: HttpClient) {}

  /**
   * Get audit logs
   */
  async list(params?: ListAuditLogsParams): Promise<PaginatedResponse<AuditLog>> {
    const queryParams = new URLSearchParams();
    
    if (params?.limit) queryParams.append('limit', params.limit.toString());
    if (params?.offset) queryParams.append('offset', params.offset.toString());
    if (params?.action) queryParams.append('action', params.action);
    if (params?.start_date) queryParams.append('start_date', params.start_date);
    if (params?.end_date) queryParams.append('end_date', params.end_date);

    const url = `/api/v1/merchant/audit-logs${queryParams.toString() ? `?${queryParams.toString()}` : ''}`;
    return this.client.request<PaginatedResponse<AuditLog>>('GET', url);
  }
}
