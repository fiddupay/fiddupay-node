import React from 'react'
import styles from './Footer.module.css'

const Footer: React.FC = () => {
  return (
    <footer className={styles.footer}>
      <div className={styles.container}>
        <div className={styles.content}>
          <div className={styles.brand}>
            <h3 className={styles.logo}>PayFlow</h3>
            <p className={styles.description}>
              Modern cryptocurrency payment gateway for businesses
            </p>
          </div>
          
          <div className={styles.links}>
            <div className={styles.linkGroup}>
              <h4>Product</h4>
              <a href="/pricing">Pricing</a>
              <a href="/docs">Documentation</a>
            </div>
            <div className={styles.linkGroup}>
              <h4>Support</h4>
              <a href="mailto:support@payflow.com">Contact</a>
              <a href="/status">Status</a>
            </div>
          </div>
        </div>
        
        <div className={styles.bottom}>
          <p>&copy; 2026 PayFlow. All rights reserved.</p>
        </div>
      </div>
    </footer>
  )
}

export default Footer
