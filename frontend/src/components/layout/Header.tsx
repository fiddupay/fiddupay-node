import React, { useEffect, useState, useRef } from 'react'
import { 
  MdNotifications, 
  MdMenu, 
  MdFlashOn, 
  MdScience, 
  MdKeyboardArrowDown,
  MdSettings,
  MdSecurity,
  MdVpnKey,
  MdExitToApp,
  MdPerson
} from 'react-icons/md'
import { useNavigate } from 'react-router-dom'
import { useAuthStore } from '@/stores/authStore'
import { merchantAPI } from '@/services/apiService'
import { setSuppressAuthRedirect } from '@/utils/api'
import { useToast } from '@/contexts/ToastContext'
import { useNotificationStore } from '@/stores/notificationStore'
import NotificationPanel from './NotificationPanel'
import styles from '@/styles/components/layout/Header.module.css'

interface HeaderProps {
  onMenuClick?: () => void
}

const Header: React.FC<HeaderProps> = ({ onMenuClick }) => {
  const { user, loadUser, logout } = useAuthStore()
  const { showToast } = useToast()
  const navigate = useNavigate()
  const { unreadCount, togglePanel, fetchNotifications } = useNotificationStore()
  
  const [isMenuOpen, setIsMenuOpen] = useState(false)
  const menuRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    fetchNotifications()
  }, [fetchNotifications])

  // Click outside to close menu
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (menuRef.current && !menuRef.current.contains(event.target as Node)) {
        setIsMenuOpen(false)
      }
    }

    if (isMenuOpen) {
      document.addEventListener('mousedown', handleClickOutside)
    }
    return () => {
      document.removeEventListener('mousedown', handleClickOutside)
    }
  }, [isMenuOpen])

  const handleSwitchEnvironment = async (e: React.MouseEvent) => {
    e.preventDefault()
    setSuppressAuthRedirect(true)

    try {
      const toLive = user?.sandbox_mode || false
      if (toLive && user?.kyc_tier === 0) {
        showToast('Tier 1 Verification required to switch to Live mode', 'warning')
        return
      }

      await merchantAPI.switchEnvironment(toLive)
      await loadUser(true)
      showToast(`Switched to ${toLive ? 'Live' : 'Sandbox'} mode`, 'success')
    } catch (error: any) {
      showToast('Failed to switch environment', 'error')
    } finally {
      setSuppressAuthRedirect(false)
    }
  }

  const handleLogout = () => {
    logout()
    navigate('/login')
    showToast('Signed out successfully', 'info')
  }

  const handleMenuAction = (path: string) => {
    navigate(path)
    setIsMenuOpen(false)
  }

  return (
    <header className={styles.header}>
      <div className={styles.container}>
        <div className={styles.left}>
          <button className={styles.menuButton} onClick={onMenuClick}>
            <MdMenu />
          </button>
        </div>

        <div className={styles.right}>
          <div className={styles.envControl}>
            <div className={`${styles.envBadge} ${user?.sandbox_mode ? styles.sandbox : styles.live}`}>
              <div className={styles.pulse} />
              {user?.sandbox_mode ? <MdScience className="text-sm" /> : <MdFlashOn className="text-sm" />}
              {user?.sandbox_mode ? 'Sandbox' : 'Live'}
            </div>

            <div className={styles.toggleContainer} onClick={handleSwitchEnvironment}>
              <span className={`${styles.toggleLabel} ${!user?.sandbox_mode ? styles.active : ''}`}>Live</span>
              <label className={styles.switch}>
                <input
                  type="checkbox"
                  checked={user?.sandbox_mode}
                  readOnly
                />
                <span className={styles.slider}></span>
              </label>
              <span className={`${styles.toggleLabel} ${user?.sandbox_mode ? styles.active : ''}`}>Sandbox</span>
            </div>
          </div>

          <button 
            className={styles.notificationButton}
            onClick={() => togglePanel()}
          >
            <MdNotifications />
            {unreadCount > 0 && (
              <span className={styles.badge}>
                {unreadCount > 9 ? '9+' : unreadCount}
              </span>
            )}
          </button>

          <NotificationPanel />

          <div className={styles.userProfileContainer} ref={menuRef}>
            <button 
              className={`${styles.userTrigger} ${isMenuOpen ? styles.active : ''}`}
              onClick={() => setIsMenuOpen(!isMenuOpen)}
            >
              <div className={styles.userAvatar}>
                {user?.business_name?.charAt(0).toUpperCase()}
              </div>
              <span className={styles.userName}>{user?.business_name}</span>
              <MdKeyboardArrowDown className={styles.chevron} />
            </button>

            {isMenuOpen && (
              <div className={styles.userDropdown}>
                <div className={styles.dropdownHeader}>
                  <div className={styles.headerBusinessName}>{user?.business_name}</div>
                  <div className={styles.headerTier}>
                    Account Status: <span>Tier {user?.kyc_tier}</span>
                  </div>
                </div>

                <button className={styles.dropdownLink} onClick={() => handleMenuAction('/app/settings?tab=settlement')}>
                  <MdPerson /> My Profile
                </button>
                <button className={styles.dropdownLink} onClick={() => handleMenuAction('/app/settings?tab=verification')}>
                  <MdSecurity /> Verification
                </button>
                <button className={styles.dropdownLink} onClick={() => handleMenuAction('/app/settings?tab=api')}>
                  <MdVpnKey /> API Settings
                </button>
                
                <div className={styles.dropdownDivider} />
                
                <button className={styles.dropdownLink} onClick={() => handleMenuAction('/app/settings')}>
                  <MdSettings /> Settings
                </button>
                <button className={`${styles.dropdownLink} ${styles.logoutBtn}`} onClick={handleLogout}>
                  <MdExitToApp /> Sign Out
                </button>
              </div>
            )}
          </div>
        </div>
      </div>
    </header>
  )
}

export default Header
