import { HttpClient } from '../client';
import { RequestOptions } from '../types';

export class Security {
  constructor(private client: HttpClient) { }

  /**
   * Get security events
   */
  async getEvents(options?: RequestOptions): Promise<any> {
    return this.client.request('GET', '/api/v1/merchants/security/events');
  }

  /**
   * Get security alerts
   */
  async getAlerts(options?: RequestOptions): Promise<any> {
    return this.client.request('GET', '/api/v1/merchants/security/alerts');
  }

  /**
   * Get security settings
   */
  async getSettings(options?: RequestOptions): Promise<any> {
    return this.client.request('GET', '/api/v1/merchants/security/settings');
  }

  /**
   * Update security settings
   */
  async updateSettings(data: {
    max_daily_withdrawal?: number;
    require_2fa_for_withdrawals?: boolean;
  }, options?: RequestOptions): Promise<any> {
    return this.client.request('PUT', '/api/v1/merchants/security/settings', data);
  }

  /**
   * Check gas balances
   */
  async checkGasBalances(options?: RequestOptions): Promise<any> {
    return this.client.request('GET', '/api/v1/merchants/security/gas-check');
  }

  /**
   * Get balance alerts
   */
  async getBalanceAlerts(options?: RequestOptions): Promise<any> {
    return this.client.request('GET', '/api/v1/merchants/security/balance-alerts');
  }

  /**
   * Acknowledge security alert
   */
  async acknowledgeAlert(alertId: string, options?: RequestOptions): Promise<any> {
    return this.client.request('POST', `/api/v1/merchants/security/alerts/${alertId}/acknowledge`);
  }

  /**
   * Resolve balance alert
   */
  async resolveBalanceAlert(alertId: string, options?: RequestOptions): Promise<any> {
    return this.client.request('POST', `/api/v1/merchants/security/balance-alerts/${alertId}/resolve`);
  }

  /**
   * Toggle master wallet withdrawal lock
   */
  async toggleWalletLock(locked: boolean, password: string, options?: RequestOptions): Promise<{ status: string; locked: boolean }> {
    return this.client.post('/api/v1/merchants/security/wallets/lock', { locked, password }, options);
  }

  /**
   * Toggle customer designated wallet withdrawal lock
   */
  async toggleCustomerWalletLock(locked: boolean, password: string, options?: RequestOptions): Promise<{ status: string; locked: boolean }> {
    return this.client.post('/api/v1/merchants/security/customers/wallets/lock', { locked, password }, options);
  }

  /**
   * Set or update the merchant's 4-digit Transaction PIN
   */
  async setTransactionPin(pin: string, options?: RequestOptions): Promise<any> {
    return this.client.post('/api/v1/merchants/security/transaction-pin', { pin }, options);
  }

  /**
   * Verify the merchant's Transaction PIN
   */
  async verifyTransactionPin(pin: string, options?: RequestOptions): Promise<any> {
    return this.client.post('/api/v1/merchants/security/transaction-pin/verify', { pin }, options);
  }
}
