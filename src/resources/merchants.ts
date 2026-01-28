import { HttpClient } from '../client';
import { Merchant } from '../types';

export class Merchants {
  constructor(private client: HttpClient) {}

  /**
   * Register new merchant
   */
  async register(data: {
    email: string;
    business_name: string;
    password: string;
  }): Promise<{ user: any; api_key: string }> {
    return this.client.request('POST', '/api/v1/merchant/register', data);
  }

  /**
   * Get current merchant profile
   */
  async retrieve(): Promise<Merchant> {
    return this.client.request<Merchant>('GET', '/api/v1/merchant/profile');
  }

  /**
   * Set wallet address for a cryptocurrency
   */
  async setWallet(data: { 
    crypto_type: string; 
    address: string 
  }): Promise<{ message: string }> {
    return this.client.request('PUT', '/api/v1/merchant/wallets', data);
  }

  /**
   * Switch environment (sandbox/production)
   */
  async switchEnvironment(data: { 
    environment: 'sandbox' | 'production' 
  }): Promise<{ message: string; environment: string }> {
    const requestData = { to_live: data.environment === 'production' };
    return this.client.request('POST', '/api/v1/merchant/environment/switch', requestData);
  }

  /**
   * Generate new API key
   */
  async generateApiKey(data?: { 
    environment?: 'sandbox' | 'production' 
  }): Promise<{ api_key: string; environment: string }> {
    const requestData = data ? { is_live: data.environment === 'production' } : { is_live: false };
    return this.client.request('POST', '/api/v1/merchant/api-keys/generate', requestData);
  }

  /**
   * Rotate existing API key
   */
  async rotateApiKey(data?: { 
    environment?: 'sandbox' | 'production' 
  }): Promise<{ api_key: string }> {
    const requestData = data ? { is_live: data.environment === 'production' } : { is_live: false };
    return this.client.request('POST', '/api/v1/merchant/api-keys/rotate', requestData);
  }

  /**
   * Set webhook URL
   */
  async setWebhook(data: { 
    webhook_url: string 
  }): Promise<{ message: string }> {
    const requestData = { url: data.webhook_url };
    return this.client.request('PUT', '/api/v1/merchant/webhook', requestData);
  }

  /**
   * Set IP whitelist
   */
  async setIpWhitelist(data: { 
    ip_addresses: string[] 
  }): Promise<{ message: string }> {
    return this.client.request('PUT', '/api/v1/merchant/ip-whitelist', data);
  }

  /**
   * Get IP whitelist
   */
  async getIpWhitelist(): Promise<{ ip_addresses: string[] }> {
    return this.client.request('GET', '/api/v1/merchant/ip-whitelist');
  }

  /**
   * Get merchant balance
   */
  async getBalance(): Promise<{
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
    return this.client.request('GET', '/api/v1/merchant/balance');
  }

  /**
   * Set wallet addresses for automatic forwarding
   */
  async setWallets(
    wallets: Record<string, string>
  ): Promise<{ message: string; wallets: Record<string, string> }> {
    return this.client.request('PUT', '/api/v1/merchant/wallets', { wallets });
  }

  /**
   * Get balance history
   */
  async getBalanceHistory(): Promise<{
    history: Array<{
      date: string;
      balance_usd: string;
      change_usd: string;
      change_percentage: number;
    }>;
  }> {
    return this.client.request('GET', '/api/v1/merchant/balance/history');
  }

  /**
   * Login merchant
   */
  async login(data: {
    email: string;
    password: string;
  }): Promise<{ user: any; api_key: string }> {
    return this.client.request('POST', '/api/v1/merchant/login', data);
  }

  /**
   * Get audit logs
   */
  async getAuditLogs(): Promise<{
    logs: Array<{
      id: string;
      action: string;
      timestamp: string;
      ip_address: string;
      user_agent: string;
    }>;
  }> {
    return this.client.request('GET', '/api/v1/audit-logs');
  }
}
