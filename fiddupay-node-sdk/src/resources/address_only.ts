import { HttpClient } from '../client';
import {
  CreateAddressOnlyPaymentRequest,
  AddressOnlyPaymentResponse,
  AddressOnlyPayment,
  AddressOnlyStats,
  AddressOnlyHealthStatus,
  AddressOnlyFeeSettingResponse,
  UpdateAddressOnlyFeeSettingRequest,
} from '../types';

export class AddressOnly {
  constructor(private client: HttpClient) {}

  /**
   * Create an address-only payment
   */
  async create(data: CreateAddressOnlyPaymentRequest): Promise<AddressOnlyPaymentResponse> {
    return this.client.request<AddressOnlyPaymentResponse>('POST', '/api/v1/merchants/address-only/create', data);
  }

  /**
   * Get payment status
   */
  async getStatus(paymentId: string): Promise<AddressOnlyPayment> {
    return this.client.request<AddressOnlyPayment>('GET', `/api/v1/merchants/address-only/status?payment_id=${paymentId}`);
  }

  /**
   * Get supported native currencies
   */
  async getSupportedCurrencies(): Promise<string[]> {
    return this.client.request<string[]>('GET', '/api/v1/merchants/address-only/currencies');
  }

  /**
   * Get address-only mode statistics
   */
  async getStats(): Promise<AddressOnlyStats> {
    return this.client.request<AddressOnlyStats>('GET', '/api/v1/merchants/address-only/stats');
  }

  /**
   * Get fee setting (who pays the processing fee)
   */
  async getFeeSetting(): Promise<AddressOnlyFeeSettingResponse> {
    return this.client.request<AddressOnlyFeeSettingResponse>('GET', '/api/v1/merchants/address-only/fee-setting');
  }

  /**
   * Update fee setting (toggle who pays the processing fee)
   * 
   * @param data - The fee setting update request
   * @returns Success status with the updated setting
   */
  async updateFeeSetting(data: UpdateAddressOnlyFeeSettingRequest): Promise<{ success: boolean; message: string; customer_pays_fee: boolean }> {
    return this.client.request('PUT', '/api/v1/merchants/address-only/fee-setting', data);
  }

  /**
   * Get address-only mode health status
   * 
   * Returns database health, monitoring status, and supported currencies.
   */
  async getHealth(): Promise<AddressOnlyHealthStatus> {
    return this.client.request<AddressOnlyHealthStatus>('GET', '/api/v1/merchants/address-only/health');
  }
}
