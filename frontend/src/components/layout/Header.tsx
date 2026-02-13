import React from 'react'
import { MdNotifications, MdMenu, MdFlashOn, MdScience } from 'react-icons/md'
import { useAuthStore } from '@/stores/authStore'
import { merchantAPI } from '@/services/apiService'
import { setSuppressAuthRedirect } from '@/utils/api'
import { useToast } from '@/contexts/ToastContext'
import styles from '@/styles/components/layout/Header.module.css'

const Header: React.FC = () => {
  const { user, loadUser } = useAuthStore()
  const { showToast } = useToast()

  const handleSwitchEnvironment = async () => {
    // Suppress 401 interceptor BEFORE the API call to prevent
    // concurrent requests from triggering logout during the switch.
    setSuppressAuthRedirect(true)

    try {
      const toLive = user?.sandbox_mode || false
      await merchantAPI.switchEnvironment(toLive)

      // If the backend returned a specific key for this environment, it's for display only now.
      // We don't need to store it as a session token since we use JWT.


      // Reload user profile to pick up the new sandbox_mode
      await loadUser(true)

      showToast(`Switched to ${toLive ? 'Live' : 'Sandbox'} mode`, 'success')
    } catch (error: any) {
      showToast('Failed to switch environment', 'error')
    } finally {
      // Re-enable the 401 interceptor
      setSuppressAuthRedirect(false)
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
