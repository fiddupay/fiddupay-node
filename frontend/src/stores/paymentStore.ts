import { create } from 'zustand'
import { Payment, PaymentData, PaymentFilters, PaginatedResponse } from '@/types'
import apiService from '@/services/api'

interface PaymentState {
  payments: Payment[]
  currentPayment: Payment | null
  loading: boolean
  error: string | null
  pagination: {
    page: number
    pageSize: number
    totalPages: number
    totalCount: number
  }
  filters: PaymentFilters
}

interface PaymentActions {
  fetchPayments: (filters?: PaymentFilters) => Promise<void>
  createPayment: (data: PaymentData) => Promise<Payment>
  getPayment: (id: string) => Promise<void>
  setFilters: (filters: PaymentFilters) => void
  clearError: () => void
  resetState: () => void
}

const initialState: PaymentState = {
  payments: [],
  currentPayment: null,
  loading: false,
  error: null,
  pagination: {
    page: 1,
    pageSize: 10,
    totalPages: 0,
    totalCount: 0,
  },
  filters: {},
}

export const usePaymentStore = create<PaymentState & PaymentActions>((set, get) => ({
  ...initialState,

  fetchPayments: async (filters?: PaymentFilters) => {
    try {
      set({ loading: true, error: null })
      
      const currentFilters = { ...get().filters, ...filters }
      const response = await apiService.getPayments(currentFilters)
      
      set({
        payments: response.data,
        pagination: response.pagination,
        filters: currentFilters,
        loading: false,
      })
    } catch (error: any) {
      set({
        error: error.response?.data?.error || 'Failed to fetch payments',
        loading: false,
      })
    }
  },

  createPayment: async (data: PaymentData) => {
    try {
      set({ loading: true, error: null })
      const payment = await apiService.createPayment(data)
      
      // Add new payment to the beginning of the list
      set((state) => ({
        payments: [payment, ...state.payments],
        loading: false,
      }))
      
      return payment
    } catch (error: any) {
      set({
        error: error.response?.data?.error || 'Failed to create payment',
        loading: false,
      })
      throw error
    }
  },

  getPayment: async (id: string) => {
    try {
      set({ loading: true, error: null })
      const payment = await apiService.getPayment(id)
      
      set({
        currentPayment: payment,
        loading: false,
      })
    } catch (error: any) {
      set({
        error: error.response?.data?.error || 'Failed to fetch payment',
        loading: false,
      })
    }
  },

  setFilters: (filters: PaymentFilters) => {
    set({ filters })
  },

  clearError: () => {
    set({ error: null })
  },

  resetState: () => {
    set(initialState)
  },
}))
