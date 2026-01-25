import { HttpClient } from '../client';
import { Merchant, RequestOptions } from '../types';

export class Merchants {
  constructor(private client: HttpClient) {}

  /**
   * Get current merchant profile
   */
  async retrieve(options?: RequestOptions): Promise<Merchant> {
    return this.client.get<Merchant>('/merchants/profile', options);
  }

  /**
   * Update webhook URL
   */
  async updateWebhook(data: { url: string }, options?: RequestOptions): Promise<{ message: string; webhook_url: string }> {
    return this.client.put('/merchants/webhook', data, options);
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
    return this.client.get('/merchants/balance', options);
  }

  /**
   * Set wallet addresses for automatic forwarding
   */
  async setWallets(
    wallets: Record<string, string>,
    options?: RequestOptions
  ): Promise<{ message: string; wallets: Record<string, string> }> {
    return this.client.put('/merchants/wallets', { wallets }, options);
  }

  /**
   * Regenerate API key
   */
  async regenerateApiKey(options?: RequestOptions): Promise<{ api_key: string }> {
    return this.client.post('/merchants/api-key/regenerate', {}, options);
  }
}
