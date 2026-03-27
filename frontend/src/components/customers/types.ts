export interface Customer {
  id: string;
  merchant_id: string;
  external_id: string;
  email?: string;
  first_name?: string;
  last_name?: string;
  is_active: boolean;
  status: string;
  status_reason?: string;
  can_withdraw: boolean;
  withdrawal_limit?: string;
  created_at: string;
}

export interface Wallet {
  crypto_type: string;
  network: string;
  address: string;
  created_at: string;
}

export interface CustomerTx {
  id: string;
  type: string;
  crypto_type: string;
  amount: string;
  fee: string;
  status: string;
  destination_address?: string;
  transaction_hash?: string;
  reference_id?: string;
  description?: string;
  created_at: string;
  amount_usd?: string;
}

export const STATUS_STYLES: Record<string, { color: string; bg: string; icon: string; label: string }> = {
  active: {
    color: "#059669",
    bg: "#d1fae5",
    icon: "fa-check-circle",
    label: "Active",
  },
  flagged: {
    color: "#d97706",
    bg: "#fef3c7",
    icon: "fa-flag",
    label: "Flagged",
  },
  suspended: {
    color: "#dc2626",
    bg: "#fee2e2",
    icon: "fa-pause-circle",
    label: "Suspended",
  },
  blocked: {
    color: "#6b7280",
    bg: "#f3f4f6",
    icon: "fa-ban",
    label: "Blocked",
  },
};

export const TX_BADGES: Record<string, { color: string; bg: string; icon: string }> = {
  WITHDRAWAL: { color: "#dc2626", bg: "#fee2e2", icon: "fa-arrow-up" },
  MERCHANT_PAYMENT: {
    color: "#7c3aed",
    bg: "#ede9fe",
    icon: "fa-shopping-cart",
  },
  SWEEP: { color: "#2563eb", bg: "#dbeafe", icon: "fa-broom" },
  DEPOSIT: { color: "#059669", bg: "#d1fae5", icon: "fa-arrow-down" },
};
