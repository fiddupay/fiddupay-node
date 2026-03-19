import { HttpClient } from '../client';
import { SupportedCurrenciesResponse, PricingResponse, SystemStatus } from '../types';

export class Public {
  constructor(private client: HttpClient) {}

  /**
   * Get supported currencies and networks
   * @param params optional filter for merchant specific settings
   */
  async getCurrencies(params?: { merchant_id?: number }): Promise<SupportedCurrenciesResponse> {
    const queryParams = new URLSearchParams();
    if (params?.merchant_id) queryParams.append('merchant_id', params.merchant_id.toString());
    
    const url = `/api/v1/currencies/supported${queryParams.toString() ? `?${queryParams.toString()}` : ''}`;
    return this.client.get<SupportedCurrenciesResponse>(url);
  }

  /**
   * Get platform pricing and volume limits
   */
  async getPricing(): Promise<PricingResponse> {
    return this.client.get<PricingResponse>('/api/v1/pricing');
  }

  /**
   * Get system health status and uptime
   */
  async getStatus(): Promise<SystemStatus> {
    return this.client.get<SystemStatus>('/api/v1/status');
  }
}
