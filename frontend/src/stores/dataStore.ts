/**
 * Global Data Store — SWR-style caching for all dashboard data.
 * 
 * Pattern: Show cached data instantly → background refresh if stale.
 * This prevents refetching on every sidebar navigation.
 */
import { create } from 'zustand'
import {
  publicAPI,
  merchantAPI,
  securityAPI,
  withdrawalAPI,
  walletAPI,
  customerAPI,
  paymentAPI,
} from '@/services/apiService'

// --- Cache Entry Type ---
interface CacheEntry<T> {
  data: T | null
  lastFetched: number
  loading: boolean
  error: string | null
}

function freshEntry<T>(data: T | null = null): CacheEntry<T> {
  return { data, lastFetched: 0, loading: false, error: null }
}

// TTL durations in ms
const TTL = {
  CURRENCIES: 5 * 60 * 1000,    // 5 minutes — rarely changes
  ANALYTICS: 60 * 1000,          // 60 seconds
  BALANCE: 30 * 1000,            // 30 seconds (mirrors balanceStore)
  WITHDRAWALS: 30 * 1000,        // 30 seconds
  WALLETS: 30 * 1000,            // 30 seconds
  CUSTOMERS: 60 * 1000,          // 60 seconds
  CUSTOMER_DETAILS: 30 * 1000,   // 30 seconds
  SECURITY_ALERTS: 30 * 1000,    // 30 seconds
  SECURITY_EVENTS: 30 * 1000,    // 30 seconds
  BALANCE_HISTORY: 60 * 1000,    // 60 seconds
  RECENT_ACTIVITY: 30 * 1000,    // 30 seconds
} as const

// --- State Shape ---
interface DataState {
  currencies: CacheEntry<any[]>
  analytics: CacheEntry<any> & { dateKey: string }
  withdrawals: CacheEntry<any[]>
  wallets: CacheEntry<any[]>
  customers: CacheEntry<any[]>
  customerSummary: CacheEntry<any>
  customerDetails: Record<string, CacheEntry<{ wallets: any[], balances: any, transactions: any[] }>>
  securityAlerts: CacheEntry<any[]>
  securityEvents: CacheEntry<any[]>
  balanceHistory: CacheEntry<any>
  recentActivity: CacheEntry<any[]>
}

// --- Actions ---
interface DataActions {
  fetchCurrencies: (force?: boolean) => Promise<any[]>
  fetchAnalytics: (params?: any, force?: boolean) => Promise<any>
  fetchWithdrawals: (force?: boolean) => Promise<any[]>
  fetchWallets: (force?: boolean) => Promise<any[]>
  fetchCustomers: (force?: boolean) => Promise<any[]>
  fetchCustomerSummary: (force?: boolean) => Promise<any>
  fetchCustomerDetails: (externalId: string, force?: boolean) => Promise<any>
  fetchSecurityAlerts: (force?: boolean) => Promise<any[]>
  fetchSecurityEvents: (force?: boolean) => Promise<any[]>
  fetchBalanceHistory: (params?: any, force?: boolean) => Promise<any>
  fetchRecentActivity: (force?: boolean) => Promise<any[]>
  // Direct setters for optimistic updates
  setWithdrawals: (data: any[]) => void
  setCustomers: (data: any[]) => void
  setSecurityAlerts: (data: any[]) => void
  // Invalidation
  invalidate: (key: keyof DataState) => void
  invalidateCustomerDetail: (externalId: string) => void
  invalidateAll: () => void
}

/**
 * Core SWR fetch helper.
 * - If data is cached and fresh: return immediately.
 * - If data is cached but stale: return cached, refetch in background.
 * - If no data: fetch and await.
 */
