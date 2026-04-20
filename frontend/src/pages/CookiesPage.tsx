import React from 'react'
import styles from '@/styles/pages/CookiesPage.module.css'

const CookiesPage: React.FC = () => {
  return (
    <div className={styles.legalPage}>
      {/* Ambient Glow */}
      <div className={styles.ambientGlowContainer}>
        <div className={`${styles.blob} ${styles.blobPrimary}`}></div>
        <div className={`${styles.blob} ${styles.blobSecondary}`}></div>
      </div>

      <div className={styles.container}>
        <header className={`${styles.header} animate-fade-in-up`}>
          <div className={styles.badge}>Transparency</div>
          <h1 className={styles.title}>Cookie Policy</h1>
          <p className={styles.subtitle}>Last updated: January 26, 2026</p>
        </header>

        <div className={styles.contentGrid}>
          <aside className={styles.sidebar}>
            <nav className={styles.stickyNav}>
              <h3>Sections</h3>
              <ul>
                <li><a href="#introduction">What are Cookies</a></li>
                <li><a href="#usage">Usage</a></li>
                <li><a href="#essential">Essential</a></li>
                <li><a href="#analytics">Analytics</a></li>
                <li><a href="#management">Management</a></li>
              </ul>
            </nav>
          </aside>

          <main className={styles.mainContent}>
            <section id="introduction" className={styles.section}>
              <h2>1. What Are Cookies</h2>
              <p>
                Cookies are small text files that are placed on your device when you visit our website. They help us provide you with a better experience by remembering your preferences and understanding how you use our services.
              </p>
            </section>

            <section id="usage" className={styles.section}>
              <h2>2. How We Use Them</h2>
              <p>
                We use cookies to maintain your session security, remember your merchant preferences, and analyze our platform's performance to ensure 99.9% uptime.
              </p>
            </section>

            <section id="essential" className={styles.section}>
              <h2>3. Essential Cookies</h2>
              <div className={styles.cookieTypeCard}>
                <div className={styles.typeIcon}><i className="fas fa-cog"></i></div>
                <div className={styles.typeText}>
                    <h3>System Foundations</h3>
                    <p>These cookies are strictly necessary for the gateway to function. They enable core functionality such as user authentication, security, and session management.</p>
                </div>
              </div>
            </section>

            <section id="analytics" className={styles.section}>
              <h2>4. Analytics & Performance</h2>
              <div className={styles.cookieTypeCard}>
                <div className={styles.typeIcon}><i className="fas fa-chart-line"></i></div>
                <div className={styles.typeText}>
                    <h3>Performance Monitoring</h3>
                    <p>These cookies help us understand how visitors interact with our website by collecting and reporting information anonymously to prevent service degradation.</p>
                </div>
              </div>
            </section>

            <section id="management" className={styles.section}>
              <h2>5. Managing Preferences</h2>
              <p>
                You can control and manage cookies in various ways. Most web browsers automatically accept cookies, but you can modify your browser settings to decline them.
              </p>
              <div className={styles.browserSupport}>
                <ul>
                  <li><strong>Chrome:</strong> Settings → Privacy and Security → Cookies</li>
                  <li><strong>Firefox:</strong> Preferences → Privacy & Security → Cookies</li>
                  <li><strong>Safari:</strong> Preferences → Privacy → Cookies</li>
                </ul>
              </div>
              <div className={styles.contactLegal}>
                  <p>For inquiries about cookies, please contact: <strong>privacy@fiddupay.com</strong></p>
              </div>
            </section>
          </main>
        </div>
      </div>
    </div>
  )
}

export default CookiesPage
