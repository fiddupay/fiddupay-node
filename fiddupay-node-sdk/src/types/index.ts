// Core types for FidduPay SDK

export type CryptoType = 'SOL' | 'ETH' | 'BNB' | 'MATIC' | 'ARB' | 'USDT_ETH' | 'USDT_BEP20' | 'USDT_POLYGON' | 'USDT_ARBITRUM' | 'USDT_SPL';

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

export interface MerchantProfile {
  id: number;
  business_name: string;
  email: string;
  created_at: string;
  kyc_verified: boolean;
  daily_volume_remaining?: string; // Only present for non-KYC merchants
  two_factor_enabled: boolean;
}

export interface CreatePaymentRequest {
  amount?: string;
  amount_usd?: string;
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

// Wallet Management Types
export interface WalletConfig {
  crypto_type: CryptoType;
  wallet_address: string;
  has_private_key: boolean;
  can_withdraw: boolean;
  created_at: string;
  updated_at: string;
}

export interface GenerateWalletRequest {
  crypto_type: CryptoType;
}

export interface ImportWalletRequest {
  crypto_type: CryptoType;
  private_key: string;
}

export interface ExportKeyRequest {
  crypto_type: CryptoType;
}

export interface ConfigureAddressRequest {
  crypto_type: CryptoType;
  wallet_address: string;
}

export interface GasEstimate {
  crypto_type: CryptoType;
  estimated_gas_fee: string;
  gas_price: string;
  gas_limit: number;
}

export interface WithdrawalCapability {
  crypto_type: CryptoType;
  can_withdraw: boolean;
  reason?: string;
  gas_balance?: string;
  minimum_gas_required?: string;
}

// Withdrawal Types
export interface CreateWithdrawalRequest {
  crypto_type: CryptoType;
  amount: string;
  destination_address: string;
}

export interface Withdrawal {
  withdrawal_id: string;
  crypto_type: CryptoType;
  amount: string;
  destination_address: string;
  status: 'pending' | 'processing' | 'completed' | 'failed' | 'cancelled';
  transaction_hash?: string;
  created_at: string;
  processed_at?: string;
  fee_amount?: string;
}

export interface ListWithdrawalsParams {
  limit?: number;
  offset?: number;
  status?: string;
  crypto_type?: CryptoType;
}

// Security Types
export interface SecurityEvent {
  event_id: string;
  event_type: string;
  severity: 'low' | 'medium' | 'high' | 'critical';
  description: string;
  ip_address?: string;
  user_agent?: string;
  created_at: string;
}

export interface SecurityAlert {
  alert_id: string;
  alert_type: string;
  severity: 'low' | 'medium' | 'high' | 'critical';
  message: string;
  acknowledged: boolean;
  acknowledged_at?: string;
  created_at: string;
}

export interface BalanceAlert {
  alert_id: string;
  crypto_type: CryptoType;
  current_balance: string;
  threshold: string;
  resolved: boolean;
  resolved_at?: string;
  created_at: string;
}

export interface SecuritySettings {
  enable_notifications: boolean;
  alert_thresholds: {
    low_balance: string;
    failed_transactions: number;
  };
  ip_whitelist_enabled: boolean;
  two_factor_enabled: boolean;
}

export interface UpdateSecuritySettingsRequest {
  enable_notifications?: boolean;
  alert_thresholds?: {
    low_balance?: string;
    failed_transactions?: number;
  };
}

export interface ListSecurityEventsParams {
  limit?: number;
  offset?: number;
  event_type?: string;
}

export interface ListSecurityAlertsParams {
  limit?: number;
  offset?: number;
  severity?: string;
}

// Balance Types
export interface Balance {
  balances: Record<CryptoType, {
    available: string;
    pending: string;
    total: string;
  }>;
  total_usd: string;
}

export interface BalanceHistory {
  transaction_id: string;
  crypto_type: CryptoType;
  amount: string;
  type: 'credit' | 'debit';
  description: string;
  created_at: string;
}

export interface ListBalanceHistoryParams {
  limit?: number;
  offset?: number;
  crypto_type?: CryptoType;
}

// Audit Log Types
export interface AuditLog {
  log_id: string;
  action: string;
  resource_type: string;
  resource_id: string;
  details: Record<string, any>;
  ip_address?: string;
  user_agent?: string;
  created_at: string;
}

export interface ListAuditLogsParams {
  limit?: number;
  offset?: number;
  action?: string;
  start_date?: string;
  end_date?: string;
}

// Sandbox Types
export interface SandboxPaymentSimulation {
  payment_id: string;
  simulated_status: PaymentStatus;
  transaction_hash?: string;
  message: string;
}

export interface SimulatePaymentRequest {
  status: 'completed' | 'failed';
  transaction_hash?: string;
}

// Generic Types
export interface PaginatedResponse<T> {
  data: T[];
  total: number;
  limit: number;
  offset: number;
  has_more: boolean;
}
