import React from 'react'
import { NavLink, useLocation } from 'react-router-dom'
import { MdFlashOn, MdScience } from 'react-icons/md'
import { useAuthStore } from '@/stores/authStore'
import { merchantAPI } from '@/services/apiService'
import { setSuppressAuthRedirect } from '@/utils/api'
import { useToast } from '@/contexts/ToastContext'
import styles from '@/styles/components/layout/Sidebar.module.css'

const navigation = [
  { name: 'Dashboard', href: '/app/dashboard', iconClass: 'fas fa-tachometer-alt' },
  { name: 'Payments', href: '/app/payments', iconClass: 'fas fa-money-bill-wave' },
  { name: 'Wallets', href: '/app/wallets', iconClass: 'fas fa-wallet' },
  { name: 'Balance', href: '/app/balance', iconClass: 'fas fa-university' },
  { name: 'Withdrawals', href: '/app/withdrawals', iconClass: 'fas fa-sign-out-alt' },
  { name: 'Customers', href: '/app/customers', iconClass: 'fas fa-users' },
  { name: 'Reports', href: '/app/reports', iconClass: 'fas fa-file-invoice' },
  { name: 'Security', href: '/app/security', iconClass: 'fas fa-shield-alt' },
  { name: 'Settings', href: '/app/settings', iconClass: 'fas fa-cog' },
]

interface SidebarProps {
  isOpen?: boolean
  onClose?: () => void
}

const Sidebar: React.FC<SidebarProps> = ({ isOpen, onClose }) => {
  const location = useLocation()
  const { user, loadUser, logout } = useAuthStore()
  const { showToast } = useToast()
  const [switching, setSwitching] = React.useState(false)

  const handleSwitchEnvironment = async (e: React.MouseEvent) => {
    e.preventDefault()
    if (switching) return

    setSuppressAuthRedirect(true)
    setSwitching(true)
    try {
      const toLive = user?.sandbox_mode || false
      await merchantAPI.switchEnvironment(toLive)
      await loadUser(true)
      showToast(`Switched to ${toLive ? 'Live' : 'Sandbox'} mode`, 'success')
      if (onClose) onClose()
    } catch (error: any) {
      showToast('Failed to switch environment', 'error')
    } finally {
      setSuppressAuthRedirect(false)
      setSwitching(false)
    }
  }

  return (
    <div className={`${styles.sidebar} ${isOpen ? styles.sidebarOpen : ''}`}>
      <div className={styles.sidebarContent}>
        <div className={styles.logo}>
          <img src="/logo/logo-brandmark.svg" alt="FidduPay" style={{ height: '32px' }} />
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

            <div className={styles.envToggleWrapper} onClick={handleSwitchEnvironment}>
              <div className={styles.envInfo}>
                <div className={`${styles.envBadge} ${user?.sandbox_mode ? styles.sandbox : styles.live}`}>
                  <div className={styles.pulse} />
                  {user?.sandbox_mode ? <MdScience /> : <MdFlashOn />}
                  <span>{user?.sandbox_mode ? 'Sandbox' : 'Live'}</span>
                </div>
                <span className={styles.envActionLabel}>
                  {switching ? 'Switching...' : `Switch to ${user?.sandbox_mode ? 'Live' : 'Sandbox'}`}
                </span>
              </div>
              <div className={styles.toggleSwitch}>
                <label className={styles.switch}>
                  <input
                    type="checkbox"
                    checked={user?.sandbox_mode}
                    readOnly
                  />
                  <span className={styles.slider}></span>
                </label>
              </div>
            </div>

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
        </nav>
      </div>
    </div>
  )
}

export default Sidebar
