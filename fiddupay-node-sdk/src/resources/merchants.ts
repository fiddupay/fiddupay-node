import { HttpClient } from '../client';
import {
  AuditLog,
  Balance,
  BalanceHistory,
  LoginRequest,
  LoginResponse,
  Merchant,
  MerchantProfile,
  MerchantReadiness,
  MerchantRegistrationRequest,
  MerchantSettingsUpdateResponse,
  RequestOptions,
  SystemStatus,
  UnifiedSettingsRequest,
} from '../types';

export class Merchants {
  constructor(private client: HttpClient) {}

  /**
   * Register new merchant
   */
  async register(
    data: MerchantRegistrationRequest
  ): Promise<LoginResponse> {
    return this.client.request('POST', '/api/v1/merchants/register', data);
  }

  /**
   * Login merchant (public endpoint - no auth required)
   */
  async login(data: LoginRequest): Promise<LoginResponse> {
    return this.client.request('POST', '/api/v1/merchants/login', data);
  }

  /**
   * Get current merchant profile
   */
  async retrieve(options?: RequestOptions): Promise<MerchantProfile> {
    const response = await this.client.request<{ user: MerchantProfile }>('GET', '/api/v1/merchants/profile');
    return response.user;
  }

  /**
   * Get merchant readiness status (Checks KYC, wallet setup, etc.)
   */
  async getReadiness(options?: RequestOptions): Promise<MerchantReadiness> {
    return this.client.get('/api/v1/merchants/status', options);
  }

  /**
   * Get merchant readiness status (legacy)
   * @deprecated Use getReadiness() instead
   */
  async getStatus(options?: RequestOptions): Promise<any> {
    return this.getReadiness(options);
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
   * Get current fee setting
   */
  async getFeeSetting(options?: RequestOptions): Promise<{ fee_percentage: number; customer_pays_fee: boolean }> {
    return this.client.request('GET', '/api/v1/merchants/fee-setting');
  }


  /**
   * Update global merchant settings (Unified)
   */
  async updateSettings(data: UnifiedSettingsRequest, options?: RequestOptions): Promise<MerchantSettingsUpdateResponse> {
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
  
  /**
   * Get IP whitelist for the merchant
   */
  async getIpWhitelist(options?: RequestOptions): Promise<{ ip_whitelist: string[] }> {
    return this.client.request('GET', '/api/v1/merchants/ip-whitelist');
  }

  /**
   * Get merchant balance
   */
  async getBalance(options?: RequestOptions): Promise<Balance> {
    return this.client.request<Balance>('GET', '/api/v1/merchants/balance');
  }

  /**
   * Get audit logs for the merchant
   */
  async getAuditLogs(params?: {
    limit?: number;
    action_type?: string;
    from?: string;
    to?: string;
  }, options?: RequestOptions): Promise<AuditLog[]> {
    const queryParams = new URLSearchParams();
    if (params?.limit) queryParams.append('limit', params.limit.toString());
    if (params?.action_type) queryParams.append('action_type', params.action_type);
    if (params?.from) queryParams.append('from', params.from);
    if (params?.to) queryParams.append('to', params.to);

    const query = queryParams.toString();
    const path = query ? `/api/v1/merchants/audit-logs?${query}` : '/api/v1/merchants/audit-logs';
    return this.client.request('GET', path);
  }

  /**
   * Get balance history for the merchant
   */
  async getBalanceHistory(params?: {
    limit?: number;
  }, options?: RequestOptions): Promise<BalanceHistory> {
    const queryParams = new URLSearchParams();
    if (params?.limit) queryParams.append('limit', params.limit.toString());

    const query = queryParams.toString();
    const path = query ? `/api/v1/merchants/balance/history?${query}` : '/api/v1/merchants/balance/history';
    return this.client.request('GET', path);
  }

  // ============================================================================
  // Public Endpoints (no auth required)
  // ============================================================================

  /**
   * Get supported currencies (public endpoint)
   */
  async getSupportedCurrencies(merchantId?: number): Promise<{ currency_groups: any; description: string }> {
    const query = merchantId ? `?merchant_id=${merchantId}` : '';
    return this.client.request('GET', `/api/v1/currencies/supported${query}`);
  }

  /**
   * Get pricing information (public endpoint)
   */
  async getPricing(): Promise<any> {
    return this.client.request('GET', '/api/v1/pricing');
  }

  /**
   * Get system status (public endpoint)
   */
  async getSystemStatus(): Promise<SystemStatus> {
    return this.client.request<SystemStatus>('GET', '/api/v1/status');
  }
}
