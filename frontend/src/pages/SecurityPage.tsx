import React from 'react'
import styles from './SecurityPage.module.css'

const SecurityPage: React.FC = () => {
  return (
    <div className={styles.securityPage}>
      <div className={styles.container}>
        <div className={styles.header}>
          <h1>Security at FidduPay</h1>
          <p>Bank-level security for cryptocurrency payments</p>
        </div>

        <div className={styles.content}>
          <section className={styles.section}>
            <h2>Enterprise-Grade Security</h2>
            <div className={styles.securityFeatures}>
              <div className={styles.feature}>
                <i className="fas fa-shield-alt"></i>
                <h3>End-to-End Encryption</h3>
                <p>All data encrypted with AES-256 encryption</p>
              </div>
              <div className={styles.feature}>
                <i className="fas fa-lock"></i>
                <h3>Multi-Signature Wallets</h3>
                <p>Enhanced security with multi-signature technology</p>
              </div>
              <div className={styles.feature}>
                <i className="fas fa-eye"></i>
                <h3>Real-time Monitoring</h3>
                <p>24/7 threat detection and response</p>
              </div>
              <div className={styles.feature}>
                <i className="fas fa-certificate"></i>
                <h3>SOC 2 Compliant</h3>
                <p>Certified security controls and processes</p>
              </div>
            </div>
          </section>

          <section className={styles.section}>
            <h2>Security Measures</h2>
            <ul className={styles.measures}>
              <li>Advanced rate limiting and DDoS protection</li>
              <li>Account lockout protection against brute force attacks</li>
              <li>Real-time fraud detection and prevention</li>
              <li>Secure key management and storage</li>
              <li>Regular security audits and penetration testing</li>
              <li>CSRF and XSS protection</li>
            </ul>
          </section>

          <section className={styles.section}>
            <h2>Compliance</h2>
            <p>FidduPay adheres to the highest security standards and regulatory requirements to ensure your funds and data are protected.</p>
          </section>
        </div>
      </div>
    </div>
  )
}

export default SecurityPage
