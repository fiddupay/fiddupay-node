// Core types for FidduPay SDK

export type CryptoType = 'SOL' | 'ETH' | 'BNB' | 'MATIC' | 'ARB' | 'USDT_ETH' | 'USDT_BEP20' | 'USDT_POLYGON' | 'USDT_ARBITRUM' | 'USDT_SPL' | 'BTC' | 'BUSD_BEP20' | 'WSOL' | 'USDC_ETH' | 'USDC_SOL' | 'USDC_POLYGON';

export type PaymentStatus = 'PENDING' | 'CONFIRMING' | 'CONFIRMED' | 'FAILED' | 'EXPIRED' | 'REFUNDED' | 'SELECTION_REQUIRED' | 'CANCELLED';

export type WebhookFormat = 'json' | 'discord' | 'slack';

export type WebhookEventType =
  | 'payment.confirmed'
  | 'payment.expired'
  | 'refund.completed'
  | 'merchant.deposit'
  | 'customer.deposit'
  | 'address_only_payment_status'
  | 'webhook.test';

export interface FidduPayConfig {
  apiKey: string;
  apiVersion?: string;
  timeout?: number;
  maxRetries?: number;
  baseURL?: string;
}

export interface MerchantRegistrationRequest {
  email: string;
  business_name: string;
  password: string;
  // Step 1 KYC
  first_name: string;
  last_name: string;
  gender: string;
  phone_number: string;
  country: string;
  applicant_role: string;
  terms_accepted: boolean;
  // Step 2 Business
  business_country: string;
  business_license_number: string;
  business_certificate_url?: string;
  website_url: string;
  // Optional Compliance
  nin_bvn?: string;
  twitter_handle?: string;
  instagram_handle?: string;
}

export interface SystemStatus {
  overall_status: 'operational' | 'degraded' | 'outage' | 'initializing';
  services: ServiceStatus[];
  uptime_stats: UptimeStats;
  last_updated: string;
  system_metrics?: SystemMetrics;
  past_incidents: SystemIncident[];
}

export interface ServiceStatus {
  name: string;
  description: string;
  status: 'operational' | 'degraded' | 'outage';
  response_time?: number;
  last_check: string;
  history: UptimePoint[];
}

export interface UptimePoint {
  date: string;
  status: 'operational' | 'degraded' | 'outage';
}

export interface UptimeStats {
  seven_days: number;
  fourteen_days: number;
  thirty_days: number;
}

export interface SystemMetrics {
  cpu_usage: number;
  memory_usage_percent: number;
}

export interface SystemIncident {
  id: string;
  title: string;
  description: string;
  status: string;
  severity: string;
  created_at: string;
  resolved_at?: string;
}

export interface LoginRequest {
  email: string;
  password: string;
  two_factor_code?: string;
  remember_me?: boolean;
}

export interface LoginResponse {
  user: MerchantProfile;
  dashboard_token: string;
}

export type MerchantRole = 'MERCHANT' | 'ADMIN' | 'MODERATOR' | 'SUPER_ADMIN' | 'USER';

export interface TrustScore {
  score: number;
  tier: string;
  identity_verified: boolean;
  social_verified: boolean;
  business_verified: boolean;
}

export interface MerchantProfile {
  id: number;
  business_name: string;
  email: string;
  role?: MerchantRole;
  created_at: string;
  kyc_verified: boolean;
  daily_limit_usd: string | null;
  daily_volume_remaining: string;
  tier_level?: string;
  kyc_tier?: number;
  compliance_status?: string;
  two_factor_enabled: boolean;
  sandbox_mode: boolean;
  settlement_mode: 'forwarding' | 'managed';
  wallets_locked: boolean;
  customer_wallets_locked: boolean;
  webhook_url?: string;
  webhook_format?: WebhookFormat;
  redirect_url?: string;
  api_key?: string;
  has_transaction_pin: boolean;
  pin_setup_at?: string;
  low_balance_threshold_usd: string;
  low_balance_alerts_enabled: boolean;
  webhook_signing_secret?: string;
  last_login_at?: string;
  username?: string;
  pay_id?: string;
  has_national_id: boolean;
  social_handles?: Record<string, any>;
  managed_mode_only?: boolean;
  withdrawal_enabled?: boolean;
  trust_score?: TrustScore;
  withdrawal_fee_percentage?: number;
  fee_percentage: string;
  customer_pays_fee: boolean;
  business_license_number?: string;
  business_certificate_url?: string;
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

