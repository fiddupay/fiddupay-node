import React from 'react'
import styles from './TermsPage.module.css'

const TermsPage: React.FC = () => {
  return (
    <div className={styles.termsPage}>
      <div className={styles.container}>
        <div className={styles.header}>
          <h1 className={styles.title}>Terms of Service</h1>
          <p className={styles.subtitle}>Last updated: January 24, 2026</p>
        </div>

        <div className={styles.content}>
          <section className={styles.section}>
            <h2>1. Acceptance of Terms</h2>
            <p>
              By accessing and using PayFlow's cryptocurrency payment gateway services, you accept and agree to be bound by the terms and provision of this agreement.
            </p>
          </section>

          <section className={styles.section}>
            <h2>2. Service Description</h2>
            <p>
              PayFlow provides cryptocurrency payment processing services, including but not limited to:
            </p>
            <ul>
              <li>Payment request generation and processing</li>
              <li>Cryptocurrency transaction monitoring</li>
              <li>Automatic payment forwarding to merchant wallets</li>
              <li>Real-time payment notifications via webhooks</li>
              <li>Transaction analytics and reporting</li>
            </ul>
          </section>

          <section className={styles.section}>
            <h2>3. Account Registration</h2>
            <p>
              To use our services, you must create an account and provide accurate, complete information. You are responsible for:
            </p>
            <ul>
              <li>Maintaining the confidentiality of your account credentials</li>
              <li>All activities that occur under your account</li>
              <li>Notifying us immediately of any unauthorized use</li>
              <li>Ensuring your account information remains current and accurate</li>
            </ul>
          </section>

          <section className={styles.section}>
            <h2>4. Fees and Payment</h2>
            <p>
              PayFlow charges a transaction fee of 0.75% per successful payment, with a minimum fee of $0.01 and maximum fee of $200.00. Additional terms:
            </p>
            <ul>
              <li>Fees are automatically deducted from processed payments</li>
              <li>No setup fees or monthly subscription charges</li>
              <li>Enterprise customers may negotiate custom fee structures</li>
              <li>Fee changes will be communicated 30 days in advance</li>
            </ul>
          </section>

          <section className={styles.section}>
            <h2>5. Prohibited Activities</h2>
            <p>
              You agree not to use PayFlow services for:
            </p>
            <ul>
              <li>Illegal activities or transactions</li>
              <li>Money laundering or terrorist financing</li>
              <li>Fraud, scams, or deceptive practices</li>
              <li>Adult content or gambling services</li>
              <li>Violation of any applicable laws or regulations</li>
            </ul>
          </section>

          <section className={styles.section}>
            <h2>6. Security and Compliance</h2>
            <p>
              PayFlow implements industry-standard security measures and complies with applicable regulations. However, you acknowledge that:
            </p>
            <ul>
              <li>Cryptocurrency transactions are irreversible</li>
              <li>You are responsible for securing your wallet addresses</li>
              <li>We may suspend accounts for suspicious activity</li>
              <li>Compliance with local laws is your responsibility</li>
            </ul>
          </section>

          <section className={styles.section}>
            <h2>7. Limitation of Liability</h2>
            <p>
              PayFlow's liability is limited to the fees paid for our services. We are not liable for:
            </p>
            <ul>
              <li>Cryptocurrency price fluctuations</li>
              <li>Network congestion or blockchain delays</li>
              <li>Third-party service interruptions</li>
              <li>Loss of funds due to user error</li>
            </ul>
          </section>

          <section className={styles.section}>
            <h2>8. Termination</h2>
            <p>
              Either party may terminate this agreement at any time. Upon termination:
            </p>
            <ul>
              <li>Your access to services will be suspended</li>
              <li>Pending transactions will be completed</li>
              <li>Account data may be retained for legal compliance</li>
              <li>Outstanding fees remain payable</li>
            </ul>
          </section>

          <section className={styles.section}>
            <h2>9. Contact Information</h2>
            <p>
              For questions about these terms, please contact us at:
            </p>
            <div className={styles.contactInfo}>
              <p>Email: legal@payflow.com</p>
              <p>Address: PayFlow Legal Department</p>
            </div>
          </section>
        </div>
      </div>
    </div>
  )
}

export default TermsPage
