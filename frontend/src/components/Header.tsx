import React, { useState } from 'react'
import { Link, useLocation } from 'react-router-dom'
import styles from './Header.module.css'

const Header: React.FC = () => {
  const [isMenuOpen, setIsMenuOpen] = useState(false)
  const location = useLocation()

  const isActive = (path: string) => location.pathname === path

  return (
    <header className={styles.header}>
      <div className={styles.container}>
        <Link to="/" className={styles.logo}>
          ChainPay
        </Link>
        
        <nav className={`${styles.nav} ${isMenuOpen ? styles.navOpen : ''}`}>
          <Link 
            to="/" 
            className={`${styles.navLink} ${isActive('/') ? styles.active : ''}`}
            onClick={() => setIsMenuOpen(false)}
          >
            Home
          </Link>
          <Link 
            to="/features" 
            className={`${styles.navLink} ${isActive('/features') ? styles.active : ''}`}
            onClick={() => setIsMenuOpen(false)}
          >
            Features
          </Link>
          <Link 
            to="/pricing" 
            className={`${styles.navLink} ${isActive('/pricing') ? styles.active : ''}`}
            onClick={() => setIsMenuOpen(false)}
          >
            Pricing
          </Link>
          <Link 
            to="/docs" 
            className={`${styles.navLink} ${isActive('/docs') ? styles.active : ''}`}
            onClick={() => setIsMenuOpen(false)}
          >
            Docs
          </Link>
          <Link 
            to="/contact" 
            className={`${styles.navLink} ${isActive('/contact') ? styles.active : ''}`}
            onClick={() => setIsMenuOpen(false)}
          >
            Contact
          </Link>
          <Link 
            to="/login" 
            className={styles.loginBtn}
            onClick={() => setIsMenuOpen(false)}
          >
            Login
          </Link>
        </nav>

        <button 
          className={styles.menuBtn}
          onClick={() => setIsMenuOpen(!isMenuOpen)}
          aria-label="Toggle menu"
        >
          <span className={styles.hamburger}></span>
          <span className={styles.hamburger}></span>
          <span className={styles.hamburger}></span>
        </button>
      </div>
    </header>
  )
}

export default Header
