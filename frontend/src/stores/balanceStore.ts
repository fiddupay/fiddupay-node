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

let inFlightBalancePromise: Promise<void> | null = null

export const useBalanceStore = create<BalanceState & BalanceActions>((set, get) => ({
  balance: null,
  loading: false,
  lastUpdated: null,

  fetchBalance: async (force = false) => {
    const now = Date.now()
    const { lastUpdated, balance } = get()
    
    // If fresh, not forced, and balance exists: return cached balance
    if (!force && lastUpdated && (now - lastUpdated < 30000) && balance !== null) {
      return
    }

    // Return in-flight promise if a request is already running
    if (inFlightBalancePromise) {
      return inFlightBalancePromise
    }

    set({ loading: true })
    inFlightBalancePromise = (async () => {
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
      } finally {
        inFlightBalancePromise = null
      }
    })()

    return inFlightBalancePromise
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
