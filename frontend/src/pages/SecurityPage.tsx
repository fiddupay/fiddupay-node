import React from 'react'
import styles from './SecurityPage.module.css'

const SecurityPage: React.FC = () => {
  return (
    <div className={styles.securityPage}>
      <div className={styles.container}>
        {/* Hero Section */}
        <section className={styles.hero}>
          <div className={styles.heroContent}>
            <h1>Enterprise-Grade Security</h1>
            <p>Your funds and data are protected by military-grade security measures and industry-leading protocols.</p>
          </div>
        </section>

        {/* Security Features Grid */}
        <section className={styles.features}>
          <div className={styles.featuresGrid}>
            <div className={styles.featureCard}>
              <div className={styles.featureIcon}>
                <i className="fas fa-shield-alt"></i>
              </div>
              <h3>Advanced Encryption</h3>
              <p>AES-256 encryption for all sensitive data with end-to-end protection for transactions and user information.</p>
            </div>

            <div className={styles.featureCard}>
              <div className={styles.featureIcon}>
                <i className="fas fa-lock"></i>
              </div>
              <h3>Multi-Signature Wallets</h3>
              <p>All funds are secured in multi-signature wallets requiring multiple approvals for any transaction.</p>
            </div>

            <div className={styles.featureCard}>
              <div className={styles.featureIcon}>
                <i className="fas fa-eye"></i>
              </div>
              <h3>Real-Time Monitoring</h3>
              <p>24/7 threat detection and monitoring systems protect against suspicious activities and attacks.</p>
            </div>

            <div className={styles.featureCard}>
              <div className={styles.featureIcon}>
                <i className="fas fa-server"></i>
              </div>
              <h3>Secure Infrastructure</h3>
              <p>Enterprise-grade infrastructure with redundant systems and automated backups for maximum reliability.</p>
            </div>

            <div className={styles.featureCard}>
              <div className={styles.featureIcon}>
                <i className="fas fa-user-shield"></i>
              </div>
              <h3>Two-Factor Authentication</h3>
              <p>Mandatory 2FA for all accounts with support for authenticator apps and hardware keys.</p>
            </div>

            <div className={styles.featureCard}>
              <div className={styles.featureIcon}>
                <i className="fas fa-chart-line"></i>
              </div>
              <h3>Rate Limiting</h3>
              <p>Advanced rate limiting and DDoS protection prevent abuse and ensure system availability.</p>
            </div>
          </div>
        </section>

        {/* Compliance Section */}
        <section className={styles.compliance}>
          <h2>Security Standards & Compliance</h2>
          <div className={styles.complianceGrid}>
            <div className={styles.complianceItem}>
              <h3>ISO 27001</h3>
              <p>Information Security Management System certification</p>
            </div>
            <div className={styles.complianceItem}>
              <h3>SOC 2 Type II</h3>
              <p>Independently audited security controls and procedures</p>
            </div>
            <div className={styles.complianceItem}>
              <h3>OWASP Top 10</h3>
              <p>Protection against all major web application vulnerabilities</p>
            </div>
            <div className={styles.complianceItem}>
              <h3>NIST Framework</h3>
              <p>Cybersecurity framework implementation and compliance</p>
            </div>
          </div>
        </section>

        {/* Security Measures */}
        <section className={styles.measures}>
          <h2>Additional Security Measures</h2>
          <div className={styles.measuresList}>
            <div className={styles.measure}>
              <i className="fas fa-check-circle"></i>
              <span>Regular security audits by third-party firms</span>
            </div>
            <div className={styles.measure}>
              <i className="fas fa-check-circle"></i>
              <span>Penetration testing and vulnerability assessments</span>
            </div>
            <div className={styles.measure}>
              <i className="fas fa-check-circle"></i>
              <span>Employee security training and background checks</span>
            </div>
            <div className={styles.measure}>
              <i className="fas fa-check-circle"></i>
              <span>Incident response and disaster recovery plans</span>
            </div>
            <div className={styles.measure}>
              <i className="fas fa-check-circle"></i>
              <span>Regular security updates and patch management</span>
            </div>
            <div className={styles.measure}>
              <i className="fas fa-check-circle"></i>
              <span>Comprehensive logging and audit trails</span>
            </div>
          </div>
        </section>
      </div>
    </div>
  )
}

export default SecurityPage
