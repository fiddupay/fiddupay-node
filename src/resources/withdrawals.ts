import { HttpClient } from '../client';
import { 
  Withdrawal, 
  CreateWithdrawalRequest, 
  ListWithdrawalsParams,
  PaginatedResponse
} from '../types';

export class Withdrawals {
  constructor(private client: HttpClient) {}

  /**
   * Create a new withdrawal
   */
  async create(data: CreateWithdrawalRequest): Promise<Withdrawal> {
    return this.client.request<Withdrawal>('POST', '/api/v1/withdrawals', data);
  }

  /**
   * List withdrawals with optional filters
   */
  async list(params?: ListWithdrawalsParams): Promise<PaginatedResponse<Withdrawal>> {
    const queryParams = new URLSearchParams();
    
    if (params?.limit) queryParams.append('limit', params.limit.toString());
    if (params?.offset) queryParams.append('offset', params.offset.toString());
    if (params?.status) queryParams.append('status', params.status);
    if (params?.crypto_type) queryParams.append('crypto_type', params.crypto_type);

    const url = `/api/v1/withdrawals${queryParams.toString() ? `?${queryParams.toString()}` : ''}`;
    return this.client.request<PaginatedResponse<Withdrawal>>('GET', url);
  }

  /**
   * Get withdrawal by ID
   */
  async get(withdrawalId: string): Promise<Withdrawal> {
    return this.client.request<Withdrawal>('GET', `/api/v1/withdrawals/${withdrawalId}`);
  }

  /**
   * Cancel withdrawal
   */
  async cancel(withdrawalId: string): Promise<Withdrawal> {
    return this.client.request<Withdrawal>('POST', `/api/v1/withdrawals/${withdrawalId}/cancel`);
  }

  /**
   * Process withdrawal
   */
  async process(withdrawalId: string): Promise<Withdrawal> {
    return this.client.request<Withdrawal>('POST', `/api/v1/withdrawals/${withdrawalId}/process`);
  }
}
