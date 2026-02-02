// Core types for FidduPay SDK

export type CryptoType = 'SOL' | 'ETH' | 'BNB' | 'MATIC' | 'ARB' | 'USDT_ETH' | 'USDT_BEP20' | 'USDT_POLYGON' | 'USDT_ARBITRUM' | 'USDT_SPL';

export type PaymentStatus = 'PENDING' | 'CONFIRMING' | 'CONFIRMED' | 'FAILED' | 'EXPIRED' | 'REFUNDED';

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
  daily_limit_usd?: string;
  tier_level?: string;
  two_factor_enabled: boolean;
}

export interface CreatePaymentRequest {
  amount?: string;
  amount_usd?: string;
  crypto_type: CryptoType;
  description?: string;
  metadata?: Record<string, any>;
  expiration_minutes?: number;
  expires_in?: number; // seconds, alternative to expiration_minutes
  webhook_url?: string;
  partial_payments_enabled?: boolean;
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
  amount: string;           // Crypto amount
  amount_usd: string;
  crypto_type: CryptoType;
  status: PaymentStatus;
  to_address: string;       // Backend uses to_address
  deposit_address?: string; // Also included for convenience
  transaction_hash?: string;
  from_address?: string;
  confirmations: number;
  required_confirmations: number;
  created_at: string;
  confirmed_at?: string;
  expires_at: string;
  description?: string;
  metadata?: Record<string, any>;
  network?: string;
  payment_link?: string;
  qr_code_data?: string;
  fee_amount?: string;
  fee_amount_usd?: string;
  partial_payments?: Record<string, any>;
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
  amount: string;
  amount_usd: string;
  status: 'pending' | 'completed' | 'failed';
  crypto_type?: CryptoType;
  target_address?: string;
  reason?: string;
  transaction_hash?: string;
  created_at: string;
  completed_at?: string;
  // Fields present in some contexts but not always response
  redundant_field?: never; // Placeholder to clear lines
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
  id: number;
  merchant_id: number;
  crypto_type: CryptoType;
  network: string;
  address: string;
  is_active: boolean;
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
  address: string; // Changed from wallet_address to match backend and resource
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
  status: 'PENDING' | 'PROCESSING' | 'COMPLETED' | 'FAILED' | 'CANCELLED';
  fee: string;
  net_amount: string;
  transaction_hash?: string;
  rejection_reason?: string;
  requires_approval: boolean;
  approved_by?: number;
  approved_at?: string;
  completed_at?: string;
  created_at: string;
  updated_at: string;
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
  severity: string; // 'low' | 'medium' | 'high' | 'critical'; - backend uses String but values match
  description: string;
  ip_address?: string;
  user_agent?: string;
  created_at: string;
}

export interface SecurityAlert {
  alert_id: string;
  alert_type: string;
  severity: string;
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
  from_address?: string;
}

// Generic Types
export interface PaginatedResponse<T> {
  data: T[];
  total: number;
  limit: number;
  offset: number;
  has_more: boolean;
}
