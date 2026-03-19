import { HttpClient } from '../client';
import { CreateAddressOnlyPaymentRequest, AddressOnlyPaymentResponse, AddressOnlyPayment, AddressOnlyStats, RequestOptions } from '../types';

export class AddressOnly {
  constructor(private client: HttpClient) { }

  /**
   * Create an address-only payment request
   */
  async createPayment(data: CreateAddressOnlyPaymentRequest, options?: RequestOptions): Promise<AddressOnlyPaymentResponse> {
    return this.client.request('POST', '/api/v1/merchants/address-only/create', data);
  }

  /**
   * Get address-only payment status
   */
  async getStatus(paymentId: string, options?: RequestOptions): Promise<AddressOnlyPayment> {
    return this.client.request('GET', `/api/v1/merchants/address-only/status?payment_id=${paymentId}`);
  }

  /**
   * Get supported native currencies
   */
  async getCurrencies(options?: RequestOptions): Promise<string[]> {
    return this.client.request('GET', '/api/v1/merchants/address-only/currencies');
  }

  /**
   * Get address-only mode statistics
   */
  async getStats(options?: RequestOptions): Promise<AddressOnlyStats> {
    return this.client.request('GET', '/api/v1/merchants/address-only/stats');
  }

  /**
   * Get address-only mode health status
   */
  async getHealth(options?: RequestOptions): Promise<any> {
    return this.client.request('GET', '/api/v1/merchants/address-only/health');
  }

  /**
   * Update merchant fee payment setting
   */
  async updateFeeSetting(customerPaysFee: boolean, options?: RequestOptions): Promise<{ success: boolean; message: string; customer_pays_fee: boolean }> {
    return this.client.request('PUT', '/api/v1/merchants/address-only/fee-setting', { customer_pays_fee: customerPaysFee });
  }

  /**
   * Get merchant fee setting
   */
  async getFeeSetting(options?: RequestOptions): Promise<{ customer_pays_fee: boolean; description: string }> {
    return this.client.request('GET', '/api/v1/merchants/address-only/fee-setting');
  }
}
