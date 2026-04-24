import { HttpClient } from '../client';
import { Analytics, Balance, BalanceHistory, RequestOptions, UnifiedTransactionsResponse } from '../types';

export class AnalyticsResource {
  constructor(private client: HttpClient) { }

  /**
   * Get analytics data
   */
  async retrieve(params?: {
    from_date?: string;
    to_date?: string;
    status?: string;
    blockchain?: string;
    granularity?: 'day' | 'week' | 'month';
  }, options?: RequestOptions): Promise<Analytics> {
    const queryParams = new URLSearchParams();

    if (params?.from_date) queryParams.append('from_date', params.from_date);
    if (params?.to_date) queryParams.append('to_date', params.to_date);
    if (params?.status) queryParams.append('status', params.status);
    if (params?.blockchain) queryParams.append('blockchain', params.blockchain);
    if (params?.granularity) queryParams.append('granularity', params.granularity);

    const query = queryParams.toString();
    const path = query ? `/api/v1/merchants/analytics?${query}` : '/api/v1/merchants/analytics';

    return this.client.request<Analytics>('GET', path);
  }

  /**
   * Export analytics data
   */
  async export(params: {
    format?: 'csv' | 'json' | 'xlsx';
    from_date: string;
    to_date: string;
    status?: string;
    blockchain?: string;
  }, options?: RequestOptions): Promise<string> {
    const queryParams = new URLSearchParams();
    if (params.format) queryParams.append('format', params.format);
    queryParams.append('from_date', params.from_date);
    queryParams.append('to_date', params.to_date);
    if (params.status) queryParams.append('status', params.status);
    if (params.blockchain) queryParams.append('blockchain', params.blockchain);

    return this.client.request('GET', `/api/v1/merchants/analytics/export?${queryParams.toString()}`);
  }

  /**
   * Get chronological feed combining payments, refunds, and withdrawals.
   * Alias for listUnifiedTransactions()
   * @deprecated Use listUnifiedTransactions() instead
   * @returns A promise resolving to a list of {@link UnifiedTransaction} wrapped in a response object.
   */
  async getUnifiedTransactions(params?: {
    limit?: number;
    offset?: number;
    from_date?: string;
    to_date?: string;
    txn_type?: 'payment' | 'refund' | 'withdrawal';
  }, options?: RequestOptions): Promise<UnifiedTransactionsResponse> {
    return this.listUnifiedTransactions(params, options);
  }

  /**
   * List combined analytical transactions (payments, refunds, withdrawals, customer txns)
   * @returns A promise resolving to a list of {@link UnifiedTransaction} wrapped in a response object.
   */
  async listUnifiedTransactions(params?: {
    limit?: number;
    offset?: number;
    from_date?: string;
    to_date?: string;
    txn_type?: 'payment' | 'refund' | 'withdrawal';
  }, options?: RequestOptions): Promise<UnifiedTransactionsResponse> {
    const queryParams = new URLSearchParams();
    if (params?.limit) queryParams.append('limit', params.limit.toString());
    if (params?.offset) queryParams.append('offset', params.offset.toString());
    if (params?.from_date) queryParams.append('from_date', params.from_date);
    if (params?.to_date) queryParams.append('to_date', params.to_date);
    if (params?.txn_type) queryParams.append('txn_type', params.txn_type);

    return this.client.get(`/api/v1/merchants/transactions${queryParams.toString() ? `?${queryParams.toString()}` : ''}`, options);
  }

  /**
   * Get current aggregated merchant balances across all assets
   */
  async getBalances(options?: RequestOptions): Promise<Balance> {
    return this.client.request<Balance>('GET', '/api/v1/merchants/balance');
  }

  /**
   * Get balance history for trend analysis
   * @param limit Number of history points to retrieve
   */
  async getBalanceHistory(limit?: number, options?: RequestOptions): Promise<BalanceHistory> {
    const query = limit ? `?limit=${limit}` : '';
    return this.client.request<BalanceHistory>('GET', `/api/v1/merchants/balance/history${query}`);
  }
}
