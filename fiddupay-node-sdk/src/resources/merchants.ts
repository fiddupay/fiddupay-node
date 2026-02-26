import { HttpClient } from '../client';
import { Merchant, MerchantProfile, RequestOptions, UnifiedSettingsRequest } from '../types';

export class Merchants {
  constructor(private client: HttpClient) { }

  /**
   * Register new merchant
   */
  async register(data: {
    email: string;
    business_name: string;
    password: string;
  }): Promise<{ user: any; api_key: string }> {
    return this.client.request('POST', '/api/v1/merchants/register', data);
  }

  /**
   * Get current merchant profile
   */
  async retrieve(options?: RequestOptions): Promise<MerchantProfile> {
    return this.client.request<MerchantProfile>('GET', '/api/v1/merchants/profile');
  }

  /**
   * Get merchant readiness status
   */
  async getStatus(options?: RequestOptions): Promise<any> {
    return this.client.request('GET', '/api/v1/merchants/status');
  }

  /**
   * Set wallet address for a cryptocurrency
   */
  async setWallet(data: {
    crypto_type: string;
    address: string
  }, options?: RequestOptions): Promise<{ message: string }> {
    return this.client.request('PUT', '/api/v1/merchants/wallets', data);
  }

  async switchEnvironment(data: {
    environment: 'sandbox' | 'production'
  }, options?: RequestOptions): Promise<{ message: string; environment: string }> {
    const requestData = { to_live: data.environment === 'production' };
    return this.client.request('POST', '/api/v1/merchants/environment/switch', requestData);
  }

  /**
   * Generate new API key
   */
  async generateApiKey(data?: {
    environment?: 'sandbox' | 'production'
  }, options?: RequestOptions): Promise<{ api_key: string; environment: string }> {
    const requestData = data ? { is_live: data.environment === 'production' } : { is_live: false };
    return this.client.request('POST', '/api/v1/merchants/api-keys/generate', requestData);
  }

  /**
   * Rotate existing API key
   */
  async rotateApiKey(data?: {
    environment?: 'sandbox' | 'production'
  }, options?: RequestOptions): Promise<{ api_key: string }> {
    const requestData = data ? { is_live: data.environment === 'production' } : { is_live: false };
    return this.client.request('POST', '/api/v1/merchants/api-keys/rotate', requestData);
  }

  /**
   * Set webhook URL
   * @deprecated Use updateSettings instead
   */
  async setWebhook(data: {
    webhook_url: string
  }, options?: RequestOptions): Promise<{ message: string }> {
    const requestData = { url: data.webhook_url };
    return this.client.request('PUT', '/api/v1/merchants/webhook', requestData);
  }

  /**
   * Set IP whitelist
   * @deprecated Use updateSettings instead
   */
  async setIpWhitelist(data: {
    ip_addresses: string[]
  }, options?: RequestOptions): Promise<{ message: string }> {
    return this.client.request('PUT', '/api/v1/merchants/ip-whitelist', data);
  }

  /**
   * Get IP whitelist
   */
  async getIpWhitelist(options?: RequestOptions): Promise<{ ip_addresses: string[] }> {
    return this.client.request('GET', '/api/v1/merchants/ip-whitelist');
  }

  /**
   * Get merchant balance
   */
  async getBalance(options?: RequestOptions): Promise<{
    balances: Array<{
      crypto_type: string;
      balance: string;
      balance_usd: string;
      pending: string;
      pending_usd: string;
    }>;
    total_balance_usd: string;
    total_pending_usd: string;
  }> {
    return this.client.request('GET', '/api/v1/merchants/balance');
  }

  /**
   * Update global settlement mode
   */
  /**
   * @deprecated Use updateSettings instead
   */
  async updateSettlementMode(mode: 'forwarding' | 'managed' | 'imported'): Promise<any> {
    return this.client.request('PUT', '/api/v1/merchants/settlement-mode', { mode });
  }

  /**
   * Update global merchant settings (Unified)
   */
  async updateSettings(data: UnifiedSettingsRequest, options?: RequestOptions): Promise<{ status: string; message: string }> {
    return this.client.request('PATCH', '/api/v1/merchants/settings', data);
  }

  /**
   * Get global merchant settings
   */
  async getSettings(options?: RequestOptions): Promise<UnifiedSettingsRequest> {
    return this.client.request<UnifiedSettingsRequest>('GET', '/api/v1/merchants/settings');
  }

  /**
   * Send a test webhook
   */
  async sendTestWebhook(options?: RequestOptions): Promise<{ status: string; message: string }> {
    return this.client.request('POST', '/api/v1/merchants/webhook/test');
  }
}
