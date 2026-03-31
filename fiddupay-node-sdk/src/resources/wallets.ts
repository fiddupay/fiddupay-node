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
   * Get actual wallet balances and volume statistics
   * @param params Optional parameters like exclude_stats to speed up the query
   */
  async getBalances(params?: { exclude_stats?: boolean }, options?: RequestOptions): Promise<{ wallets: any[] }> {
    const query = params?.exclude_stats ? '?exclude_stats=true' : '';
    return this.client.request('GET', `/api/v1/merchants/wallets/balances${query}`, undefined, options);
  }

  /**
   * Unified wallet setup (address or generate)
   */
  async setup(data: {
    crypto_type: string;
    mode: 'address' | 'generate';
    address?: string;
    is_active?: boolean;
  }, options?: RequestOptions): Promise<any> {
    return this.client.request('POST', '/api/v1/merchants/wallets', data);
  }

  /**
   * Generate a new wallet for a cryptocurrency
   * @deprecated Use setup with mode 'generate' instead
   */
  async generate(data: { crypto_type: string }, options?: RequestOptions): Promise<any> {
    return this.setup({ crypto_type: data.crypto_type, mode: 'generate' }, options);
  }


  /**
   * Configure a wallet with just an address (no private key)
   * @deprecated Use setup with mode 'address' instead
   */
  async configureAddress(data: {
    crypto_type: string;
    address: string;
    is_active?: boolean;
  }, options?: RequestOptions): Promise<any> {
    return this.setup({ crypto_type: data.crypto_type, mode: 'address', address: data.address, is_active: data.is_active }, options);
  }


  /**
   * Get gas estimates
   */
  async getGasEstimates(options?: RequestOptions): Promise<any> {
    return this.client.request('GET', '/api/v1/merchants/wallets/gas-estimates');
  }

  /**
   * Check gas requirements (alias for gasCheck with parameters)
   */
  async checkGasRequirements(params: {
    crypto_type: string;
    amount: number;
  }, options?: RequestOptions): Promise<any> {
    const queryParams = new URLSearchParams();
    queryParams.append('crypto_type', params.crypto_type);
    queryParams.append('amount', params.amount.toString());

    return this.client.request('GET', `/api/v1/merchants/wallets/gas-check?${queryParams.toString()}`);
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
