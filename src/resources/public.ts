import { HttpClient } from '../client';
import { 
  SupportedCurrenciesResponse, 
  PricingResponse, 
  SystemStatus, 
  PublicPaymentStatus, 
  CryptoType,
  PublicPaymentRequest 
} from '../types';

export class Public {
  constructor(private client: HttpClient) {}

  /**
   * Create a new payment via Publishable Key (for pure no-code frontend widgets)
   */
  async createPayment(payload: PublicPaymentRequest): Promise<{ payment_id: string; payment_url: string }> {
    return this.client.post('/api/v1/public/payments/create', payload);
  }

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

  /**
   * Get public payment status by link ID or payment ID.
   * Useful for frontend checkout pages.
   */
  async getPaymentStatus(linkId: string): Promise<PublicPaymentStatus> {
    return this.client.get<PublicPaymentStatus>(`/${linkId}/status`);
  }

  /**
   * Select a cryptocurrency for a multi-currency payment.
   */
  async finalizeSelection(linkId: string, cryptoType: CryptoType): Promise<PublicPaymentStatus> {
    return this.client.post<PublicPaymentStatus>(`/${linkId}/select`, { crypto_type: cryptoType });
  }

  /**
   * Trigger a background verification for a dynamic payment.
   */
  async triggerVerification(linkId: string): Promise<{ message: string; status: string }> {
    return this.client.post(`/${linkId}/verify`, {});
  }

  /**
   * Publicly cancel a pending payment.
   */
  async cancelPayment(paymentId: string): Promise<{ status: string; redirect_url?: string }> {
    return this.client.post(`/${paymentId}/cancel`, {});
  }
}
