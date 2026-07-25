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

/**
 * Address-Only Mode Resource
 *
 * Handles direct-to-address payments and management for self-managed wallets.
 */
export class AddressOnly {
  constructor(private client: HttpClient) {}

  /**
   * Create an address-only payment request
   * @param data Request details
   */
  async create(data: CreateAddressOnlyPaymentRequest): Promise<AddressOnlyPaymentResponse> {
    return this.client.request<AddressOnlyPaymentResponse>(
      'POST',
      '/api/v1/merchants/address-only/create',
      data
    );
  }

  /**
   * Get status of an address-only payment
   * @param paymentId The payment ID
   */
  async getStatus(paymentId: string): Promise<AddressOnlyPayment> {
    return this.client.request<AddressOnlyPayment>(
      'GET',
      `/api/v1/merchants/address-only/status?payment_id=${paymentId}`
    );
  }

  /**
   * List supported native currencies for address-only mode
   */
  async getCurrencies(): Promise<string[]> {
    return this.client.request<string[]>('GET', '/api/v1/merchants/address-only/currencies');
  }

  /**
   * Get address-only mode statistics for the merchant
   */
  async getStats(): Promise<AddressOnlyStats> {
    return this.client.request<AddressOnlyStats>('GET', '/api/v1/merchants/address-only/stats');
  }

  /**
   * Get address-only fee setting
   */
  async getFeeSetting(): Promise<AddressOnlyFeeSettingResponse> {
    return this.client.request<AddressOnlyFeeSettingResponse>(
      'GET',
      '/api/v1/merchants/address-only/fee-setting'
    );
  }

  /**
   * Update address-only fee setting
   * @param customerPaysFee Whether the customer pays the fee
   */
  async updateFeeSetting(customerPaysFee: boolean): Promise<AddressOnlyFeeSettingResponse> {
    return this.client.request<AddressOnlyFeeSettingResponse>(
      'PUT',
      '/api/v1/merchants/address-only/fee-setting',
      { customer_pays_fee: customerPaysFee } as UpdateAddressOnlyFeeSettingRequest
    );
  }

  /**
   * Get address-only mode health status
   */
  async getHealth(): Promise<AddressOnlyHealthStatus> {
    return this.client.request<AddressOnlyHealthStatus>(
      'GET',
      '/api/v1/merchants/address-only/health'
    );
  }
}
