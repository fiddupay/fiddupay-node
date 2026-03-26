# Offline-First Strategy (PWA Roadmap)

This document outlines the technical path and specific toolset for transitioning the FidduPay merchant frontend to a resilient Offline-First architecture.

## 🛠️ Required Toolset (The "Shopping List")

To implement this, the following libraries and utilities are required:

1.  **Vite PWA Plugin** (`vite-plugin-pwa`): Automates Service Worker generation and manifest management.
2.  **Workbox**: The industry-standard library for Service Worker logic (handles complex caching like *Stale-While-Revalidate*).
3.  **Dexie.js** or **idb-keyval**: High-performance wrappers for **IndexedDB**. Essential for storing large amounts of transaction history locally where `localStorage` fails.
4.  **TanStack Query** (`@tanstack/react-query`): Replaces standard Axios calls with auto-caching, background revalidation, and built-in offline support.
5.  **Query Persister** (`@tanstack/query-sync-storage-persister`): Bridges React Query and IndexedDB for permanent storage.
6.  **Broadcast Channel API**: Allows the background Service Worker to communicate status updates back to the UI.

## 1. Core Architecture: Service Worker
To handle the initial page load and asset delivery without internet:
- **Strategy**: **Cache-First** for static assets (React bundle, JS, CSS, Images, and **Google Fonts**).
- **Implementation**: Workbox intercepts network requests for application files, serving them from the browser cache instantly. This ensures the "Shell" of the app loads in <1 second regardless of connectivity.

## 2. Data Persistence Strategy (Zustand + Dexie)
For storing merchant balances and settings:
- **Zustand Persistence**: Configure the `persist` middleware.
- **Storage Engine**: Migrate from `localStorage` (limited to 5MB) to **Dexie** to ensure high-volume balance and customer data survives browser cleaning.
- **Scope**:
    - **Balances**: Store available/total balances and the last-updated timestamp.
    - **Customer Detail Modal**: Cache the last 50 viewed customers to ensure their details remain accessible.
    - **Merchant Settings**: Cache fee settings and dashboard preferences.

## 3. Network Resilience & Data Layer (React Query)
This replaces standard Axios calls with a **"Stale-While-Revalidate"** pattern:
1.  **Immediate Load**: The UI reads data directly from the local IndexedDB/Dexie cache (Instant visual feedback).
2.  **Background Sync**: The app triggers an API fetch to the Rust backend in the background.
3.  **Silent Update**: If the fetch succeeds, the local DB and UI are updated silently with fresh data. If it fails, the UI simply stays on the cached version without showing an error.

## 4. The Action Queue (Background Sync API)
For actions like "Request Withdrawal" or "Update Profile" performed while offline:
- **Process**: Save the request payload into a `pending_actions` table in IndexedDB.
- **Background Sync**: Register a sync event in the Service Worker.
- **Recovery**: When the browser detects the internet is back, the Service Worker "replays" the outbox to the API in the background—even if the user has closed the tab.

## 5. UI/UX "Trust" Requirements
- **Connectivity Listener**: Use `window.addEventListener('offline')` to toggle a global "Disconnected" banner.
- **Optimistic Updates**: Immediately show the result of an action (e.g., a new withdrawal appearing in the list) but mark it with a "Pending Sync" icon.
- **Last Sync Timestamp**: Display "Showing data from [timestamp]" to manage merchant expectations.

---

### Recommended Implementation Order:
1.  **Phase 1**: Install `vite-plugin-pwa` and enable basic asset caching (Offline page load).
2.  **Phase 2**: Add Zustand persistence middleware for balances and profile data.
3.  **Phase 3**: Transitioning heavy data fetching (Transactions, Customers) to TanStack Query for automatic background revalidation and IndexedDB persistence.
