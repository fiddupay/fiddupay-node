/**
 * Real-Time Cache Revalidator — WebSocket Edition
 *
 * Connects to the backend WebSocket at /api/v1/merchants/ws (already implemented,
 * backed by Redis Pub/Sub). The server pushes a lightweight JSON event ONLY when
 * data actually changes. We map each event type to the exact cache key(s) that
 * need refreshing and call force-fetch on them — zero wasted API calls.
 *
 * Event → Cache invalidation map:
 *   merchant.balance_updated  → balance + recentActivity + analytics + balanceHistory
 *   merchant.deposit          → balance + recentActivity + withdrawals + analytics
 *   customer.deposit          → balance + recentActivity + customers + customerSummary
 *   connected                 → (no-op — just the handshake ack)
 *
 * Lifecycle:
 *   start()  → called from App.tsx when isAuthenticated becomes true
 *   stop()   → called from authStore.logout() immediately on logout
 *
 * Reconnection: exponential backoff (1s → 2s → 4s … up to 30s cap).
 * Tab hidden: keeps WS alive (cheap, server-push only). No extra polling added.
 */

import { useDataStore } from './dataStore'
import { useBalanceStore } from './balanceStore'

// Derive WS URL from the REST base URL
function getWsUrl(): string {
  const restBase =
    (import.meta as any).env?.VITE_API_URL || 'https://api.fiddupay.com'
  // https://api.fiddupay.com → wss://api.fiddupay.com
  // http://localhost:3000   → ws://localhost:3000
  return restBase
    .replace(/^https:\/\//, 'wss://')
    .replace(/^http:\/\//, 'ws://')
    .replace(/\/$/, '') + '/api/v1/merchants/ws'
}

function getAuthToken(): string | null {
  return (
    sessionStorage.getItem('fiddupay_dashboard_token') ||
    localStorage.getItem('fiddupay_dashboard_token')
  )
}

// --- Event Handlers ---

function onBalanceUpdated() {
  const { fetchBalance } = useBalanceStore.getState()
  const { fetchRecentActivity, fetchAnalytics, fetchBalanceHistory } =
    useDataStore.getState()

  fetchBalance(true).catch(() => {})
  fetchRecentActivity(true).catch(() => {})
  fetchAnalytics(undefined, true).catch(() => {})
  fetchBalanceHistory(undefined, true).catch(() => {})
}

function onMerchantDeposit() {
  const { fetchBalance } = useBalanceStore.getState()
  const { fetchRecentActivity, fetchWithdrawals, fetchAnalytics } =
    useDataStore.getState()

  fetchBalance(true).catch(() => {})
  fetchRecentActivity(true).catch(() => {})
  fetchWithdrawals(true).catch(() => {})
  fetchAnalytics(undefined, true).catch(() => {})
}

function onCustomerDeposit() {
  const { fetchBalance } = useBalanceStore.getState()
  const { fetchRecentActivity, fetchCustomers, fetchCustomerSummary } =
    useDataStore.getState()

  fetchBalance(true).catch(() => {})
  fetchRecentActivity(true).catch(() => {})

  // Read current filters from the active URL to ensure we refresh the exact cache key the user sees
  const params = new URLSearchParams(window.location.search)
  const page = Math.max(1, parseInt(params.get('page') ?? '1', 10))
  const search = params.get('search') ?? undefined
  const status = params.get('status') ?? undefined
  fetchCustomers(page, 10, search, status, true).catch(() => {})

  fetchCustomerSummary(true).catch(() => {})
}

// --- Dispatcher: maps event string → handler ---
function dispatch(event: string) {
  switch (event) {
    case 'merchant.balance_updated':
      onBalanceUpdated()
      break
    case 'merchant.deposit':
      onMerchantDeposit()
      break
    case 'customer.deposit':
      onCustomerDeposit()
      break
    case 'connected':
      // Server handshake ack — nothing to do
      break
    default:
      // Unknown event — silently ignore (future-proof)
      break
  }
}

// --- WebSocket State Machine ---

let ws: WebSocket | null = null
let running = false
let reconnectTimer: ReturnType<typeof setTimeout> | null = null
let reconnectDelay = 1000 // ms, starts at 1s

const MAX_RECONNECT_DELAY = 30_000 // 30s cap

function clearReconnectTimer() {
  if (reconnectTimer !== null) {
    clearTimeout(reconnectTimer)
    reconnectTimer = null
  }
}

function connect() {
  if (!running) return

  const token = getAuthToken()
  const url = getWsUrl()

  // Pass auth token via Sec-WebSocket-Protocol header (the only header WS supports)
  // Backend echoes it back to complete the handshake.
  ws = token
    ? new WebSocket(url, [`Bearer.${token}`])
    : new WebSocket(url)

  ws.onopen = () => {
    reconnectDelay = 1000 // reset backoff on successful connect
  }

  ws.onmessage = (ev: MessageEvent) => {
    try {
      const msg = JSON.parse(ev.data as string)
      const event: string = msg?.event ?? ''
      if (event) dispatch(event)
    } catch {
      // Malformed payload — ignore
    }
  }

  ws.onclose = (ev: CloseEvent) => {
    ws = null
    // 1000 = normal closure (server/browser initiated intentional close)
    // Don't reconnect if we stopped ourselves.
    if (!running || ev.code === 1000) return

    // Schedule reconnect with exponential backoff
    clearReconnectTimer()
    reconnectTimer = setTimeout(() => {
      reconnectDelay = Math.min(reconnectDelay * 2, MAX_RECONNECT_DELAY)
      connect()
    }, reconnectDelay)
  }

  ws.onerror = () => {
    // onclose fires right after onerror — reconnect logic lives there
    ws?.close()
  }
}

function disconnect() {
  clearReconnectTimer()
  if (ws) {
    // Use code 1000 so onclose knows it was intentional and skips reconnect
    ws.close(1000, 'Logout')
    ws = null
  }
}

// --- Public API ---

export const backgroundPoller = {
  /**
   * Open the WebSocket connection. Safe to call multiple times — idempotent.
   * Called from App.tsx when isAuthenticated becomes true.
   */
  start() {
    if (running) return
    running = true
    connect()
  },

  /**
   * Close the WebSocket connection and cancel any pending reconnect.
   * Called from authStore.logout() immediately on logout.
   */
  stop() {
    if (!running) return
    running = false
    disconnect()
  },

  /** Returns true if the WS connection loop is active. */
  isRunning() {
    return running
  },

  /** Returns the current WebSocket readyState (or -1 if not connected). */
  readyState() {
    return ws ? ws.readyState : -1
  },
}
