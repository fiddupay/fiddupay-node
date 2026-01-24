import React from 'react'
import { MdNotifications, MdMenu } from 'react-icons/md'
import { useAuthStore } from '@/stores/authStore'
import styles from './Header.module.css'

const Header: React.FC = () => {
  const { user } = useAuthStore()

  return (
    <header className={styles.header}>
      <div className={styles.container}>
        <div className={styles.left}>
          <button className={styles.menuButton}>
            <MdMenu />
          </button>
        </div>
        
        <div className={styles.right}>
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
