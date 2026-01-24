import React from 'react'
import { Link } from 'react-router-dom'
import styles from './HomePage.module.css'

const HomePage: React.FC = () => {
  return (
    <div className={styles.homePage}>
      {/* Hero Section */}
      <section className={styles.hero}>
        <div className={styles.container}>
          <div className={styles.heroContent}>
            <h1 className={styles.heroTitle}>
              Accept Crypto Payments
              <span className={styles.highlight}> Instantly</span>
            </h1>
            <p className={styles.heroDescription}>
              Modern payment gateway for cryptocurrency transactions. 
              Accept SOL, USDT across multiple networks with automatic forwarding and real-time notifications.
            </p>
            <div className={styles.heroActions}>
              <Link to="/login" className={styles.primaryBtn}>
                Get Started
              </Link>
              <Link to="/pricing" className={styles.secondaryBtn}>
                View Pricing
              </Link>
            </div>
          </div>
          <div className={styles.heroVisual}>
            <div className={styles.card}>
              <div className={styles.cardHeader}>
                <div className={styles.dot}></div>
                <div className={styles.dot}></div>
                <div className={styles.dot}></div>
              </div>
              <div className={styles.cardContent}>
                <div className={styles.payment}>
                  <div className={styles.paymentIcon}>₿</div>
                  <div className={styles.paymentDetails}>
                    <div className={styles.paymentAmount}>$1,250.00</div>
                    <div className={styles.paymentStatus}>Confirmed</div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* Features Section */}
      <section className={styles.features}>
        <div className={styles.container}>
          <h2 className={styles.sectionTitle}>Why Choose PayFlow?</h2>
          <div className={styles.featuresGrid}>
            <div className={styles.feature}>
              <div className={styles.featureIcon}>⚡</div>
              <h3>Instant Processing</h3>
              <p>Real-time payment confirmation and automatic forwarding to your wallets</p>
            </div>
            <div className={styles.feature}>
              <div className={styles.featureIcon}>🔒</div>
              <h3>Bank-Level Security</h3>
              <p>Advanced encryption, rate limiting, and threat detection protect your transactions</p>
            </div>
            <div className={styles.feature}>
              <div className={styles.featureIcon}>🌐</div>
              <h3>Multi-Network Support</h3>
              <p>Accept USDT on Ethereum, BSC, Polygon, Arbitrum, and Solana networks</p>
            </div>
            <div className={styles.feature}>
              <div className={styles.featureIcon}>📊</div>
              <h3>Real-Time Dashboard</h3>
              <p>Monitor payments, track analytics, and manage your crypto business</p>
            </div>
          </div>
        </div>
      </section>

      {/* CTA Section */}
      <section className={styles.cta}>
        <div className={styles.container}>
          <div className={styles.ctaContent}>
            <h2>Ready to Start Accepting Crypto?</h2>
            <p>Join thousands of businesses using PayFlow for secure cryptocurrency payments</p>
            <Link to="/login" className={styles.ctaBtn}>
              Start Free Trial
            </Link>
          </div>
        </div>
      </section>
    </div>
  )
}

export default HomePage
