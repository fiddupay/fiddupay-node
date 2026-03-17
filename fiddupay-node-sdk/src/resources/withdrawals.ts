import { HttpClient } from '../client';
import {
  Withdrawal,
  CreateWithdrawalRequest,
  ListWithdrawalsParams,
  PaginatedResponse
} from '../types';

export class Withdrawals {
  constructor(private client: HttpClient) { }

  /**
   * Create a new withdrawal
   */
  async create(data: CreateWithdrawalRequest): Promise<Withdrawal> {
    return this.client.request<Withdrawal>('POST', '/api/v1/merchants/withdrawals', data);
  }

  /**
   * List withdrawals with optional filters
   */
  async list(params?: ListWithdrawalsParams): Promise<Withdrawal[]> {
    const queryParams = new URLSearchParams();

    if (params?.limit) queryParams.append('limit', params.limit.toString());
    if (params?.offset) queryParams.append('offset', params.offset.toString());
    if (params?.status) queryParams.append('status', params.status);
    if (params?.crypto_type) queryParams.append('crypto_type', params.crypto_type);

    const url = `/api/v1/merchants/withdrawals${queryParams.toString() ? `?${queryParams.toString()}` : ''}`;
    return this.client.request<Withdrawal[]>('GET', url);
  }

  /**
   * Get withdrawal by ID
   */
  async get(withdrawalId: string): Promise<Withdrawal> {
    return this.client.request<Withdrawal>('GET', `/api/v1/merchants/withdrawals/${withdrawalId}`);
  }

  /**
   * Cancel withdrawal
   */
  async cancel(withdrawalId: string): Promise<Withdrawal> {
    return this.client.request<Withdrawal>('POST', `/api/v1/merchants/withdrawals/${withdrawalId}/cancel`);
  }

  /**
   * Process withdrawal (managed mode or admin triggered)
   */
  async process(withdrawalId: string, encryptionPassword?: string): Promise<Withdrawal> {
    const data = encryptionPassword ? { encryption_password: encryptionPassword } : {};
    return this.client.request<Withdrawal>('POST', `/api/v1/merchants/withdrawals/${withdrawalId}/process`, data);
  }

  /**
   * Validate gas requirements for a withdrawal
   */
  async validateGas(cryptoType: string, amount: string): Promise<any> {
    return this.client.request('GET', `/api/v1/merchants/wallets/gas-check?crypto_type=${cryptoType}&amount=${amount}`);
  }

  /**
   * Get gas price estimates for all supported networks
   */
  async getGasEstimates(): Promise<any> {
    return this.client.request('GET', '/api/v1/merchants/wallets/gas-estimates');
  }

  /**
   * Check if withdrawal is possible for a specific cryptocurrency
   */
  async checkCapability(cryptoType: string): Promise<any> {
    return this.client.request('GET', `/api/v1/merchants/wallets/withdrawal-capability/${cryptoType}`);
  }
}
