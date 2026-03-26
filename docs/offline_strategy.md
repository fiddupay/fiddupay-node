# Offline-First Strategy (PWA Roadmap)

This document outlines the technical path for transitioning the FidduPay merchant frontend to a resilient Offline-First architecture.

## 1. Core Architecture: Service Worker
To handle the initial page load and asset delivery without internet:
- **Tool**: `vite-plugin-pwa`
- **Strategy**: **Cache-First** for static assets (React bundle, CSS, Images, Google Fonts). This ensures the "Shell" of the app loads in <1 second regardless of connectivity.

## 2. Data Persistence Strategy
Since the dashboard uses **Zustand** for state management:
- **Middleware**: Use `zustand/middleware/persist`.
- **Storage Engine**: Migrate from `localStorage` (which is limited to 5MB) to **IndexedDB** using a library like `idb-keyval`.
- **Scope**:
    - **Balances**: Store the last known balance and the "last updated" timestamp.
    - **Customer Detail Modal**: Cache the last 50 viewed customers to ensure their details remain accessible.
    - **Transaction History**: Store the most recent 100 transactions locally.

## 3. Network Resilience Handler
Replace standard Axios calls with a **"Stale-While-Revalidate"** pattern:
1.  **Immediate Load**: UI reads from the local IndexedDB.
2.  **Background Sync**: Trigger an API call to the Rust backend.
3.  **Silent Update**: If the network is available, the local DB and UI are updated silently. If not, the UI shows a "Using Cached Data" indicator.

## 4. Offline Activity Queue (Queued Sync)
For actions like "Request Withdrawal" or "Update Profile" performed while offline:
- **Process**: Save the request payload into an **Outbox** table in IndexedDB.
- **Background Sync**: Register a `sync` event in the Service Worker.
- **Recovery**: When the browser detects the internet is back, the Service Worker "replays" the outbox to the API in the background—even if the user has closed the tab.

## 5. UI/UX Requirements
- **Connectivity Listener**: Use `window.addEventListener('offline')` to toggle a global "Disconnected" banner.
- **Optimistic Updates**: Immediately show the result of an action (e.g., a new withdrawal appearing in the list) but mark it with a "Pending Sync" icon.

---

### Recommended Implementation Order:
1.  **Phase 1**: Install `vite-plugin-pwa` and enable asset caching (Offline page load).
2.  **Phase 2**: Add Zustand persistence middleware to the `useMerchantStore` for balances and profile data.
3.  **Phase 3**: Implement a global "Offline Banner" and transition heavy data fetching to TanStack Query for automatic background revalidation.
