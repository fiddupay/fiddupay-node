import React from 'react'
import styles from '@/styles/pages/TermsPage.module.css'

const TermsPage: React.FC = () => {
  return (
    <div className={styles.legalPage}>
      {/* Ambient Glow */}
      <div className={styles.ambientGlowContainer}>
        <div className={`${styles.blob} ${styles.blobPrimary}`}></div>
        <div className={`${styles.blob} ${styles.blobSecondary}`}></div>
      </div>

      <div className={styles.container}>
        <header className={`${styles.header} animate-fade-in-up`}>
          <div className={styles.badge}>Gateway Usage</div>
          <h1 className={styles.title}>Terms of Service</h1>
          <p className={styles.subtitle}>Last updated: January 24, 2026</p>
        </header>

        <div className={styles.contentGrid}>
          <aside className={styles.sidebar}>
            <nav className={styles.stickyNav}>
              <h3>Sections</h3>
              <ul>
                <li><a href="#acceptance">Acceptance</a></li>
                <li><a href="#services">Service Description</a></li>
                <li><a href="#accounts">Accounts</a></li>
                <li><a href="#fees">Fees & Payments</a></li>
                <li><a href="#prohibited">Prohibited Activities</a></li>
                <li><a href="#security">Security</a></li>
                <li><a href="#liability">Liability</a></li>
              </ul>
            </nav>
          </aside>

          <main className={styles.mainContent}>
            <section id="acceptance" className={styles.section}>
              <h2>1. Acceptance of Terms</h2>
              <p>
                By accessing and using FidduPay's high-performance cryptocurrency payment gateway services, you accept and agree to be bound by the terms and provisions of this agreement.
              </p>
            </section>

            <section id="services" className={styles.section}>
              <h2>2. Service Description</h2>
              <p>
                FidduPay provides advanced cryptocurrency payment processing services, including:
              </p>
              <ul>
                <li>Automated payment request generation and L3 blockchain monitoring.</li>
                <li>Cryptocurrency transaction synchronization across 6+ networks.</li>
                <li>Institutional-grade payment forwarding to self-custody or merchant wallets.</li>
                <li>Real-time JSON webhooks and SSE event notifications.</li>
                <li>Transaction analytics, reporting, and financial reconciliation tools.</li>
              </ul>
            </section>

            <section id="accounts" className={styles.section}>
              <h2>3. Account Registration</h2>
              <p>
                To use our services, you must create an account and provide accurate, complete information. You are responsible for:
              </p>
              <ul>
                <li>Maintaining the strict confidentiality of your account credentials and 2FA secrets.</li>
                <li>All activities that occur under your merchant identity.</li>
                <li>Notifying us immediately of any detected unauthorized use or breach.</li>
                <li>Ensuring your business registration data remains valid and current.</li>
              </ul>
            </section>

            <section id="fees" className={styles.section}>
              <h2>4. Fees and Payment</h2>
              <p>
                FidduPay charges a standard transaction fee of 0.75% per successful settlement. 
              </p>
              <ul>
                <li>Fees are automatically deducted during the blockchain settlement process.</li>
                <li>Zero setup fees or monthly recurring subscription charges for standard merchants.</li>
                <li>Enterprise customers may apply for custom high-volume fee structures.</li>
                <li>Fee modifications will be communicated with a 30-day implementation notice.</li>
              </ul>
            </section>

            <section id="prohibited" className={styles.section}>
              <h2>5. Prohibited Activities</h2>
              <p>
                You agree not to use FidduPay services for:
              </p>
              <ul>
                <li>Illegal activities, money laundering, or terrorist financing.</li>
                <li>Fraudulent schemes, deceptive marketing, or scam operations.</li>
                <li>Adult content, unlicensed gambling, or darknet market transactions.</li>
                <li>Any activity that violates local or international financial regulations.</li>
              </ul>
            </section>

            <section id="security" className={styles.section}>
              <h2>6. Security and Compliance</h2>
              <p>
                FidduPay implements industry-standard encryption and L3 monitoring. However:
              </p>
              <ul>
                <li>Blockchain transactions are final and irreversible once confirmed on-chain.</li>
                <li>Merchant is solely responsible for maintaining secure wallet seeds and private keys.</li>
                <li>We reserve the right to suspend any account flagged for suspicious or high-risk activity.</li>
              </ul>
            </section>

            <section id="liability" className={styles.section}>
              <h2>7. Limitation of Liability</h2>
              <p>
                FidduPay's liability is strictly limited to the cumulative fees paid for our services in the 12 months preceding any claim. We are not liable for blockchain congestion, fork events, or exchange rate volatility.
              </p>
            </section>

            <section id="governing-law" className={styles.section}>
              <h2>8. Governing Law</h2>
              <p>
                These Terms shall be governed by and construed in accordance with the laws of the <b>Federal Republic of Nigeria</b>. 
                Any disputes arising under these terms shall be subject to the exclusive jurisdiction of the 
                courts of Nigeria.
              </p>
              <div className={styles.contactLegal}>
                  <p>For questions about these terms, please contact: <strong>legal@fiddupay.com</strong></p>
              </div>
            </section>
          </main>
        </div>
      </div>
    </div>
  )
}

export default TermsPage
