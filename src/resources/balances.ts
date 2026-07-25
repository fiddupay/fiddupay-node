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
  constructor(private client: HttpClient) { }

  /**
   * Get current balance
   */
  async get(): Promise<Balance> {
    return this.client.request<Balance>('GET', '/api/v1/merchants/balance');
  }

  /**
   * Get balance history
   */
  async getHistory(params?: ListBalanceHistoryParams): Promise<BalanceHistory> {
    const queryParams = new URLSearchParams();

    if (params) {
      for (const [key, value] of Object.entries(params)) {
        if (value !== undefined && value !== null) {
          queryParams.append(key, value.toString());
        }
      }
    }

    const url = `/api/v1/merchants/balance/history${queryParams.toString() ? `?${queryParams.toString()}` : ''}`;
    return this.client.request<BalanceHistory>('GET', url);
  }
}

export class AuditLogs {
  constructor(private client: HttpClient) { }

  /**
   * Get audit logs
   */
  async list(params?: ListAuditLogsParams): Promise<AuditLog[]> {
    const queryParams = new URLSearchParams();

    if (params) {
      for (const [key, value] of Object.entries(params)) {
        if (value !== undefined && value !== null) {
          queryParams.append(key, value.toString());
        }
      }
    }

    const url = `/api/v1/merchants/audit-logs${queryParams.toString() ? `?${queryParams.toString()}` : ''}`;
    return this.client.request<AuditLog[]>('GET', url);
  }
}
