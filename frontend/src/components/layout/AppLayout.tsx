import React from 'react'
import { Outlet, Navigate } from 'react-router-dom'
import { useAuthStore } from '@/stores/authStore'
import Sidebar from './Sidebar'
import Header from './Header'
import { LiveDropToast } from './LiveDropToast'
import LoadingSpinner from '../ui/LoadingSpinner'
import styles from '@/styles/components/layout/AppLayout.module.css'

const AppLayout: React.FC = () => {
  const { isAuthenticated, loading } = useAuthStore()
  const [isMobileMenuOpen, setIsMobileMenuOpen] = React.useState(false)

  const toggleMobileMenu = () => setIsMobileMenuOpen(!isMobileMenuOpen)
  const closeMobileMenu = () => setIsMobileMenuOpen(false)

  if (loading) {
    return (
      <div className={styles.loadingContainer}>
        <LoadingSpinner />
      </div>
    )
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
