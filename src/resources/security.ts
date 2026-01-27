import { HttpClient } from '../client';
import { RequestOptions } from '../types';

export class Security {
  constructor(private client: HttpClient) {}

  /**
   * Get security events
   */
  async getEvents(options?: RequestOptions): Promise<any> {
    return this.client.request('GET', '/api/v1/security/events');
  }

  /**
   * Get security alerts
   */
  async getAlerts(options?: RequestOptions): Promise<any> {
    return this.client.request('GET', '/api/v1/security/alerts');
  }

  /**
   * Get security settings
   */
  async getSettings(options?: RequestOptions): Promise<any> {
    return this.client.request('GET', '/api/v1/security/settings');
  }

  /**
   * Update security settings
   */
  async updateSettings(data: {
    max_daily_withdrawal?: number;
    require_2fa_for_withdrawals?: boolean;
  }, options?: RequestOptions): Promise<any> {
    return this.client.request('PUT', '/api/v1/security/settings', data);
  }

  /**
   * Check gas balances
   */
  async checkGasBalances(options?: RequestOptions): Promise<any> {
    return this.client.request('GET', '/api/v1/security/gas-check');
  }

  /**
   * Get balance alerts
   */
  async getBalanceAlerts(options?: RequestOptions): Promise<any> {
    return this.client.request('GET', '/api/v1/security/balance-alerts');
  }

  /**
   * Acknowledge security alert
   */
  async acknowledgeAlert(alertId: string, options?: RequestOptions): Promise<any> {
    return this.client.request('POST', `/api/v1/security/alerts/${alertId}/acknowledge`);
  }

  /**
   * Resolve balance alert
   */
  async resolveBalanceAlert(alertId: string, options?: RequestOptions): Promise<any> {
    return this.client.request('POST', `/api/v1/security/balance-alerts/${alertId}/resolve`);
  }
}
