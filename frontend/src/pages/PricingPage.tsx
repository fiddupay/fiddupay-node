import React from 'react'
import { Link } from 'react-router-dom'
import styles from './PricingPage.module.css'

const PricingPage: React.FC = () => {
  return (
    <div className={styles.pricingPage}>
      <div className={styles.container}>
        <div className={styles.header}>
          <h1 className={styles.title}>Simple, Transparent Pricing</h1>
          <p className={styles.subtitle}>
            Start free, scale as you grow. No hidden fees, no setup costs.
          </p>
        </div>

        <div className={styles.plans}>
          <div className={styles.plan}>
            <div className={styles.planHeader}>
              <h3 className={styles.planName}>Starter</h3>
              <div className={styles.planPrice}>
                <span className={styles.price}>Free</span>
              </div>
              <p className={styles.planDescription}>Perfect for testing and small businesses</p>
            </div>
            <ul className={styles.features}>
              <li>Up to $1,000/month volume</li>
              <li>Basic dashboard</li>
              <li>Email support</li>
              <li>Standard processing speed</li>
            </ul>
            <Link to="/login" className={styles.planBtn}>
              Get Started
            </Link>
          </div>

          <div className={`${styles.plan} ${styles.popular}`}>
            <div className={styles.popularBadge}>Most Popular</div>
            <div className={styles.planHeader}>
              <h3 className={styles.planName}>Professional</h3>
              <div className={styles.planPrice}>
                <span className={styles.price}>2.5%</span>
                <span className={styles.period}>per transaction</span>
              </div>
              <p className={styles.planDescription}>For growing businesses and e-commerce</p>
            </div>
            <ul className={styles.features}>
              <li>Unlimited transaction volume</li>
              <li>Advanced analytics dashboard</li>
              <li>Priority support</li>
              <li>Instant processing</li>
              <li>Webhook notifications</li>
              <li>Multi-network support</li>
            </ul>
            <Link to="/login" className={`${styles.planBtn} ${styles.primaryBtn}`}>
              Start Free Trial
            </Link>
          </div>

          <div className={styles.plan}>
            <div className={styles.planHeader}>
              <h3 className={styles.planName}>Enterprise</h3>
              <div className={styles.planPrice}>
                <span className={styles.price}>Custom</span>
              </div>
              <p className={styles.planDescription}>For large-scale operations</p>
            </div>
            <ul className={styles.features}>
              <li>Custom transaction rates</li>
              <li>Dedicated account manager</li>
              <li>24/7 phone support</li>
              <li>Custom integrations</li>
              <li>SLA guarantees</li>
              <li>White-label options</li>
            </ul>
            <a href="mailto:sales@payflow.com" className={styles.planBtn}>
              Contact Sales
            </a>
          </div>
        </div>

        <div className={styles.faq}>
          <h2>Frequently Asked Questions</h2>
          <div className={styles.faqGrid}>
            <div className={styles.faqItem}>
              <h3>What cryptocurrencies do you support?</h3>
              <p>We support SOL (Solana) and USDT across 5 major networks: Ethereum, BSC, Polygon, Arbitrum, and Solana.</p>
            </div>
            <div className={styles.faqItem}>
              <h3>How fast are transactions processed?</h3>
              <p>Transactions are confirmed in real-time and automatically forwarded to your designated wallets within seconds.</p>
            </div>
            <div className={styles.faqItem}>
              <h3>Is there a setup fee?</h3>
              <p>No setup fees, no monthly fees. You only pay when you process transactions.</p>
            </div>
            <div className={styles.faqItem}>
              <h3>Can I integrate with my existing system?</h3>
              <p>Yes, we provide comprehensive APIs and webhooks for seamless integration with any platform.</p>
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}

export default PricingPage