  // Invoice-specific fields (Supported by backend)
  is_invoice?: boolean;
  customer_name?: string;
  customer_email?: string;
  customer_external_id?: string;
  items?: InvoiceItem[];
  tax?: string;
  due_date?: string;
  notes?: string;
}

export interface PublicPaymentRequest {
  publishable_key: string;
  amount?: string;
  amount_usd?: string;
  crypto_type?: CryptoType;
  description?: string;
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
  fee_percentage?: string;
  fee_amount?: string;
  fee_amount_usd?: string;
  webhook_url?: string;
  customer_external_id?: string;
  partial_payments_enabled?: boolean;
  total_paid?: string;
  remaining_balance?: string;
  is_non_custodial?: boolean;
  block_number?: number;
  sandbox_mode?: boolean;
  partial_payments?: Record<string, any>;
  last_verification_at?: string;
}
/**
 * Address-Only Mode Types
 */
export interface AddressOnlyPaymentResponse {
  payment_id: string;
  gateway_deposit_address: string;
  requested_amount: string;
  customer_amount: string;
  processing_fee: string;
  customer_pays_fee: boolean;
  customer_instructions: string;
  supported_currencies: string[];
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
  forwarding_amount?: string;
  merchant_destination_address?: string;
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
  page?: number;
  page_size?: number;
  status?: PaymentStatus;
  crypto_type?: CryptoType;
  blockchain?: string;
  from_date?: string;
  to_date?: string;
  [key: string]: any;
}

export interface PaginationInfo {
  page: number;
  page_size: number;
  total_pages: number;
  total_count: number;
}

export interface ListPaymentsResponse {
  data: Payment[];
  pagination: PaginationInfo;
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
  status?: string;
  blockchain?: string;
  format?: 'csv' | 'json' | 'xlsx';
}

export interface UnifiedSettingsRequest {
  webhook_url?: string;
  redirect_url?: string;
  webhook_format?: WebhookFormat;
  settlement_mode?: 'forwarding' | 'managed';
  customer_pays_fee?: boolean;
  fee_percentage?: number;
  withdrawal_fee_percentage?: number;
  ip_whitelist?: string[];
  sandbox_mode?: boolean;
  rotate_webhook_secret?: boolean;
  low_balance_threshold_usd?: string;
  low_balance_alerts_enabled?: boolean;
  webhook_signing_secret?: string;

  // Sandbox Flags
  solana_sandbox_enabled?: boolean;
  bnb_sandbox_enabled?: boolean;
  eth_sandbox_enabled?: boolean;
  matic_sandbox_enabled?: boolean;
  arb_sandbox_enabled?: boolean;
  btc_sandbox_enabled?: boolean;
}

export interface MerchantSettingsUpdateResponse {
  status: string;
  message: string;
  new_webhook_secret?: string;
}

export interface Merchant {
  merchant_id: string;
  email: string;
  business_name: string;
  status: 'pending_verification' | 'verified' | 'suspended';
  role?: MerchantRole;
  balance: {
    available_usd: string;
    pending_usd: string;
  };
  low_balance_alerts_enabled?: boolean;
  low_balance_threshold_usd?: string;
  created_at: string;
  verified_at?: string;
}

export interface MerchantReadiness {
  is_ready: boolean;
  missing_steps: string[];
  settlement_mode: string;
  sandbox_mode: boolean;
}

export interface WebhookEvent {
  id: string;
  type: WebhookEventType;
  data: Payment | Refund;
  created_at: string;
}

export interface TimeSeriesPoint {
  date: string;
  volume_usd: string;
  count: number;
}

export interface BlockchainStats {
  volume_usd: string;
  payment_count: number;
  average_value: string;
}

