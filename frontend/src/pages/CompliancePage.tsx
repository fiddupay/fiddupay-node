import React from 'react'
import styles from '@/styles/pages/CompliancePage.module.css'

const CompliancePage: React.FC = () => {
  return (
    <div className={styles.legalPage}>
      {/* Ambient Glow */}
      <div className={styles.ambientGlowContainer}>
        <div className={`${styles.blob} ${styles.blobPrimary}`}></div>
        <div className={`${styles.blob} ${styles.blobSecondary}`}></div>
      </div>

      <div className={styles.container}>
        <header className={`${styles.header} animate-fade-in-up`}>
          <div className={styles.badge}>Regulatory Standards</div>
          <h1 className={styles.title}>Compliance & Regulatory</h1>
          <p className={styles.subtitle}>Our commitment to financial integrity and security.</p>
        </header>

        <div className={styles.contentGrid}>
          <aside className={styles.sidebar}>
            <nav className={styles.stickyNav}>
              <h3>Sections</h3>
              <ul>
                <li><a href="#overview">Overview</a></li>
                <li><a href="#framework">Regulatory Framework</a></li>
                <li><a href="#security-standards">Security Standards</a></li>
                <li><a href="#monitoring">AML/KYC Monitoring</a></li>
              </ul>
            </nav>
          </aside>

          <main className={styles.mainContent}>
            <section id="overview" className={styles.section}>
              <h2>Compliance Overview</h2>
              <p>
                FidduPay maintains the highest standards of regulatory compliance across all jurisdictions where we operate. We believe that transparency and strict adherence to financial regulations are the cornerstones of a trusted crypto ecosystem.
              </p>
            </section>

            <section id="framework" className={styles.section}>
              <h2>Regulatory Framework</h2>
              <div className={styles.complianceGrid}>
                <div className={styles.complianceCard}>
                  <div className={styles.cardIcon}><i className="fas fa-shield-alt"></i></div>
                  <h3>Security Best Practices</h3>
                  <p>Guided by industry-standard security controls and rigorous internal operational procedures.</p>
                </div>
                <div className={styles.complianceCard}>
                  <div className={styles.cardIcon}><i className="fas fa-lock"></i></div>
                  <h3>Encryption Standards</h3>
                  <p>Adherence to high-level asymmetric encryption for all digital asset interactions and keys.</p>
                </div>
                <div className={styles.complianceCard}>
                  <div className={styles.cardIcon}><i className="fas fa-balance-scale"></i></div>
                  <h3>Tiered KYC Framework</h3>
                  <p>Infrastructure ready for tiered identity verification to meet future regulatory requirements.</p>
                </div>
                <div className={styles.complianceCard}>
                  <div className={styles.cardIcon}><i className="fas fa-globe"></i></div>
                  <h3>Privacy Focused</h3>
                  <p>Built with data minimization and user privacy as core architectural principles.</p>
                </div>
              </div>
            </section>

            <section id="security-standards" className={styles.section}>
              <h2>Security Standards</h2>
              <p>Our infrastructure is benchmarked against the most rigorous international security frameworks:</p>
              <ul>
                <li><strong>ISO 27001:</strong> Information Security Management System certification.</li>
                <li><strong>ISO 27017:</strong> Cloud-specific security controls for decentralized infrastructure.</li>
                <li><strong>NIST Framework:</strong> Continuous cybersecurity alignment with US federal standards.</li>
                <li><strong>OWASP Top 10:</strong> Advanced protection against common web and API vulnerabilities.</li>
              </ul>
            </section>

            <section id="monitoring" className={styles.section}>
                <h2>Continuous Monitoring</h2>
                <p>
                    We employ real-time L3 blockchain monitoring to detect suspicious on-chain activity. This automated layer ensures that our gateway remains a secure environment for legitimate business transactions while excluding high-risk or sanctioned entities.
                </p>
                <div className={styles.contactLegal}>
                    <p>For development and partnership inquiries: <strong>support@fiddupay.com</strong></p>
                </div>
            </section>
          </main>
        </div>
      </div>
    </div>
  )
}

export default CompliancePage
