import React, { useState } from 'react'
import { Link } from 'react-router-dom'
import styles from './Header.module.css'

const Header: React.FC = () => {
  const [isMenuOpen, setIsMenuOpen] = useState(false)

  return (
    <header className={styles.header}>
      <div className={styles.container}>
        <Link to="/" className={styles.logo}>
          PayFlow
        </Link>
        
        <nav className={`${styles.nav} ${isMenuOpen ? styles.navOpen : ''}`}>
          <Link to="/pricing" className={styles.navLink}>Pricing</Link>
          <Link to="/login" className={styles.loginBtn}>Login</Link>
        </nav>

        <button 
          className={styles.menuBtn}
          onClick={() => setIsMenuOpen(!isMenuOpen)}
          aria-label="Toggle menu"
        >
          <span></span>
          <span></span>
          <span></span>
        </button>
      </div>
    </header>
  )
}

export default Header
