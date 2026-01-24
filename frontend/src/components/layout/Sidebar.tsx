import React from 'react'
import { NavLink, useLocation } from 'react-router-dom'
import {
  MdDashboard,
  MdPayment,
  MdAccountBalanceWallet,
  MdBarChart,
  MdSettings,
  MdLogout,
  MdAccountBalance,
  MdAssessment,
} from 'react-icons/md'
import { useAuthStore } from '@/stores/authStore'
import styles from './Sidebar.module.css'

const navigation = [
  { name: 'Dashboard', href: '/dashboard', icon: MdDashboard },
  { name: 'Payments', href: '/payments', icon: MdPayment },
  { name: 'Wallets', href: '/wallets', icon: MdAccountBalanceWallet },
  { name: 'Balance', href: '/balance', icon: MdAccountBalance },
  { name: 'Withdrawals', href: '/withdrawals', icon: MdLogout },
  { name: 'Analytics', href: '/analytics', icon: MdBarChart },
  { name: 'Reports', href: '/reports', icon: MdAssessment },
  { name: 'Settings', href: '/settings', icon: MdSettings },
]

const Sidebar: React.FC = () => {
  const location = useLocation()
  const { user, logout } = useAuthStore()

  return (
    <div className={styles.sidebar}>
      <div className={styles.sidebarContent}>
        <div className={styles.logo}>
          <h1>ChainPay</h1>
        </div>
        
        <nav className={styles.nav}>
          <ul className={styles.navList}>
            {navigation.map((item) => {
              const isActive = location.pathname.startsWith(item.href)
              return (
                <li key={item.name}>
                  <NavLink
                    to={item.href}
                    className={`${styles.navLink} ${isActive ? styles.navLinkActive : ''}`}
                  >
                    <item.icon className={styles.navIcon} />
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
              <MdLogout className={styles.navIcon} />
              Sign out
            </button>
          </div>
        </nav>
      </div>
    </div>
  )
}

export default Sidebar
