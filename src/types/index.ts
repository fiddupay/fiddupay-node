// Core types for FidduPay SDK

export type CryptoType = 'SOL' | 'ETH' | 'BNB' | 'MATIC' | 'ARB' | 'USDT_ETH' | 'USDT_BSC' | 'USDT_POLYGON' | 'USDT_ARBITRUM' | 'USDT_SPL';

export type PaymentStatus = 'PENDING' | 'CONFIRMING' | 'CONFIRMED' | 'FAILED' | 'EXPIRED';

export type WebhookEventType = 
  | 'payment.confirmed' 
  | 'payment.expired' 
  | 'payment.failed' 
  | 'refund.completed' 
  | 'refund.failed';

export interface FidduPayConfig {
  apiKey: string;
  environment?: 'sandbox' | 'production';
  apiVersion?: string;
  timeout?: number;
  maxRetries?: number;
  baseURL?: string;
}

export interface CreatePaymentRequest {
  amount_usd: string;
  crypto_type: CryptoType;
  description?: string;
  metadata?: Record<string, any>;
  expiration_minutes?: number;
  webhook_url?: string;
}

export interface CreateAddressOnlyPaymentRequest {
  requested_amount: string;
  crypto_type: CryptoType;
  merchant_address: string;
  description?: string;
  metadata?: Record<string, any>;
  expiration_minutes?: number;
  webhook_url?: string;
}

export interface Payment {
  payment_id: string;
  amount_usd: string;
  crypto_amount: string;
  crypto_type: CryptoType;
  status: PaymentStatus;
  deposit_address: string;
  transaction_hash?: string;
  confirmations?: number;
  created_at: string;
  confirmed_at?: string;
  expires_at: string;
  description?: string;
  metadata?: Record<string, any>;
  payment_link?: string;
  qr_code_data?: string;
}

export interface AddressOnlyPayment {
  payment_id: string;
  requested_amount: string;
  customer_amount: string;
  processing_fee: string;
  crypto_type: CryptoType;
  gateway_deposit_address: string;
  customer_pays_fee: boolean;
  customer_instructions: string;
  supported_currencies: string[];
  expires_at?: string;
  status?: PaymentStatus;
  transaction_hash?: string;
  confirmations?: number;
  created_at?: string;
  confirmed_at?: string;
  description?: string;
  metadata?: Record<string, any>;
}

export interface ListPaymentsRequest {
  limit?: number;
  offset?: number;
  status?: PaymentStatus;
  crypto_type?: CryptoType;
}

export interface ListPaymentsResponse {
  payments: Payment[];
  total: number;
  has_more: boolean;
}

export interface CreateRefundRequest {
  payment_id: string;
  amount?: string;
  reason?: string;
}

export interface Refund {
  refund_id: string;
  payment_id: string;
  status: 'PROCESSING' | 'COMPLETED' | 'FAILED';
  amount: string;
  amount_usd: string;
  crypto_type: CryptoType;
  refund_address: string;
  reason?: string;
  created_at: string;
  processed_at?: string;
  transaction_hash?: string;
}

export interface Merchant {
  merchant_id: string;
  email: string;
  business_name: string;
  status: 'pending_verification' | 'verified' | 'suspended';
  balance: {
    available_usd: string;
    pending_usd: string;
  };
  created_at: string;
  verified_at?: string;
}

export interface WebhookEvent {
  id: string;
  type: WebhookEventType;
  data: Payment | Refund;
  created_at: string;
}

export interface Analytics {
  period: {
    start_date: string;
    end_date: string;
    granularity: 'day' | 'week' | 'month';
  };
  summary: {
    total_payments: number;
    total_volume_usd: string;
    successful_payments: number;
    failed_payments: number;
    success_rate: number;
    average_payment_usd: string;
  };
  data: Array<{
    date: string;
    payments: number;
    volume_usd: string;
    success_rate: number;
  }>;
}

export interface RequestOptions {
  timeout?: number;
  retries?: number;
  idempotencyKey?: string;
}

// Fee Toggle Types
export interface UpdateFeeSettingRequest {
  customer_pays_fee: boolean;
}

export interface FeeSettingResponse {
  customer_pays_fee: boolean;
  description: string;
}

export interface UpdateFeeSettingResponse {
  success: boolean;
  customer_pays_fee: boolean;
  message: string;
}
