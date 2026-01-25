import React from 'react'
import { Link } from 'react-router-dom'
import styles from './PricingPage.module.css'

const PricingPage: React.FC = () => {
  return (
    <div className={styles.container}>
      {/* Header */}
      <header className={styles.header}>
        <div className={styles.nav}>
          <Link to="/" className={styles.logo}>
            <h2>FidduPay</h2>
            <span>by TechyTro</span>
          </Link>
          <div className={styles.navLinks}>
            <Link to="/">Home</Link>
            <Link to="/about">About</Link>
            <Link to="/contact">Contact</Link>
            <Link to="/login" className={styles.loginBtn}>Login</Link>
          </div>
        </div>
      </header>

      {/* Hero */}
      <section className={styles.hero}>
        <div className={styles.heroContent}>
          <h1>Simple, Transparent Pricing</h1>
          <p>No setup fees. No monthly fees. Pay only when you earn.</p>
        </div>
      </section>

      {/* Pricing */}
      <section className={styles.pricing}>
        <div className={styles.pricingGrid}>
          <div className={styles.pricingCard}>
            <h3>Starter</h3>
            <div className={styles.price}>
              <span className={styles.priceNumber}>2.9%</span>
              <span className={styles.priceUnit}>per transaction</span>
            </div>
            <ul className={styles.features}>
              <li>✓ All supported cryptocurrencies</li>
              <li>✓ Automatic forwarding</li>
              <li>✓ Basic webhook notifications</li>
              <li>✓ API access</li>
              <li>✓ Email support</li>
            </ul>
            <Link to="/register" className={styles.pricingBtn}>Get Started</Link>
          </div>

          <div className={`${styles.pricingCard} ${styles.popular}`}>
            <div className={styles.popularBadge}>Most Popular</div>
            <h3>Business</h3>
            <div className={styles.price}>
              <span className={styles.priceNumber}>2.4%</span>
              <span className={styles.priceUnit}>per transaction</span>
            </div>
            <ul className={styles.features}>
              <li>✓ Everything in Starter</li>
              <li>✓ Advanced webhooks</li>
              <li>✓ Priority support</li>
              <li>✓ Custom integration help</li>
              <li>✓ Volume discounts</li>
            </ul>
            <Link to="/register" className={styles.pricingBtn}>Start Free Trial</Link>
          </div>

          <div className={styles.pricingCard}>
            <h3>Enterprise</h3>
            <div className={styles.price}>
              <span className={styles.priceNumber}>Custom</span>
              <span className={styles.priceUnit}>pricing</span>
            </div>
            <ul className={styles.features}>
              <li>✓ Everything in Business</li>
              <li>✓ Dedicated support</li>
              <li>✓ Custom features</li>
              <li>✓ SLA guarantees</li>
              <li>✓ White-label options</li>
            </ul>
            <Link to="/contact" className={styles.pricingBtn}>Contact Sales</Link>
          </div>
        </div>
      </section>

      {/* Footer */}
      <footer className={styles.footer}>
        <div className={styles.footerContent}>
          <p>&copy; 2026 TechyTro Software. All rights reserved.</p>
        </div>
      </footer>
    </div>
  )
}

export default PricingPage
