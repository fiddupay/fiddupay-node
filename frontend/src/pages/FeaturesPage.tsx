import React from 'react'
import styles from './FeaturesPage.module.css'

const FeaturesPage: React.FC = () => {
  return (
    <div className={styles.featuresPage}>
      <div className={styles.container}>
        <div className={styles.header}>
          <h1 className={styles.title}>Powerful Features for Modern Businesses</h1>
          <p className={styles.subtitle}>
            Everything you need to accept cryptocurrency payments securely and efficiently
          </p>
        </div>

        <div className={styles.features}>
          <div className={styles.feature}>
            <div className={styles.featureIcon}>
              <i className="fas fa-bolt"></i>
            </div>
            <h3>Lightning Fast Processing</h3>
            <p>Real-time payment confirmation and automatic forwarding to your wallets within seconds</p>
          </div>

          <div className={styles.feature}>
            <div className={styles.featureIcon}>
              <i className="fas fa-shield-alt"></i>
            </div>
            <h3>Bank-Level Security</h3>
            <p>Advanced encryption, rate limiting, and real-time threat detection protect every transaction</p>
          </div>

          <div className={styles.feature}>
            <div className={styles.featureIcon}>
              <i className="fas fa-network-wired"></i>
            </div>
            <h3>Multi-Network Support</h3>
            <p>Accept USDT on Ethereum, BSC, Polygon, Arbitrum, and Solana networks</p>
          </div>

          <div className={styles.feature}>
            <div className={styles.featureIcon}>
              <i className="fas fa-chart-line"></i>
            </div>
            <h3>Advanced Analytics</h3>
            <p>Real-time dashboard with detailed transaction analytics and business insights</p>
          </div>

          <div className={styles.feature}>
            <div className={styles.featureIcon}>
              <i className="fas fa-webhook"></i>
            </div>
            <h3>Webhook Integration</h3>
            <p>Instant notifications and seamless integration with your existing systems</p>
          </div>

          <div className={styles.feature}>
            <div className={styles.featureIcon}>
              <i className="fas fa-mobile-alt"></i>
            </div>
            <h3>Mobile Optimized</h3>
            <p>Responsive design works perfectly on all devices and screen sizes</p>
          </div>
        </div>
      </div>
    </div>
  )
}

export default FeaturesPage
