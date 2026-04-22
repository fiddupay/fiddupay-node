import { create } from 'zustand'
import { merchantAPI } from '@/services/apiService'
import { Balance } from '@/types'

interface BalanceState {
  balance: Balance | null
  loading: boolean
  lastUpdated: number | null
}

interface BalanceActions {
  fetchBalance: (force?: boolean) => Promise<void>
  updateBalance: (newBalance: Balance) => void
  clearBalance: () => void
}

export const useBalanceStore = create<BalanceState & BalanceActions>((set, get) => ({
  balance: null,
  loading: false,
  lastUpdated: null,

  fetchBalance: async (force = false) => {
    // Basic cache logic: if loaded in last 30s, don't refetch unless forced
    const now = Date.now()
    const { lastUpdated, loading } = get()
    
    if (!force && lastUpdated && (now - lastUpdated < 30000)) {
      return
    }

    if (loading) return

    set({ loading: true })
    try {
      const res = await merchantAPI.getBalance()
      set({ 
        balance: res.data, 
        loading: false,
        lastUpdated: Date.now()
      })
    } catch (error) {
      console.error('Failed to fetch balance:', error)
      set({ loading: false })
    }
  },

  updateBalance: (newBalance) => {
    set({ 
      balance: newBalance,
      lastUpdated: Date.now() 
    })
  },

  clearBalance: () => {
    set({ balance: null, lastUpdated: null })
  }
}))
