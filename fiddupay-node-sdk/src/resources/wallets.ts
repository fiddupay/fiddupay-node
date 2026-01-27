import { HttpClient } from '../client';
import { RequestOptions } from '../types';

export class Wallets {
  constructor(private client: HttpClient) {}

  /**
   * Get wallet configurations
   */
  async getConfigurations(options?: RequestOptions): Promise<any> {
    return this.client.request('GET', '/api/v1/wallets');
  }

  /**
   * Generate new wallet
   */
  async generate(data: { crypto_type: string }, options?: RequestOptions): Promise<any> {
    return this.client.request('POST', '/api/v1/wallets/generate', data);
  }

  /**
   * Import wallet
   */
  async import(data: { 
    crypto_type: string; 
    private_key: string 
  }, options?: RequestOptions): Promise<any> {
    return this.client.request('POST', '/api/v1/wallets/import', data);
  }

  /**
   * Configure wallet address
   */
  async configureAddress(data: {
    crypto_type: string;
    address: string;
  }, options?: RequestOptions): Promise<any> {
    return this.client.request('POST', '/api/v1/wallets/configure-address', data);
  }

  /**
   * Export wallet key
   */
  async exportKey(data: { crypto_type: string }, options?: RequestOptions): Promise<any> {
    return this.client.request('POST', '/api/v1/wallets/export-key', data);
  }

  /**
   * Get gas estimates
   */
  async getGasEstimates(options?: RequestOptions): Promise<any> {
    return this.client.request('GET', '/api/v1/wallets/gas-estimates');
  }

  /**
   * Check gas requirements
   */
  async checkGasRequirements(params?: {
    crypto_type?: string;
    amount?: number;
  }, options?: RequestOptions): Promise<any> {
    const queryParams = new URLSearchParams();
    if (params?.crypto_type) queryParams.append('crypto_type', params.crypto_type);
    if (params?.amount) queryParams.append('amount', params.amount.toString());
    
    const url = `/api/v1/wallets/gas-check${queryParams.toString() ? `?${queryParams.toString()}` : '?crypto_type=ETH&amount=1.0'}`;
    return this.client.request('GET', url);
  }

  /**
   * Check withdrawal capability for crypto type
   */
  async checkWithdrawalCapability(cryptoType: string, options?: RequestOptions): Promise<any> {
    return this.client.request('GET', `/api/v1/wallets/withdrawal-capability/${cryptoType}`);
  }
}
