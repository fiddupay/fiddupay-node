import { create } from 'zustand';

// Types
export interface UserProfile {
    id: number;
    nickname: string;
    kyc_level: number;
    is_vendor: boolean;
    completion_rate: number;
    thumbs_up_count: number;
    thumbs_down_count: number;
    total_trades: number;
}

export interface P2PBalance {
    id: number;
    crypto_type: string;
    available_balance: number;
    locked_balance: number;
    total_balance: number;
}

interface AppState {
    user: UserProfile | null;
    balances: P2PBalance[];
    isAuthenticated: boolean;
    activeTradeId: string | null;

    // Actions
    setUser: (profile: UserProfile | null) => void;
    setBalances: (balances: P2PBalance[]) => void;
    setActiveTradeId: (id: string | null) => void;
    logout: () => void;
}

// 1. Core State Store
export const useAppStore = create<AppState>((set) => ({
    user: null,
    balances: [],
    isAuthenticated: false,
    activeTradeId: null,

    setUser: (profile) => set({ user: profile, isAuthenticated: !!profile }),
    setBalances: (balances) => set({ balances }),
    setActiveTradeId: (id) => set({ activeTradeId: id }),
    logout: () => set({ user: null, balances: [], isAuthenticated: false, activeTradeId: null })
}));

// 2. Chat/WebSocket Store (Separated for performance)
interface ChatState {
    isConnected: boolean;
    notifications: string[];
    addNotification: (msg: string) => void;
    setConnected: (status: boolean) => void;
}

export const useChatStore = create<ChatState>((set) => ({
    isConnected: false,
    notifications: [],
    addNotification: (msg) => set((state) => ({ notifications: [...state.notifications, msg] })),
    setConnected: (status) => set({ isConnected: status }),
}));
