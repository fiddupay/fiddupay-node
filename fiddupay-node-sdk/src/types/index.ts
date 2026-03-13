// Core types for FidduPay SDK

export type CryptoType = 'SOL' | 'ETH' | 'BNB' | 'MATIC' | 'ARB' | 'USDT_ETH' | 'USDT_BEP20' | 'USDT_POLYGON' | 'USDT_ARBITRUM' | 'USDT_SPL';

export type PaymentStatus = 'PENDING' | 'CONFIRMING' | 'CONFIRMED' | 'FAILED' | 'EXPIRED' | 'REFUNDED' | 'SELECTION_REQUIRED';

export type WebhookEventType =
  | 'payment.detected'
  | 'payment.confirmed'
  | 'payment.partially_paid'
  | 'payment.expired'
  | 'payment.failed'
  | 'payment.captured'
  | 'refund.created'
  | 'refund.completed'
  | 'refund.failed'
  | 'withdrawal.created'
  | 'withdrawal.processed'
  | 'wallet.low_balance';

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
  daily_volume_usd: string;
  daily_volume_limit: string;
  daily_volume_remaining: string;
  tier_level?: string;
  two_factor_enabled: boolean;
  sandbox_mode: boolean;
  settlement_mode: 'forwarding' | 'managed';
  wallets_locked: boolean;
  customer_wallets_locked: boolean;
}

export interface CreatePaymentRequest {
  amount?: string;
  amount_usd?: string;
  crypto_type?: CryptoType;
  description?: string;
  metadata?: Record<string, any>;
  expiration_minutes?: number;
  expires_in?: number; // seconds, alternative to expiration_minutes
  webhook_url?: string;
  partial_payments_enabled?: boolean;
}

export interface SelectionRequest {
  crypto_type: CryptoType;
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
  amount?: string;           // Crypto amount (optional if status is SELECTION_REQUIRED)
  amount_usd: string;
  crypto_type?: CryptoType;  // Optional if status is SELECTION_REQUIRED
  status: PaymentStatus;
  to_address?: string;       // Backend uses to_address (optional if status is SELECTION_REQUIRED)
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
  webhook_url?: string;
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

export interface AddressOnlyStats {
  total_payments: number;
  completed_payments: number;
  pending_payments: number;
  total_volume: string;
  total_fees_collected: string;
}

export interface AddressOnlyHealthStatus {
  database_healthy: boolean;
  monitoring_active: boolean;
  supported_currencies: string[];
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
}

export interface AnalyticsQuery {
  from_date?: string;
  to_date?: string;
  granularity?: 'day' | 'week' | 'month';
}

export interface UnifiedSettingsRequest {
  webhook_url?: string;
  redirect_url?: string;
  webhook_format?: 'json' | 'form';
  settlement_mode?: 'forwarding' | 'managed';
  customer_pays_fee?: boolean;
  fee_percentage?: number;
  ip_whitelist?: string[];
  sandbox_mode?: boolean;
  rotate_webhook_secret?: boolean;
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

export interface MerchantWalletBalance {
  crypto_type: string;
  network: string;
  address: string;
  is_active: boolean;
  available_balance: string;
  reserved_balance: string;
  total_balance: string;
  transaction_count: number;
  total_volume_crypto: string;
}

export interface GenerateWalletRequest {
  crypto_type: CryptoType;
  network?: string;
  is_active?: boolean;
  enable_all_evm?: boolean;
}

export interface GeneratedWallet {
  crypto_type: string;
  address: string;
  network: string;
  is_active: boolean;
}

export interface GeneratedWalletResponse {
  wallet: GeneratedWallet;
  mode: string;
  message: string;
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

export interface ProcessWithdrawalRequest {
  encryption_password?: string;
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
  crypto_type: string;
  current_balance: string;
  threshold: string;
  status: 'active' | 'resolved'; // Changed from resolved: bool to match monitoring logic
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

export interface SetLockRequest {
  locked: boolean;
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
  success: boolean;
  transaction_hash?: string;
  from_address?: string;
}

// Invoice Types
export interface InvoiceItem {
  description: string;
  quantity: number;
  unit_price: string;
  amount: string;
}

export interface CreateInvoiceRequest {
  customer_email?: string;
  customer_name?: string;
  items: InvoiceItem[];
  tax?: string;
  due_date?: string;
  notes?: string;
}

export interface Invoice {
  invoice_id: string;
  merchant_id: number;
  customer_email?: string;
  customer_name?: string;
  status: 'PENDING' | 'PAID' | 'CANCELLED';
  items: InvoiceItem[];
  subtotal: string;
  tax: string;
  total: string;
  payment_id?: string;
  due_date?: string;
  notes?: string;
  created_at: string;
  paid_at?: string;
}

// Generic Types
export interface PaginatedResponse<T> {
  data: T[];
  total: number;
  limit: number;
  offset: number;
  has_more: boolean;
}

// Merchant Customer Types (Sub-Accounts)
export interface ListCustomersParams {
  limit?: number;
  offset?: number;
}

export interface CustomerWithdrawalRequest {
  crypto_type: CryptoType;
  amount: string;
  destination_address: string;
}

export interface CustomerSweepRequest {
  crypto_type: CryptoType;
  amount?: string;
}

export interface MerchantCustomer {
  id: number;
  merchant_id: number;
  external_id: string;
  email?: string;
  metadata?: any;
  is_active: boolean;
  created_at: string;
  updated_at: string;
}

export interface CreateCustomerRequest {
  external_id: string;
  email?: string;
  metadata?: any;
}

export interface ProvisionWalletRequest {
  external_id: string;
  networks: ('evm' | 'solana')[];
}

export interface CustomerBalance {
  id: number;
  customer_id: number;
  merchant_id: number;
  crypto_type: string;
  available_balance: string;
  locked_balance: string;
  total_balance: string;
  last_updated_at: string;
}

export interface CustomerBalanceResponse {
  external_id: string;
  balances: CustomerBalance[];
}

export interface CustomerTransaction {
  id: number;
  customer_id: number;
  merchant_id: number;
  crypto_type: string;
  amount: string;
  amount_usd: string;
  tx_hash: string;
  status: string;
  created_at: string;
}

export interface CustomerStatusRequest {
  status: 'active' | 'suspended' | 'inactive';
  reason?: string;
}

export interface CustomerPermissionsRequest {
  can_withdraw?: boolean;
  withdrawal_limit?: string;
}

/**
 * Customer Internal Payment Request
 */
export interface CustomerPayMerchantRequest {
  crypto_type: string;
  amount: string;
  reference_id?: string;
  description?: string;
}

/**
 * Balance History Entry
 */
export interface BalanceHistoryEntry {
  id: number;
  crypto_type: string;
  amount: string;
  type: string;
  status: string;
  reference_id?: string;
  created_at: string;
}

/**
 * Security Monitoring Types
 */
// Consolidated with existing Security types above
