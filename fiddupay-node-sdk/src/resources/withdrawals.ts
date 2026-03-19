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

    if (params) {
      for (const [key, value] of Object.entries(params)) {
        if (value !== undefined && value !== null) {
          queryParams.append(key, value.toString());
        }
      }
    }

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
  async process(withdrawalId: string, encryptionPassword: string): Promise<Withdrawal> {
    return this.client.request<Withdrawal>('POST', `/api/v1/merchants/withdrawals/${withdrawalId}/process`, { encryption_password: encryptionPassword });
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
