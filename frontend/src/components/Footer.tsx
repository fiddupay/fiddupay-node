import React from 'react'
import { Link } from 'react-router-dom'
import styles from './Footer.module.css'

const Footer: React.FC = () => {
  return (
    <footer className={styles.footer}>
      <div className={styles.container}>
        <div className={styles.content}>
          <div className={styles.brand}>
            <div className={styles.logo}>
              <i className="fas fa-coins"></i>
              <span>FidduPay</span>
            </div>
            <p className={styles.description}>
              Enterprise-grade cryptocurrency payment gateway trusted by businesses worldwide. 
              Accept payments across 5 blockchain networks with bank-level security.
            </p>
            <div className={styles.social}>
              <a href="#" className={styles.socialLink}>
                <i className="fab fa-twitter"></i>
              </a>
              <a href="#" className={styles.socialLink}>
                <i className="fab fa-linkedin"></i>
              </a>
              <a href="#" className={styles.socialLink}>
                <i className="fab fa-github"></i>
              </a>
              <a href="#" className={styles.socialLink}>
                <i className="fab fa-discord"></i>
              </a>
            </div>
          </div>
          
          <div className={styles.links}>
            <div className={styles.linkGroup}>
              <h4><i className="fas fa-cube"></i> Product</h4>
              <Link to="/pricing">Pricing</Link>
              <Link to="/docs">API Documentation</Link>
              <Link to="/features">Features</Link>
              <Link to="/contact">Demo</Link>
            </div>
            
            <div className={styles.linkGroup}>
              <h4><i className="fas fa-building"></i> Company</h4>
              <Link to="/about">About Us</Link>
              <Link to="/contact">Contact</Link>
              <Link to="/careers">Careers</Link>
              <Link to="/blog">Blog</Link>
            </div>
            
            <div className={styles.linkGroup}>
              <h4><i className="fas fa-headset"></i> Support</h4>
              <Link to="/docs">Help Center</Link>
              <Link to="/contact">Contact Support</Link>
              <Link to="/status">System Status</Link>
              <Link to="/security">Security</Link>
            </div>
            
            <div className={styles.linkGroup}>
              <h4><i className="fas fa-gavel"></i> Legal</h4>
              <Link to="/terms">Terms of Service</Link>
              <Link to="/privacy">Privacy Policy</Link>
              <Link to="/compliance">Compliance</Link>
              <Link to="/cookies">Cookie Policy</Link>
            </div>
          </div>
        </div>
        
        <div className={styles.bottom}>
          <div className={styles.bottomContent}>
            <p>&copy; 2026 FidduPay by TechyTro Software. All rights reserved.</p>
            <div className={styles.badges}>
              <div className={styles.badge}>
                <i className="fas fa-shield-alt"></i>
                <span>10/10 Security</span>
              </div>
              <div className={styles.badge}>
                <i className="fas fa-certificate"></i>
                <span>SOC 2 Compliant</span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </footer>
  )
}

export default Footer
