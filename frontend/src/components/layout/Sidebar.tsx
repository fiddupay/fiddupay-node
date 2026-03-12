import React from 'react'
import { NavLink, useLocation } from 'react-router-dom'
import { useAuthStore } from '@/stores/authStore'
import styles from '@/styles/components/layout/Sidebar.module.css'

const navigation = [
  { name: 'Dashboard', href: '/app/dashboard', iconClass: 'fas fa-tachometer-alt' },
  { name: 'Payments', href: '/app/payments', iconClass: 'fas fa-money-bill-wave' },
  { name: 'Wallets', href: '/app/wallets', iconClass: 'fas fa-wallet' },
  { name: 'Balance', href: '/app/balance', iconClass: 'fas fa-university' },
  { name: 'Withdrawals', href: '/app/withdrawals', iconClass: 'fas fa-sign-out-alt' },
  { name: 'Customers', href: '/app/customers', iconClass: 'fas fa-users' },
  { name: 'Reports', href: '/app/reports', iconClass: 'fas fa-file-invoice' },
  { name: 'Settings', href: '/app/settings', iconClass: 'fas fa-cog' },
]

interface SidebarProps {
  isOpen?: boolean
  onClose?: () => void
}

const Sidebar: React.FC<SidebarProps> = ({ isOpen, onClose }) => {
  const location = useLocation()
  const { user, logout } = useAuthStore()

  return (
    <div className={`${styles.sidebar} ${isOpen ? styles.sidebarOpen : ''}`}>
      <div className={styles.sidebarContent}>
        <div className={styles.logo}>
          <h1>FidduPay</h1>
        </div>

        <nav className={styles.nav}>
          <ul className={styles.navList}>
            {navigation.map((item) => {
              const isActive = location.pathname.startsWith(item.href)
              return (
                <li key={item.name} onClick={onClose}>
                  <NavLink
                    to={item.href}
                    className={`${styles.navLink} ${isActive ? styles.navLinkActive : ''}`}
                  >
                    <i className={`${item.iconClass} ${styles.navIcon}`}></i>
                    {item.name}
                  </NavLink>
                </li>
              )
            })}
          </ul>

          <div className={styles.userSection}>
            <div className={styles.userInfo}>
              <div className={styles.userAvatar}>
                {user?.business_name?.charAt(0).toUpperCase()}
              </div>
              <div className={styles.userDetails}>
                <p className={styles.userName}>{user?.business_name}</p>
                <p className={styles.userEmail}>{user?.email}</p>
              </div>
            </div>

            <button onClick={logout} className={styles.logoutButton}>
              <i className={`fas fa-sign-out-alt ${styles.navIcon}`}></i>
              Sign out
            </button>
          </div>
        </nav>
      </div>
    </div>
  )
}

export default Sidebar
