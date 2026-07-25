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
  RequestOptions,
  SelectionRequest,
  AddressOnlyStats,
  AddressOnlyHealthStatus
} from '../types';
import { FidduPayValidationError } from '../errors';

export class Payments {
  constructor(private client: HttpClient) { }

  /**
   * Create a new payment
   */
  async create(data: CreatePaymentRequest, options?: RequestOptions): Promise<Payment> {
    this.validateCreatePayment(data);
    return this.client.request<Payment>('POST', '/api/v1/merchants/payments', data);
  }

  /**
   * Finalize currency selection for a multi-currency payment
   */
  async finalizeSelection(paymentId: string, data: SelectionRequest, options?: RequestOptions): Promise<Payment> {
    if (!paymentId) {
      throw new FidduPayValidationError('Payment ID is required', 'payment_id');
    }
    if (!data.crypto_type) {
      throw new FidduPayValidationError('Crypto type is required', 'crypto_type');
    }
    return this.client.post<Payment>(`/api/v1/merchants/payments/${paymentId}/select`, data, options);
  }

  /**
   * Retrieve a payment by ID
   */
  async retrieve(paymentId: string, options?: RequestOptions): Promise<Payment> {
    if (!paymentId) {
      throw new FidduPayValidationError('Payment ID is required', 'payment_id');
    }
    return this.client.get<Payment>(`/api/v1/merchants/payments/${paymentId}`, options);
  }

  /**
   * Verify a payment with transaction hash
   */
  async verify(paymentId: string, data: {
    transaction_hash: string
  }, options?: RequestOptions): Promise<{ confirmed: boolean }> {
    if (!paymentId) {
      throw new FidduPayValidationError('Payment ID is required', 'payment_id');
    }
    return this.client.request('POST', `/api/v1/merchants/payments/${paymentId}/verify`, data);
  }

  /**
   * Trigger a background verification for a dynamic payment.
   * This is a lightweight alternative to manually providing a transaction hash.
   */
  async triggerVerification(paymentId: string, options?: RequestOptions): Promise<{ status: string }> {
    if (!paymentId) {
      throw new FidduPayValidationError('Payment ID is required', 'payment_id');
    }
    return this.client.post(`/${paymentId}/verify`, {}, options);
  }

  /**
   * List payments with optional filters
   */
  async list(params?: ListPaymentsRequest, options?: RequestOptions): Promise<ListPaymentsResponse> {
    const queryParams = new URLSearchParams();

    if (params) {
      for (const [key, value] of Object.entries(params)) {
        if (value !== undefined && value !== null) {
          queryParams.append(key, value.toString());
        }
      }
    }

    const query = queryParams.toString();
    const path = query ? `/api/v1/merchants/payments?${query}` : '/api/v1/merchants/payments';

    return this.client.request<ListPaymentsResponse>('GET', path);
  }

  /**
   * Cancel a pending payment
   */
  async cancel(paymentId: string, options?: RequestOptions): Promise<Payment> {
    if (!paymentId) {
      throw new FidduPayValidationError('Payment ID is required', 'payment_id');
    }
    return this.client.post<Payment>(`/api/v1/merchants/payments/${paymentId}/cancel`, {}, options);
  }

  /**
   * Create an address-only payment
   */
  async createAddressOnly(data: CreateAddressOnlyPaymentRequest, options?: RequestOptions): Promise<AddressOnlyPayment> {
    this.validateCreateAddressOnlyPayment(data);
    return this.client.post<AddressOnlyPayment>('/api/v1/merchants/address-only/create', data, options);
  }

  /**
   * Retrieve an address-only payment status by ID
   */
  async retrieveAddressOnly(paymentId: string, options?: RequestOptions): Promise<AddressOnlyPayment> {
    if (!paymentId) {
      throw new FidduPayValidationError('Payment ID is required', 'payment_id');
    }
    return this.client.get<AddressOnlyPayment>(`/api/v1/merchants/address-only/status?payment_id=${paymentId}`, options);
  }

  /**
   * List supported native currencies for address-only mode
   */
  async listAddressOnlyCurrencies(options?: RequestOptions): Promise<string[]> {
    return this.client.get<string[]>('/api/v1/merchants/address-only/currencies', options);
  }

  /**
   * Get address-only mode statistics
   */
  async getAddressOnlyStats(options?: RequestOptions): Promise<AddressOnlyStats> {
    return this.client.get<AddressOnlyStats>('/api/v1/merchants/address-only/stats', options);
  }

  /**
   * Get address-only mode health status
   */
  async getAddressOnlyHealth(options?: RequestOptions): Promise<AddressOnlyHealthStatus> {
    return this.client.get<AddressOnlyHealthStatus>('/api/v1/merchants/address-only/health', options);
  }

  /**
   * Update fee setting (customer pays fee vs merchant pays fee)
   */
  async updateFeeSetting(data: UpdateFeeSettingRequest, options?: RequestOptions): Promise<UpdateFeeSettingResponse> {
    if (typeof data.customer_pays_fee !== 'boolean') {
      throw new FidduPayValidationError('customer_pays_fee must be a boolean', 'customer_pays_fee');
    }
    return this.client.put<UpdateFeeSettingResponse>('/api/v1/merchants/address-only/fee-setting', data, options);
  }

  async getFeeSetting(options?: RequestOptions): Promise<FeeSettingResponse> {
    return this.client.get<FeeSettingResponse>('/api/v1/merchants/address-only/fee-setting', options);
  }

  private validateCreatePayment(data: CreatePaymentRequest): void {
    // Validate that either amount or amount_usd is provided, but not both
    if (data.amount && data.amount_usd) {
      throw new FidduPayValidationError('Provide either amount or amount_usd, not both', 'amount');
    }

    if (!data.amount && !data.amount_usd) {
      throw new FidduPayValidationError('Either amount or amount_usd must be provided', 'amount');
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

    // crypto_type is optional for multi-currency checkout
    if (data.crypto_type) {
      const validCryptoTypes = ['SOL', 'ETH', 'BNB', 'MATIC', 'ARB', 'USDT_ETH', 'USDT_BEP20', 'USDT_POLYGON', 'USDT_ARBITRUM', 'USDT_SPL', 'BTC', 'BUSD_BEP20', 'WSOL', 'USDC_ETH', 'USDC_SOL', 'USDC_POLYGON'];
      if (!validCryptoTypes.includes(data.crypto_type)) {
        throw new FidduPayValidationError(
          `Invalid crypto type. Must be one of: ${validCryptoTypes.join(', ')}`,
          'crypto_type'
        );
      }
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

    const validCryptoTypes = ['SOL', 'ETH', 'BNB', 'MATIC', 'ARB', 'USDT_ETH', 'USDT_BEP20', 'USDT_POLYGON', 'USDT_ARBITRUM', 'USDT_SPL', 'BTC', 'BUSD_BEP20', 'WSOL', 'USDC_ETH', 'USDC_SOL', 'USDC_POLYGON'];
    if (!validCryptoTypes.includes(data.crypto_type)) {
      throw new FidduPayValidationError(
        `Invalid crypto type. Must be one of: ${validCryptoTypes.join(', ')}`,
        'crypto_type'
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