export interface Analytics {
  total_volume_usd: string;
  successful_payments: number;
  failed_payments: number;
  pending_payments: number;
  total_payments: number;
  total_fees_paid: string;
  average_transaction_value: string;
  by_blockchain: Record<string, BlockchainStats>;
  payment_trends: TimeSeriesPoint[];
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
  fee_percentage: number;
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
  available_usd?: string;
  reserved_balance: string;
  reserved_usd?: string;
  total_balance: string;
  total_usd?: string;
  balance_usd?: string; // Legacy compatibility
  transaction_count: number;
  total_volume_crypto: string;
  total_volume_usd?: string;
}

export interface GenerateWalletRequest {
  crypto_type: CryptoType;
  network?: string;
  is_active?: boolean;
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
  pin: string;
}

export interface ProcessWithdrawalRequest {
  encryption_password: string;
}

export interface ApproveWithdrawalRequest {
  approved: boolean;
  rejection_reason?: string;
}

export interface Withdrawal {
  withdrawal_id: string;
  crypto_type: CryptoType;
  amount: string;
  amount_usd: string;
  destination_address: string;
  status: 'PENDING' | 'PROCESSING' | 'COMPLETED' | 'FAILED' | 'CANCELLED' | 'REJECTED';
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
  [key: string]: any;
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
  password?: string; // Required by backend for verification
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
export interface BalanceEntry {
  crypto_type: string; // Changed to string for flexibility on some endpoints
  available_balance: string;
  available_usd: string;
  reserved_balance: string;
  reserved_usd: string;
  total_balance: string;
  total_usd: string;
  last_updated?: string;
}

export interface BalanceSummary {
  available_usd: string;
  reserved_usd: string;
  total_usd: string;
  balances: BalanceEntry[];
}

export type Balance = BalanceSummary; // The main balance endpoint now returns a summary

export interface NotificationActionResult {
  status: string;
  affected: number;
}

export interface BalanceTrendPoint {
  date: string;
  total_usd: string;
  balances: Record<string, string>; // crypto_type -> amount
}

export interface BalanceHistory {
  points: BalanceTrendPoint[];
}

export interface UnifiedTransaction {
  type: 'payment' | 'refund' | 'withdrawal' | 'deposit' | 'sweep' | string;
  id: string;
  crypto_amount: string;
  usd_amount: string;
  crypto_type: string;
  status: string;
  transaction_hash?: string;
  created_at: string;
}

export interface UnifiedTransactionsResponse {
  transactions: UnifiedTransaction[];
}

export interface ListBalanceHistoryParams {
  limit?: number;
}

export interface AuditLog {
  id: number;
  merchant_id?: number;
  action_type: string;
  entity_type?: string;
  entity_id?: string;
  ip_address?: string;
  user_agent?: string;
  details?: Record<string, any>;
  created_at: string;
}

export interface ListAuditLogsParams {
  from?: string;
  to?: string;
  action_type?: string;
  limit?: number;
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
  currency?: string;
  due_date?: string;
  notes?: string;
}

export interface Invoice {
  id: number;
  invoice_id: string;
  merchant_id: number;
  customer_email?: string;
  customer_name?: string;
  status: 'PENDING' | 'PAID' | 'CANCELLED';
  items: InvoiceItem[];
  subtotal: string;
  tax: string;
  total: string;
  currency: string;
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
  [key: string]: any;
}

export interface CustomerSweepRequest {
  sweep_mode: 'ALL' | 'NATIVE_ONLY' | 'STABLE_ONLY' | 'SPECIFIC';
  crypto_types?: CryptoType[];
  amount?: string;
  pin: string;
}

export interface MerchantCustomer {
  id: number;
  merchant_id: number;
  external_id: string;
  email?: string;
  first_name?: string;
  last_name?: string;
  metadata?: any;
  is_active: boolean;
  status: string;
  status_reason?: string;
  can_withdraw: boolean;
  withdrawal_limit?: string;
  created_at: string;
  updated_at: string;
}

export interface CreateCustomerRequest {
  external_id: string;
  email?: string;
  first_name?: string;
  last_name?: string;
  metadata?: any;
}

export interface ProvisionWalletRequest {
  networks: ('evm' | 'solana')[];
}

export interface BulkProvisionRequest {
  /** Array of customer external_ids to provision wallets for. Omit if using all_customers. */
  customer_ids?: string[];
  /** Set to true to provision wallets for ALL customers. Overrides customer_ids. */
  all_customers?: boolean;
}

export interface BulkProvisionResponse {
  count: number;
  message: string;
}

export interface CustomerBalance {
  id: number;
  customer_id: number;
  merchant_id: number;
  crypto_type: string;
  available_balance: string;
  available_balance_usd?: string; // Added to match backend
  locked_balance: string;
  locked_balance_usd?: string; // Added to match backend
  total_balance: string;
  total_balance_usd?: string;
  last_updated_at: string;
  sandbox_mode?: boolean;
}

export interface CustomerBalanceResponse {
  external_id: string;
  balances: CustomerBalance[];
}

export interface CustomerWallet {
  crypto_type: string;
  network: string;
  address: string;
  created_at?: string;
  sandbox_mode?: boolean;
}

export interface CustomerWalletsResponse {
  external_id: string;
  wallets: CustomerWallet[];
}

export interface WalletBalancesResponse {
  wallets: BalanceEntry[];
}

export interface CustomerTransaction {
  id: number;
  customer_id: number;
  merchant_id: number;
  type: string; // WITHDRAWAL, MERCHANT_PAYMENT, SWEEP
  crypto_type: string;
  amount: string;
  amount_usd: string;
  fee: string;
  status: string;
  destination_address?: string;
  transaction_hash?: string;
  reference_id?: string;
  description?: string;
  created_at: string;
  updated_at: string;
  sandbox_mode: boolean;
}

export interface CustomerStatusRequest {
  status: 'active' | 'flagged' | 'suspended' | 'blocked';
  reason?: string;
}

export interface CustomerPermissionsRequest {
  can_withdraw?: boolean;
  withdrawal_limit?: string;
}

export interface CustomerSummaryResponse {
  total_customers: number;
  active_customers: number;
  flagged_customers: number;
  recent_customers: number;
  total_balance_usd: number;
}

/**
 * Public Endpoint Types
 */
export interface SupportedCurrencyItem {
  crypto_type: string;
  network: string;
  icon_url?: string;
  confirmations: number;
  price_usd: number;
}

export interface SupportedCurrenciesResponse {
  currency_groups: Record<string, SupportedCurrencyItem[]>;
  description: string;
}

export interface PublicPaymentStatus {
  status: PaymentStatus;
  payment_id: string;
  link_id: string;
  amount_usd: string;
  crypto_type?: CryptoType;
  deposit_address?: string;
  amount_crypto?: string;
  expires_at: string;
  is_expired: boolean;
  redirect_url?: string;
  merchant_name?: string;
  merchant_logo_url?: string;
}

export interface PricingResponse {
  transaction_fee_percentage: string;
  daily_volume_limit_non_kyc_usd: string;
  supported_networks: number;
  supported_cryptocurrencies: string[];
  features: {
    instant_settlements: boolean;
    real_time_notifications: boolean;
    webhook_support: boolean;
    sandbox_testing: boolean;
    api_access: boolean;
    dashboard_analytics: boolean;
  };
  limits: {
    kyc_verified: {
      daily_volume_limit: string;
      transaction_limit: string;
    };
    non_kyc: {
      daily_volume_limit: string;
      transaction_limit: string;
    };
  };
}





/**
 * Notification Types
 */
export interface MerchantNotification {
  id: string;
  title: string;
  message: string;
  notification_type: string;
  event_type: string;
  is_read: boolean;
  sandbox_mode: boolean;
  created_at: string;
  expires_at?: string;
}

export interface NotificationListResponse {
  notifications: MerchantNotification[];
  total: number;
  unread_count: number;
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
 * Address-Only Fee Setting Types
 */
export interface AddressOnlyFeeSettingResponse {
  customer_pays_fee: boolean;
  description: string;
}

export interface UpdateAddressOnlyFeeSettingRequest {
  customer_pays_fee: boolean;
}
