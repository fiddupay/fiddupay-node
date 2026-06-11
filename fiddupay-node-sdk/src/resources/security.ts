import { HttpClient } from '../client';
import {
  BalanceAlert,
  ListSecurityAlertsParams,
  ListSecurityEventsParams,
  RequestOptions,
  SecurityAlert,
  SecurityEvent,
  SecuritySettings,
  UpdatePasswordRequest,
  GasBalancesResponse
} from '../types';

export class Security {
  constructor(private client: HttpClient) { }

  /**
   * Get security events
   */
  async getEvents(params?: ListSecurityEventsParams, options?: RequestOptions): Promise<SecurityEvent[]> {
    const queryParams = new URLSearchParams();
    if (params?.limit) queryParams.append('limit', params.limit.toString());
    if (params?.offset) queryParams.append('offset', params.offset.toString());
    if (params?.event_type) queryParams.append('event_type', params.event_type);

    const query = queryParams.toString();
    const path = query ? `/api/v1/merchants/security/events?${query}` : '/api/v1/merchants/security/events';
    return this.client.get<SecurityEvent[]>(path, options);
  }

  /**
   * Get security alerts
   */
  async getAlerts(params?: ListSecurityAlertsParams, options?: RequestOptions): Promise<SecurityAlert[]> {
    const queryParams = new URLSearchParams();
    if (params?.limit) queryParams.append('limit', params.limit.toString());
    if (params?.offset) queryParams.append('offset', params.offset.toString());
    if (params?.severity) queryParams.append('severity', params.severity);

    const query = queryParams.toString();
    const path = query ? `/api/v1/merchants/security/alerts?${query}` : '/api/v1/merchants/security/alerts';
    return this.client.get<SecurityAlert[]>(path, options);
  }

  /**
   * Get security settings
   */
  async getSettings(options?: RequestOptions): Promise<SecuritySettings> {
    return this.client.get<SecuritySettings>('/api/v1/merchants/security/settings', options);
  }

  /**
   * Update security settings
   */
  async updateSettings(data: Partial<SecuritySettings>, options?: RequestOptions): Promise<SecuritySettings> {
    return this.client.put<SecuritySettings>('/api/v1/merchants/security/settings', data, options);
  }

  async checkGasBalances(options?: RequestOptions): Promise<GasBalancesResponse> {
    return this.client.request<GasBalancesResponse>('GET', '/api/v1/merchants/security/gas-check');
  }

  /**
   * Get balance alerts
   */
  async getBalanceAlerts(options?: RequestOptions): Promise<BalanceAlert[]> {
    return this.client.get<BalanceAlert[]>('/api/v1/merchants/security/balance-alerts', options);
  }

  /**
   * Acknowledge security alert
   */
  async acknowledgeAlert(alertId: string, options?: RequestOptions): Promise<{ status: string }> {
    return this.client.post<{ status: string }>(`/api/v1/merchants/security/alerts/${alertId}/acknowledge`, {}, options);
  }

  /**
   * Resolve balance alert
   */
  async resolveBalanceAlert(alertId: string, options?: RequestOptions): Promise<{ status: string }> {
    return this.client.post<{ status: string }>(`/api/v1/merchants/security/balance-alerts/${alertId}/resolve`, {}, options);
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
  async setTransactionPin(pin: string, options?: RequestOptions): Promise<{ message: string }> {
    return this.client.post('/api/v1/merchants/security/transaction-pin', { pin }, options);
  }

  /**
   * Verify the merchant's Transaction PIN
   */
  async verifyTransactionPin(pin: string, options?: RequestOptions): Promise<{ valid: boolean }> {
    return this.client.post('/api/v1/merchants/security/transaction-pin/verify', { pin }, options);
  }

  /**
   * Update the merchant's account password
   */
  async updatePassword(data: UpdatePasswordRequest, options?: RequestOptions): Promise<{ message: string }> {
    return this.client.post('/api/v1/merchants/security/password', data, options);
  }
}
