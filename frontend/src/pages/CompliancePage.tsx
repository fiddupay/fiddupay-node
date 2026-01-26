import React from 'react'
import styles from './CompliancePage.module.css'

const CompliancePage: React.FC = () => {
  return (
    <div className={styles.compliancePage}>
      <div className={styles.container}>
        <div className={styles.content}>
          <h1>Compliance & Regulatory</h1>
          <p className={styles.lead}>
            FidduPay maintains the highest standards of regulatory compliance across all jurisdictions where we operate.
          </p>

          <section className={styles.section}>
            <h2>Regulatory Framework</h2>
            <div className={styles.complianceGrid}>
              <div className={styles.complianceItem}>
                <h3><i className="fas fa-shield-alt"></i> SOC 2 Type II</h3>
                <p>Independently audited security controls and operational procedures.</p>
              </div>
              <div className={styles.complianceItem}>
                <h3><i className="fas fa-lock"></i> PCI DSS Level 1</h3>
                <p>Highest level of payment card industry data security standards.</p>
              </div>
              <div className={styles.complianceItem}>
                <h3><i className="fas fa-balance-scale"></i> AML/KYC</h3>
                <p>Anti-Money Laundering and Know Your Customer compliance programs.</p>
              </div>
              <div className={styles.complianceItem}>
                <h3><i className="fas fa-globe"></i> GDPR</h3>
                <p>Full compliance with European General Data Protection Regulation.</p>
              </div>
            </div>
          </section>

          <section className={styles.section}>
            <h2>Security Standards</h2>
            <ul>
              <li>ISO 27001 Information Security Management</li>
              <li>ISO 27017 Cloud Security</li>
              <li>NIST Cybersecurity Framework</li>
              <li>OWASP Top 10 Security Controls</li>
            </ul>
          </section>
        </div>
      </div>
    </div>
  )
}

export default CompliancePage
