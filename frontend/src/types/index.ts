// Authentication Types
export interface User {
  id: number
  business_name: string
  email: string
  created_at: string
  two_factor_enabled: boolean
  kyc_verified: boolean
  daily_volume_remaining: string
  daily_limit_usd?: string
  sandbox_mode: boolean
  settlement_mode: 'forwarding' | 'managed'
  webhook_url?: string
  webhook_format?: string
  redirect_url?: string
  api_key?: string
  live_publishable_key?: string
  test_publishable_key?: string
  wallets_locked: boolean
  customer_wallets_locked: boolean
  ip_whitelist?: string[]
  low_balance_alerts_enabled?: boolean
  low_balance_threshold_usd?: string
  has_transaction_pin: boolean
  pin_setup_at?: string
}

export interface SecurityEvent {
  id: string
  merchant_id: string
  action_type: string
  description: string
  ip_address: string
  user_agent?: string
  created_at: string
}

export interface SecurityAlert {
  id: string
  merchant_id: string
  type: string
  severity: 'low' | 'medium' | 'high' | 'critical'
  message: string
  is_acknowledged: boolean
  acknowledged_at?: string
  created_at: string
}

export interface BalanceAlert {
  id: string
  merchant_id: string
  crypto_type: string
  current_balance: string
  threshold_balance: string
  status: 'PENDING' | 'RESOLVED'
  resolved_at?: string
  created_at: string
}


export interface LoginCredentials {
  email: string
  password: string
  remember_me?: boolean
  two_factor_code?: string
}

export interface RegisterData {
  business_name: string
  email: string
  password: string
}

// Payment Types
export interface Payment {
  payment_id: string
  status: 'PENDING' | 'CONFIRMED' | 'FAILED' | 'EXPIRED' | 'SELECTION_REQUIRED' | 'CANCELLED'
  amount?: string
  amount_usd: string
  crypto_type?: string
  network: string
  deposit_address: string
  payment_link: string
  qr_code_data: string
  fee_amount: string
  fee_amount_usd: string
  expires_at: string
  created_at: string
  confirmed_at?: string
  transaction_hash?: string
  description?: string
}

export interface AddressOnlyPayment {
  payment_id: string
  requested_amount: string
  customer_amount: string
  processing_fee: string
  crypto_type: string
  gateway_deposit_address: string
  customer_pays_fee: boolean
  customer_instructions: string
  supported_currencies: string[]
  expires_at?: string
  status?: 'PENDING' | 'CONFIRMED' | 'FAILED' | 'EXPIRED' | 'SELECTION_REQUIRED' | 'CANCELLED'
  transaction_hash?: string
  confirmations?: number
  created_at?: string
  confirmed_at?: string
  description?: string
  metadata?: Record<string, any>
}

export interface PaymentData {
  amount_usd?: string  // USD-based payment
  amount?: string      // Crypto-based payment
  crypto_type?: string
  description?: string
  webhook_url?: string
  metadata?: Record<string, any>
  expires_in?: number
  expiration_minutes?: number
  partial_payments_enabled?: boolean
}

export interface AddressOnlyPaymentData {
  requested_amount: string
  crypto_type: string
  merchant_address: string
  description?: string
}

export interface FeeSettingData {
  customer_pays_fee: boolean
}

export interface FeeSettingResponse {
  fee_percentage: number
  customer_pays_fee?: boolean
}

export interface PaymentFilters {
  status?: string
  crypto_type?: string
  blockchain?: string
  start_date?: string
  end_date?: string
  from_date?: string // Added for consistency with analytics
  to_date?: string   // Added for consistency with analytics
  page?: number
  page_size?: number
}

// Wallet Types
export interface WalletConfig {
  crypto_type: string
  address: string
  is_active?: boolean
}

export interface Wallet {
  crypto_type: string
  address: string
  is_active: boolean
  configured_at: string
  updated_at?: string
}

// Analytics Types
export interface Analytics {
  total_volume_usd: string
  successful_payments: number
  failed_payments: number
  pending_payments: number
  total_payments: number
  total_fees_paid: string
  average_transaction_value: string
  by_blockchain: Record<string, BlockchainStats>
  payment_trends: TimeSeriesPoint[]
}

export interface TimeSeriesPoint {
  date: string
  count: number
  volume_usd: string
}

export interface BlockchainStats {
  volume_usd: string
  payment_count: number
  average_value: string
}

// Balance Types
export interface Balance {
  total_usd: string
  available_usd: string
  reserved_usd: string
  balances: CurrencyBalance[]
}

export interface CurrencyBalance {
  crypto_type: string
  total_balance: string
  available_balance: string
  reserved_balance: string
  balance_usd: string
  total_usd: string
  available_usd: string
  reserved_usd: string
  last_updated: string
}

// Balance History Types
export interface BalanceTrendPoint {
  date: string
  total_usd: string
  balances: Record<string, string> // crypto_type -> amount
}

export interface BalanceHistory {
  points: BalanceTrendPoint[]
}

// Withdrawal Types
export interface Withdrawal {
  withdrawal_id: string
  status: 'PENDING' | 'APPROVED' | 'COMPLETED' | 'FAILED'
  amount: string
  crypto_type: string
  destination_address: string
  fee_amount: string
  net_amount: string
  transaction_hash?: string
  created_at: string
  completed_at?: string
}

export interface WithdrawalData {
  amount: string
  crypto_type: string
  destination_address: string
}

// API Response Types
export interface ApiResponse<T> {
  success: boolean
  data?: T
  error?: string
  message?: string
}

export interface PaginatedResponse<T> {
  data: T[]
  pagination: {
    page: number
    page_size: number
    total_pages: number
    total_count: number
  }
}
export interface SelectionRequest {
  crypto_type: string
}
