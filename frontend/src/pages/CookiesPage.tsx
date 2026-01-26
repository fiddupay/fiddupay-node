import React from 'react'
import styles from './CookiesPage.module.css'

const CookiesPage: React.FC = () => {
  return (
    <div className={styles.cookiesPage}>
      <div className={styles.container}>
        <div className={styles.content}>
          <h1>Cookie Policy</h1>
          <p className={styles.lead}>
            This Cookie Policy explains how FidduPay uses cookies and similar technologies when you visit our website.
          </p>
          <p className={styles.lastUpdated}>Last updated: January 26, 2026</p>

          <section className={styles.section}>
            <h2>What Are Cookies</h2>
            <p>
              Cookies are small text files that are placed on your device when you visit our website. 
              They help us provide you with a better experience by remembering your preferences and 
              understanding how you use our services.
            </p>
          </section>

          <section className={styles.section}>
            <h2>Types of Cookies We Use</h2>
            
            <div className={styles.cookieType}>
              <h3><i className="fas fa-cog"></i> Essential Cookies</h3>
              <p>These cookies are necessary for the website to function properly. They enable core functionality such as security, network management, and accessibility.</p>
              <ul>
                <li>Authentication and session management</li>
                <li>Security and fraud prevention</li>
                <li>Load balancing and performance</li>
              </ul>
            </div>

            <div className={styles.cookieType}>
              <h3><i className="fas fa-chart-line"></i> Analytics Cookies</h3>
              <p>These cookies help us understand how visitors interact with our website by collecting and reporting information anonymously.</p>
              <ul>
                <li>Page views and user journeys</li>
                <li>Performance metrics</li>
                <li>Error tracking and debugging</li>
              </ul>
            </div>

            <div className={styles.cookieType}>
              <h3><i className="fas fa-user"></i> Functional Cookies</h3>
              <p>These cookies enable enhanced functionality and personalization, such as remembering your preferences.</p>
              <ul>
                <li>Language and region preferences</li>
                <li>Theme and display settings</li>
                <li>Form data and user inputs</li>
              </ul>
            </div>
          </section>

          <section className={styles.section}>
            <h2>Managing Your Cookie Preferences</h2>
            <p>
              You can control and manage cookies in various ways. Most web browsers automatically accept cookies, 
              but you can modify your browser settings to decline cookies if you prefer.
            </p>
            
            <div className={styles.browserList}>
              <h3>Browser Settings:</h3>
              <ul>
                <li><strong>Chrome:</strong> Settings → Privacy and Security → Cookies</li>
                <li><strong>Firefox:</strong> Preferences → Privacy & Security → Cookies</li>
                <li><strong>Safari:</strong> Preferences → Privacy → Cookies</li>
                <li><strong>Edge:</strong> Settings → Cookies and Site Permissions</li>
              </ul>
            </div>
          </section>

          <section className={styles.section}>
            <h2>Third-Party Cookies</h2>
            <p>
              We may use third-party services that set their own cookies. These services include:
            </p>
            <ul>
              <li>Google Analytics for website analytics</li>
              <li>Cloudflare for security and performance</li>
              <li>Stripe for payment processing</li>
            </ul>
          </section>

          <section className={styles.section}>
            <h2>Contact Us</h2>
            <p>
              If you have any questions about our Cookie Policy, please contact us at{' '}
              <a href="mailto:privacy@fiddupay.com">privacy@fiddupay.com</a>
            </p>
          </section>
        </div>
      </div>
    </div>
  )
}

export default CookiesPage
