import { HttpClient } from '../client';
import {
  CreatePaymentRequest,
  CreateAddressOnlyPaymentRequest,
  Payment,
  AddressOnlyPayment,
  ListPaymentsRequest,
  ListPaymentsResponse,
  UpdateFeeSettingRequest,
  FeeSettingResponse,
  UpdateFeeSettingResponse,
  RequestOptions
} from '../types';
import { FidduPayValidationError } from '../errors';

export class Payments {
  constructor(private client: HttpClient) {}

  /**
   * Create a new payment
   */
  async create(data: CreatePaymentRequest, options?: RequestOptions): Promise<Payment> {
    this.validateCreatePayment(data);
    return this.client.request<Payment>('POST', '/api/v1/payments', data);
  }

  /**
   * Retrieve a payment by ID
   */
  async retrieve(paymentId: string, options?: RequestOptions): Promise<Payment> {
    if (!paymentId) {
      throw new FidduPayValidationError('Payment ID is required', 'payment_id');
    }
    return this.client.get<Payment>(`/api/v1/payments/${paymentId}`, options);
  }

  /**
   * Verify a payment with transaction hash
   */
  async verify(paymentId: string, data: { 
    transaction_hash: string 
  }, options?: RequestOptions): Promise<any> {
    if (!paymentId) {
      throw new FidduPayValidationError('Payment ID is required', 'payment_id');
    }
    return this.client.request('POST', `/api/v1/payments/${paymentId}/verify`, data);
  }

  /**
   * List payments with optional filters
   */
  async list(params?: ListPaymentsRequest, options?: RequestOptions): Promise<ListPaymentsResponse> {
    const queryParams = new URLSearchParams();
    
    if (params?.limit) queryParams.append('limit', params.limit.toString());
    if (params?.offset) queryParams.append('offset', params.offset.toString());
    if (params?.status) queryParams.append('status', params.status);
    if (params?.crypto_type) queryParams.append('crypto_type', params.crypto_type);

    const query = queryParams.toString();
    const path = query ? `/api/v1/payments?${query}` : '/api/v1/payments';
    
    return this.client.request<ListPaymentsResponse>('GET', path);
  }

  /**
   * Cancel a pending payment
   */
  async cancel(paymentId: string, options?: RequestOptions): Promise<Payment> {
    if (!paymentId) {
      throw new FidduPayValidationError('Payment ID is required', 'payment_id');
    }
    return this.client.post<Payment>(`/payments/${paymentId}/cancel`, {}, options);
  }

  /**
   * Create an address-only payment
   */
  async createAddressOnly(data: CreateAddressOnlyPaymentRequest, options?: RequestOptions): Promise<AddressOnlyPayment> {
    this.validateCreateAddressOnlyPayment(data);
    return this.client.post<AddressOnlyPayment>('/address-only-payments', data, options);
  }

  /**
   * Retrieve an address-only payment by ID
   */
  async retrieveAddressOnly(paymentId: string, options?: RequestOptions): Promise<AddressOnlyPayment> {
    if (!paymentId) {
      throw new FidduPayValidationError('Payment ID is required', 'payment_id');
    }
    return this.client.get<AddressOnlyPayment>(`/address-only-payments/${paymentId}`, options);
  }

  /**
   * Update fee setting (customer pays fee vs merchant pays fee)
   */
  async updateFeeSetting(data: UpdateFeeSettingRequest, options?: RequestOptions): Promise<UpdateFeeSettingResponse> {
    if (typeof data.customer_pays_fee !== 'boolean') {
      throw new FidduPayValidationError('customer_pays_fee must be a boolean', 'customer_pays_fee');
    }
    return this.client.post<UpdateFeeSettingResponse>('/fee-setting', data, options);
  }

  /**
   * Get current fee setting
   */
  async getFeeSetting(options?: RequestOptions): Promise<FeeSettingResponse> {
    return this.client.get<FeeSettingResponse>('/fee-setting', options);
  }