async function swrFetch<T>(
  get: () => CacheEntry<T>,
  set: (patch: Partial<CacheEntry<T>>) => void,
  apiFn: () => Promise<T>,
  ttl: number,
  force: boolean
): Promise<T> {
  const current = get()
  const now = Date.now()
  const isFresh = current.lastFetched > 0 && (now - current.lastFetched < ttl)

  // If fresh and not forced, return cached
  if (isFresh && !force && current.data !== null) {
    return current.data
  }

  // If stale but has data, return cached and refresh in background
  if (current.data !== null && !force) {
    // Don't double-fetch
    if (!current.loading) {
      set({ loading: true })
      apiFn()
        .then((data) => set({ data, lastFetched: Date.now(), loading: false, error: null }))
        .catch((err) => {
          console.warn('Background refresh failed:', err)
          set({ loading: false, error: err?.message || 'Refresh failed' })
        })
    }
    return current.data
  }

  // No cached data or forced — await the result
  if (current.loading) {
    // Already fetching, wait a bit and return whatever we have
    return current.data as T
  }

  set({ loading: true, error: null })
  try {
    const data = await apiFn()
    set({ data, lastFetched: Date.now(), loading: false, error: null })
    return data
  } catch (err: any) {
    const errorMsg = err?.message || 'Fetch failed'
    set({ loading: false, error: errorMsg })
    // Return cached data even on error (if available)
    if (current.data !== null) return current.data
    throw err
  }
}

