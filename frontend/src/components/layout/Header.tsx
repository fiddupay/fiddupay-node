import React from 'react'
import { MdNotifications, MdMenu, MdFlashOn, MdScience } from 'react-icons/md'
import { useAuthStore } from '@/stores/authStore'
import { merchantAPI } from '@/services/apiService'
import { useToast } from '@/contexts/ToastContext'
import styles from '@/styles/components/layout/Header.module.css'

const Header: React.FC = () => {
  const { user, loadUser } = useAuthStore()
  const { showToast } = useToast()

  const handleSwitchEnvironment = async () => {
    try {
      const toLive = user?.sandbox_mode || false
      const response = await merchantAPI.switchEnvironment(toLive)

      // The API returns a new API key, which our auth store uses for the session
      localStorage.setItem('fiddupay_token', response.data.api_key)

      await loadUser()
      showToast(`Switched to ${toLive ? 'Live' : 'Sandbox'} mode`, 'success')

      // Force refresh to clear states if necessary, or just rely on loadUser
      window.location.reload()
    } catch (error: any) {
      showToast('Failed to switch environment', 'error')
    }
  }

  return (
    <header className={styles.header}>
      <div className={styles.container}>
        <div className={styles.left}>
          <button className={styles.menuButton}>
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

          <button className={styles.notificationButton}>
            <MdNotifications />
          </button>

          <div className={styles.userInfo}>
            <div className={styles.userAvatar}>
              {user?.business_name?.charAt(0).toUpperCase()}
            </div>
            <span className={styles.userName}>{user?.business_name}</span>
          </div>
        </div>
      </div>
    </header>
  )
}

export default Header
