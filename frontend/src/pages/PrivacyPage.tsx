import React from 'react'
import styles from './PrivacyPage.module.css'

const PrivacyPage: React.FC = () => {
  return (
    <div className={styles.privacyPage}>
      <div className={styles.container}>
        <div className={styles.header}>
          <h1 className={styles.title}>Privacy Policy</h1>
          <p className={styles.subtitle}>Last updated: January 24, 2026</p>
        </div>

        <div className={styles.content}>
          <section className={styles.section}>
            <h2>1. Information We Collect</h2>
            <p>
              FidduPay collects information necessary to provide our cryptocurrency payment services:
            </p>
            <div className={styles.subsection}>
              <h3>Account Information</h3>
              <ul>
                <li>Name, email address, and contact details</li>
                <li>Company information and business details</li>
                <li>Cryptocurrency wallet addresses</li>
                <li>API keys and authentication credentials</li>
              </ul>
            </div>
            <div className={styles.subsection}>
              <h3>Transaction Data</h3>
              <ul>
                <li>Payment amounts and cryptocurrency types</li>
                <li>Transaction timestamps and status</li>
                <li>Blockchain transaction hashes</li>
                <li>Customer payment information (when provided)</li>
              </ul>
            </div>
          </section>

          <section className={styles.section}>
            <h2>2. How We Use Your Information</h2>
            <p>
              We use collected information to:
            </p>
            <ul>
              <li>Process cryptocurrency payments and transactions</li>
              <li>Provide customer support and technical assistance</li>
              <li>Monitor for fraud and ensure security</li>
              <li>Comply with legal and regulatory requirements</li>
              <li>Improve our services and develop new features</li>
              <li>Send important service notifications and updates</li>
            </ul>
          </section>

          <section className={styles.section}>
            <h2>3. Information Sharing</h2>
            <p>
              FidduPay does not sell or rent your personal information. We may share information only in these circumstances:
            </p>
            <ul>
              <li>With your explicit consent</li>
              <li>To comply with legal obligations or court orders</li>
              <li>To prevent fraud or protect our rights</li>
              <li>With service providers who assist our operations</li>
              <li>In connection with a business transfer or merger</li>
            </ul>
          </section>

          <section className={styles.section}>
            <h2>4. Data Security</h2>
            <p>
              We implement comprehensive security measures to protect your information:
            </p>
            <ul>
              <li>End-to-end encryption for sensitive data</li>
              <li>Secure data centers with physical access controls</li>
              <li>Regular security audits and penetration testing</li>
              <li>Employee access controls and training</li>
              <li>Incident response and breach notification procedures</li>
            </ul>
          </section>

          <section className={styles.section}>
            <h2>5. Data Retention</h2>
            <p>
              We retain your information for as long as necessary to:
            </p>
            <ul>
              <li>Provide our services to you</li>
              <li>Comply with legal and regulatory requirements</li>
              <li>Resolve disputes and enforce agreements</li>
              <li>Maintain business records for tax purposes</li>
            </ul>
            <p>
              Transaction data is typically retained for 7 years to comply with financial regulations.
            </p>
          </section>

          <section className={styles.section}>
            <h2>6. Your Rights</h2>
            <p>
              Depending on your location, you may have the following rights:
            </p>
            <ul>
              <li>Access your personal information we hold</li>
              <li>Correct inaccurate or incomplete information</li>
              <li>Delete your personal information (subject to legal requirements)</li>
              <li>Restrict or object to certain processing activities</li>
              <li>Data portability for information you provided</li>
              <li>Withdraw consent where processing is based on consent</li>
            </ul>
          </section>

          <section className={styles.section}>
            <h2>7. Cookies and Tracking</h2>
            <p>
              FidduPay uses cookies and similar technologies to:
            </p>
            <ul>
              <li>Maintain your login session</li>
              <li>Remember your preferences</li>
              <li>Analyze website usage and performance</li>
              <li>Provide personalized experiences</li>
            </ul>
            <p>
              You can control cookie settings through your browser preferences.
            </p>
          </section>

          <section className={styles.section}>
            <h2>8. International Transfers</h2>
            <p>
              Your information may be transferred to and processed in countries other than your own. We ensure appropriate safeguards are in place, including:
            </p>
            <ul>
              <li>Standard contractual clauses</li>
              <li>Adequacy decisions by relevant authorities</li>
              <li>Certification schemes and codes of conduct</li>
            </ul>
          </section>

          <section className={styles.section}>
            <h2>9. Contact Us</h2>
            <p>
              For privacy-related questions or to exercise your rights, contact us at:
            </p>
            <div className={styles.contactInfo}>
              <p>Email: privacy@fiddupay.com</p>
              <p>Subject: Privacy Inquiry</p>
              <p>Response Time: Within 30 days</p>
            </div>
          </section>
        </div>
      </div>
    </div>
  )
}

export default PrivacyPage
