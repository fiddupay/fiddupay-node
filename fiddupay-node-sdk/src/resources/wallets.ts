import { HttpClient } from '../client';
import { RequestOptions } from '../types';

export class Wallets {
  constructor(private client: HttpClient) { }

  /**
   * Get wallet configurations
   */
  async getConfigurations(options?: RequestOptions): Promise<any> {
    return this.client.request('GET', '/api/v1/merchants/wallets');
  }

  /**
   * Unified wallet setup (address, generate, or import)
   */
  async setup(data: {
    crypto_type: string;
    mode: 'address' | 'generate' | 'import';
    address?: string;
    private_key?: string;
    is_active?: boolean;
  }, options?: RequestOptions): Promise<any> {
    return this.client.request('POST', '/api/v1/merchants/wallets', data);
  }

  /**
   * Generate a new wallet for a cryptocurrency
   * @deprecated Use setup with mode 'generate' instead
   */
  async generate(data: { crypto_type: string }, options?: RequestOptions): Promise<any> {
    return this.client.request('POST', '/api/v1/merchants/wallets/generate', data);
  }

  /**
   * Import an existing wallet using private key
   * @deprecated Use setup with mode 'import' instead
   */
  async import(data: {
    crypto_type: string;
    private_key: string
  }, options?: RequestOptions): Promise<any> {
    return this.client.request('POST', '/api/v1/merchants/wallets/import', data);
  }

  /**
   * Configure a wallet with just an address (no private key)
   * @deprecated Use setup with mode 'address' instead
   */
  async configureAddress(data: {
    crypto_type: string;
    address: string;
  }, options?: RequestOptions): Promise<any> {
    return this.client.request('POST', '/api/v1/merchants/wallets/configure-address', data);
  }

  /**
   * Export wallet key
   */
  async exportKey(data: { crypto_type: string }, options?: RequestOptions): Promise<any> {
    return this.client.request('POST', '/api/v1/merchants/wallets/export-key', data);
  }

  /**
   * Get gas estimates
   */
  async getGasEstimates(options?: RequestOptions): Promise<any> {
    return this.client.request('GET', '/api/v1/merchants/wallets/gas-estimates');
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

    const url = `/api/v1/merchants/wallets/gas-check${queryParams.toString() ? `?${queryParams.toString()}` : '?crypto_type=ETH&amount=1.0'}`;
    return this.client.request('GET', url);
  }

  /**
   * Check withdrawal capability for crypto type
   */
  async checkWithdrawalCapability(cryptoType: string, options?: RequestOptions): Promise<any> {
    return this.client.request('GET', `/api/v1/merchants/wallets/withdrawal-capability/${cryptoType}`);
  }

  /**
   * Revoke/Remove wallet configuration
   */
  async revoke(cryptoType: string, options?: RequestOptions): Promise<any> {
    return this.client.request('DELETE', `/api/v1/merchants/wallets/${cryptoType}`);
  }
}
