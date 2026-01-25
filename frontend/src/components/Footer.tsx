import React from 'react'
import { Link } from 'react-router-dom'
import styles from './Footer.module.css'

const Footer: React.FC = () => {
  return (
    <footer className={styles.footer}>
      <div className={styles.container}>
        <div className={styles.content}>
          <div className={styles.brand}>
            <h3 className={styles.logo}>FidduPay</h3>
            <p className={styles.description}>
              Modern cryptocurrency payment gateway for businesses
            </p>
          </div>
          
          <div className={styles.links}>
            <div className={styles.linkGroup}>
              <h4>Product</h4>
              <Link to="/pricing">Pricing</Link>
              <Link to="/docs">Documentation</Link>
              <Link to="/features">Features</Link>
            </div>
            <div className={styles.linkGroup}>
              <h4>Company</h4>
              <Link to="/contact">Contact</Link>
              <Link to="/terms">Terms</Link>
              <Link to="/privacy">Privacy</Link>
            </div>
          </div>
        </div>
        
        <div className={styles.bottom}>
          <p>&copy; 2026 FidduPay. All rights reserved.</p>
        </div>
      </div>
    </footer>
  )
}

export default Footer
