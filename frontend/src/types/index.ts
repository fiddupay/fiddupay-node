// Authentication Types
export interface User {
  id: number
  business_name: string
  email: string
  created_at: string
  two_factor_enabled: boolean
}

export interface LoginCredentials {
  email: string
  password: string
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
  status: 'PENDING' | 'CONFIRMED' | 'FAILED' | 'EXPIRED'
  amount: string
  amount_usd: string
  crypto_type: string
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

export interface PaymentData {
  amount_usd: string
  crypto_type: string
  description?: string
}

export interface PaymentFilters {
  status?: string
  crypto_type?: string
  start_date?: string
  end_date?: string
  page?: number
  page_size?: number
}

// Wallet Types
export interface WalletConfig {
  crypto_type: string
  address: string
}

export interface Wallet {
  crypto_type: string
  address: string
  configured_at: string
}

// Analytics Types
export interface Analytics {
  total_payments: number
  total_volume_usd: string
  successful_payments: number
  pending_payments: number
  failed_payments: number
  average_payment_usd: string
  payment_trends: PaymentTrend[]
  currency_breakdown: CurrencyBreakdown[]
}

export interface PaymentTrend {
  date: string
  count: number
  volume_usd: string
}

export interface CurrencyBreakdown {
  crypto_type: string
  count: number
  volume_usd: string
  percentage: number
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
  amount: string
  amount_usd: string
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
