import { HttpClient } from '../client';

export class Security {
  constructor(private client: HttpClient) {}

  /**
   * Get security events
   */
  async getEvents(): Promise<any> {
    return this.client.request('GET', '/api/v1/merchant/security/events');
  }

  /**
   * Get security alerts
   */
  async getAlerts(): Promise<any> {
    return this.client.request('GET', '/api/v1/merchant/security/alerts');
  }

  /**
   * Get security settings
   */
  async getSettings(): Promise<any> {
    return this.client.request('GET', '/api/v1/merchant/security/settings');
  }

  /**
   * Update security settings
   */
  async updateSettings(data: {
    max_daily_withdrawal?: number;
    require_2fa_for_withdrawals?: boolean;
  }): Promise<any> {
    return this.client.request('PUT', '/api/v1/merchant/security/settings', data);
  }

  /**
   * Check gas balances
   */
  async checkGasBalances(): Promise<any> {
    return this.client.request('GET', '/api/v1/merchant/security/gas-check');
  }

  /**
   * Get balance alerts
   */
  async getBalanceAlerts(): Promise<any> {
    return this.client.request('GET', '/api/v1/merchant/security/balance-alerts');
  }

  /**
   * Acknowledge security alert
   */
  async acknowledgeAlert(alertId: string): Promise<any> {
    return this.client.request('POST', `/api/v1/merchant/security/alerts/${alertId}/acknowledge`);
  }

  /**
   * Resolve balance alert
   */
  async resolveBalanceAlert(alertId: string): Promise<any> {
    return this.client.request('POST', `/api/v1/merchant/security/balance-alerts/${alertId}/resolve`);
  }
}