export const useDataStore = create<DataState & DataActions>((set, get) => ({
  // --- Initial State ---
  currencies: freshEntry<any[]>([]),
  analytics: { ...freshEntry<any>(), dateKey: '' },
  withdrawals: freshEntry<any[]>([]),
  wallets: freshEntry<any[]>([]),
  customers: freshEntry<any[]>([]),
  customerSummary: freshEntry<any>(),
  customerDetails: {},
  securityAlerts: freshEntry<any[]>([]),
  securityEvents: freshEntry<any[]>([]),
  balanceHistory: freshEntry<any>(),
  recentActivity: freshEntry<any[]>([]),

  // --- Fetch Actions ---

  fetchCurrencies: async (force = false) => {
    return swrFetch(
      () => get().currencies,
      (patch) => set((s) => ({ currencies: { ...s.currencies, ...patch } })),
      async () => {
        const res = await publicAPI.getSupportedCurrencies()
        const groups = res.data?.currency_groups || {}
        return Object.values(groups).flat() as any[]
      },
      TTL.CURRENCIES,
      force
    )
  },

  fetchAnalytics: async (params?: any, force = false) => {
    const dateKey = params ? JSON.stringify(params) : 'default'
    const current = get().analytics

    // If date range changed, force refresh
    if (current.dateKey !== dateKey) {
      force = true
    }

    return swrFetch(
      () => get().analytics,
      (patch) => set((s) => ({ analytics: { ...s.analytics, ...patch, dateKey } })),
      async () => {
        const res = await merchantAPI.getAnalytics(params)
        return res.data
      },
      TTL.ANALYTICS,
      force
    )
  },

  fetchWithdrawals: async (force = false) => {
    return swrFetch(
      () => get().withdrawals,
      (patch) => set((s) => ({ withdrawals: { ...s.withdrawals, ...patch } })),
      async () => {
        const res = await withdrawalAPI.getHistory()
        return Array.isArray(res.data) ? res.data : []
      },
      TTL.WITHDRAWALS,
      force
    )
  },

  fetchWallets: async (force = false) => {
    return swrFetch(
      () => get().wallets,
      (patch) => set((s) => ({ wallets: { ...s.wallets, ...patch } })),
      async () => {
        const res = await walletAPI.getAll()
        return Array.isArray(res.data?.wallets) ? res.data.wallets : []
      },
      TTL.WALLETS,
      force
    )
  },

  fetchCustomers: async (force = false) => {
    return swrFetch(
      () => get().customers,
      (patch) => set((s) => ({ customers: { ...s.customers, ...patch } })),
      async () => {
        const res = await customerAPI.list({ limit: 1000000 })
        return res.data?.customers || []
      },
      TTL.CUSTOMERS,
      force
    )
  },

  fetchCustomerSummary: async (force = false) => {
    return swrFetch(
      () => get().customerSummary,
      (patch) => set((s) => ({ customerSummary: { ...s.customerSummary, ...patch } })),
      async () => {
        const res = await customerAPI.getSummary()
        return res.data
      },
      TTL.CUSTOMERS,
      force
    )
  },

  fetchCustomerDetails: async (externalId: string, force = false) => {
    return swrFetch(
      () => get().customerDetails[externalId] || freshEntry(),
      (patch) => set((s) => ({
        customerDetails: {
          ...s.customerDetails,
          [externalId]: { ...(s.customerDetails[externalId] || freshEntry()), ...patch }
        }
      })),
      async () => {
        const [walletRes, balRes, txRes] = await Promise.all([
          customerAPI.getWallets(externalId),
          customerAPI.getBalances(externalId),
          customerAPI.getTransactions(externalId, { limit: 20 }),
        ])
        return {
          wallets: walletRes.data?.wallets || [],
          balances: balRes.data?.balances || null,
          transactions: txRes.data?.transactions || [],
        }
      },
      TTL.CUSTOMER_DETAILS,
      force
    )
  },

  fetchSecurityAlerts: async (force = false) => {
    return swrFetch(
      () => get().securityAlerts,
      (patch) => set((s) => ({ securityAlerts: { ...s.securityAlerts, ...patch } })),
      async () => {
        const res = await securityAPI.getAlerts()
        return res.data?.alerts || []
      },
      TTL.SECURITY_ALERTS,
      force
    )
  },

  fetchSecurityEvents: async (force = false) => {
    return swrFetch(
      () => get().securityEvents,
      (patch) => set((s) => ({ securityEvents: { ...s.securityEvents, ...patch } })),
      async () => {
        const res = await securityAPI.getEvents({ limit: 50 })
        return res.data?.events || []
      },
      TTL.SECURITY_EVENTS,
      force
    )
  },

  fetchBalanceHistory: async (params?: any, force = false) => {
    return swrFetch(
      () => get().balanceHistory,
      (patch) => set((s) => ({ balanceHistory: { ...s.balanceHistory, ...patch } })),
      async () => {
        const res = await merchantAPI.getBalanceHistory(params)
        return res.data
      },
      TTL.BALANCE_HISTORY,
      force
    )
  },

  fetchRecentActivity: async (force = false) => {
    return swrFetch(
      () => get().recentActivity,
      (patch) => set((s) => ({ recentActivity: { ...s.recentActivity, ...patch } })),
      async () => {
        const res = await paymentAPI.getUnifiedTransactions({ limit: 5 })
        return res.data?.transactions || []
      },
      TTL.RECENT_ACTIVITY,
      force
    )
  },

  // --- Direct setters for optimistic updates ---
  setWithdrawals: (data) => set((s) => ({
    withdrawals: { ...s.withdrawals, data, lastFetched: Date.now() }
  })),

  setCustomers: (data) => set((s) => ({
    customers: { ...s.customers, data, lastFetched: Date.now() }
  })),

  setSecurityAlerts: (data) => set((s) => ({
    securityAlerts: { ...s.securityAlerts, data, lastFetched: Date.now() }
  })),

  // --- Invalidation ---
  invalidate: (key) => set((s) => ({
    [key]: { ...s[key], lastFetched: 0 }
  })),

  invalidateCustomerDetail: (externalId) => set((s) => {
    const next = { ...s.customerDetails }
    if (next[externalId]) {
      next[externalId] = { ...next[externalId], lastFetched: 0 }
    }
    return { customerDetails: next }
  }),

  invalidateAll: () => set({
    currencies: freshEntry<any[]>([]),
    analytics: { ...freshEntry<any>(), dateKey: '' },
    withdrawals: freshEntry<any[]>([]),
    wallets: freshEntry<any[]>([]),
    customers: freshEntry<any[]>([]),
    customerSummary: freshEntry<any>(),
    customerDetails: {},
    securityAlerts: freshEntry<any[]>([]),
    securityEvents: freshEntry<any[]>([]),
    balanceHistory: freshEntry<any>(),
  }),
}))