  private validateCreatePayment(data: CreatePaymentRequest): void {
    // Validate that either amount or amount_usd is provided, but not both
    if (data.amount && data.amount_usd) {
      throw new FidduPayValidationError('Provide either amount or amount_usd, not both', 'amount');
    }

    if (!data.amount && !data.amount_usd) {
      throw new FidduPayValidationError('Either amount or amount_usd must be provided', 'amount');
    }

    if (!data.crypto_type) {
      throw new FidduPayValidationError('Crypto type is required', 'crypto_type');
    }

    // Validate the provided amount (either amount or amount_usd)
    const amountValue = data.amount || data.amount_usd;
    const amount = parseFloat(amountValue!);
    if (isNaN(amount) || amount <= 0) {
      throw new FidduPayValidationError('Amount must be a positive number', data.amount ? 'amount' : 'amount_usd');
    }

    if (amount < 0.01) {
      throw new FidduPayValidationError('Minimum amount is $0.01', data.amount ? 'amount' : 'amount_usd');
    }

    // Note: No maximum amount limit - server enforces daily volume limits based on KYC status

    const validCryptoTypes = ['SOL', 'ETH', 'BNB', 'MATIC', 'ARB', 'USDT_ETH', 'USDT_BSC', 'USDT_POLYGON', 'USDT_ARBITRUM', 'USDT_SPL'];
    if (!validCryptoTypes.includes(data.crypto_type)) {
      throw new FidduPayValidationError(
        `Invalid crypto type. Must be one of: ${validCryptoTypes.join(', ')}`,
        'crypto_type'
      );
    }

    if (data.expiration_minutes !== undefined) {
      if (data.expiration_minutes < 5 || data.expiration_minutes > 1440) {
        throw new FidduPayValidationError(
          'Expiration must be between 5 and 1440 minutes',
          'expiration_minutes'
        );
      }
    }

    if (data.description && data.description.length > 500) {
      throw new FidduPayValidationError(
        'Description must be 500 characters or less',
        'description'
      );
    }
  }

  private validateCreateAddressOnlyPayment(data: CreateAddressOnlyPaymentRequest): void {
    if (!data.requested_amount) {
      throw new FidduPayValidationError('Requested amount is required', 'requested_amount');
    }

    if (!data.crypto_type) {
      throw new FidduPayValidationError('Crypto type is required', 'crypto_type');
    }

    if (!data.merchant_address) {
      throw new FidduPayValidationError('Merchant address is required', 'merchant_address');
    }

    const amount = parseFloat(data.requested_amount);
    if (isNaN(amount) || amount <= 0) {
      throw new FidduPayValidationError('Requested amount must be a positive number', 'requested_amount');
    }

    if (amount < 0.01) {
      throw new FidduPayValidationError('Minimum amount is $0.01', 'requested_amount');
    }

    // Note: No maximum amount limit - server enforces daily volume limits based on KYC status

    const validCryptoTypes = ['SOL', 'ETH', 'BNB', 'MATIC', 'ARB', 'USDT_ETH', 'USDT_BSC', 'USDT_POLYGON', 'USDT_ARBITRUM', 'USDT_SPL'];
    if (!validCryptoTypes.includes(data.crypto_type)) {
      throw new FidduPayValidationError(
        `Invalid crypto type. Must be one of: ${validCryptoTypes.join(', ')}`,
        'crypto_type'
      );
    }

    if (data.expiration_minutes !== undefined) {
      if (data.expiration_minutes < 5 || data.expiration_minutes > 1440) {
        throw new FidduPayValidationError(
          'Expiration must be between 5 and 1440 minutes',
          'expiration_minutes'
        );
      }
    }

    if (data.description && data.description.length > 500) {
      throw new FidduPayValidationError(
        'Description must be 500 characters or less',
        'description'
      );
    }

    // Basic address validation
    if (data.merchant_address.length < 10) {
      throw new FidduPayValidationError(
        'Invalid merchant address format',
        'merchant_address'
      );
    }
  }
}
