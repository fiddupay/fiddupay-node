import React, { useEffect } from 'react'
import { Outlet, Navigate } from 'react-router-dom'
import { useAuthStore } from '@/stores/authStore'
import { useDataStore } from '@/stores/dataStore'
import { useBalanceStore } from '@/stores/balanceStore'
import { useNotificationStore } from '@/stores/notificationStore'
import Sidebar from './Sidebar'
import Header from './Header'
import { LiveDropToast } from './LiveDropToast'
import { DashboardSkeleton } from './PageSkeletons'
import styles from '@/styles/components/layout/AppLayout.module.css'

const AppLayout: React.FC = () => {
  const { isAuthenticated, loading } = useAuthStore()
  const [isMobileMenuOpen, setIsMobileMenuOpen] = React.useState(false)

  const toggleMobileMenu = () => setIsMobileMenuOpen(!isMobileMenuOpen)
  const closeMobileMenu = () => setIsMobileMenuOpen(false)

  // Boot-time prefetch: warm the global cache when the merchant enters the app shell.
  // This ensures currencies, balance, and notifications are already loaded
  // before the user navigates to any specific page.
  useEffect(() => {
    if (isAuthenticated) {
      const store = useDataStore.getState()
      
      // 1. Critical Base Data
      store.fetchCurrencies()
      useBalanceStore.getState().fetchBalance()
      useNotificationStore.getState().fetchNotifications()

      // 2. Dashboard Warm-up (Background)
      store.fetchAnalytics()
      store.fetchCustomerSummary()
      store.fetchSecurityAlerts()
      store.fetchRecentActivity()
    }
  }, [isAuthenticated])

  if (loading) {
    return <DashboardSkeleton />
  }

  if (!isAuthenticated) {
    return <Navigate to="/login" replace />
  }

  return (
    <div className={styles.layout}>
      <LiveDropToast />
      <Sidebar isOpen={isMobileMenuOpen} onClose={closeMobileMenu} />
      
      {isMobileMenuOpen && (
        <div className={styles.overlay} onClick={closeMobileMenu} />
      )}

      <div className={styles.mainContent}>
        <Header onMenuClick={toggleMobileMenu} />
        <main className={styles.main}>
          <div className={styles.container}>
            <Outlet />
          </div>
        </main>
      </div>
    </div>
  )
}

export default AppLayout

