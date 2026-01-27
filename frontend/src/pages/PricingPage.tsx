import React, { useState, useEffect } from 'react'
import { Link } from 'react-router-dom'
import styles from './PricingPage.module.css'

interface PricingData {
  transaction_fee_percentage: number
  daily_volume_limit_non_kyc_usd: string
  supported_networks: number
}

const PricingPage: React.FC = () => {
  const [pricingData, setPricingData] = useState<PricingData>({
    transaction_fee_percentage: 0.75,
    daily_volume_limit_non_kyc_usd: '1000.00',
    supported_networks: 5
  })

  useEffect(() => {
    loadPricingData()
  }, [])

  const loadPricingData = async () => {
    try {
      const response = await fetch('/api/v1/pricing')
      if (response.ok) {
        const data = await response.json()
        setPricingData(data)
      }
    } catch (error) {
      console.error('Failed to load pricing data:', error)
      // Use default values if API fails
    }
  }
  return (
    <div className={styles.pricingPage}>
      <div className={styles.container}>
        <div className={styles.header}>
          <h1 className={styles.title}>Simple, Transparent Pricing</h1>
          <p className={styles.subtitle}>
            Start accepting crypto payments today with our straightforward pricing. 
            No hidden fees, no setup costs, no monthly subscriptions.
          </p>
        </div>

        <div className={styles.plans}>
          <div className={styles.plan}>
            <div className={styles.planBadge}>Most Popular</div>
            <div className={styles.planHeader}>
              <h3 className={styles.planName}>Pay-Per-Use</h3>
              <div className={styles.planPrice}>
                <span className={styles.price}>{pricingData.transaction_fee_percentage}%</span>
                <span className={styles.period}>per successful transaction</span>
              </div>
              <p className={styles.planDescription}>
                Perfect for businesses of all sizes. Only pay when you receive payments.
              </p>
            </div>
            
            <div className={styles.planFeatures}>
              <h4>What's included:</h4>
              <ul className={styles.features}>
                <li><i className="fas fa-check"></i> {pricingData.transaction_fee_percentage}% transaction fee</li>
                <li><i className="fas fa-check"></i> No setup or monthly fees</li>
                <li><i className="fas fa-check"></i> {pricingData.supported_networks} blockchain networks</li>
                <li><i className="fas fa-check"></i> Real-time processing</li>
                <li><i className="fas fa-check"></i> Advanced dashboard</li>
                <li><i className="fas fa-check"></i> Webhook notifications</li>
                <li><i className="fas fa-check"></i> API access</li>
                <li><i className="fas fa-check"></i> Email support</li>
              </ul>
            </div>
            
            <Link to="/register" className={`${styles.planBtn} ${styles.primaryBtn}`}>
              <i className="fas fa-arrow-right"></i>
              Start Accepting Payments
            </Link>
          </div>

          <div className={styles.plan}>
            <div className={styles.planHeader}>
              <h3 className={styles.planName}>Enterprise</h3>
              <div className={styles.planPrice}>
                <span className={styles.price}>Custom</span>
                <span className={styles.period}>volume-based pricing</span>
              </div>
              <p className={styles.planDescription}>
                For high-volume businesses with custom requirements and dedicated support.
              </p>
            </div>
            
            <div className={styles.planFeatures}>
              <h4>Everything in Pay-Per-Use, plus:</h4>
              <ul className={styles.features}>
                <li><i className="fas fa-check"></i> Volume discounts available</li>
                <li><i className="fas fa-check"></i> Dedicated account manager</li>
                <li><i className="fas fa-check"></i> Priority support (24/7)</li>
                <li><i className="fas fa-check"></i> Custom integrations</li>
                <li><i className="fas fa-check"></i> Advanced analytics</li>
                <li><i className="fas fa-check"></i> White-label options</li>
                <li><i className="fas fa-check"></i> SLA guarantees</li>
                <li><i className="fas fa-check"></i> Custom reporting</li>
                <li><i className="fas fa-check"></i> Multi-user accounts</li>
                <li><i className="fas fa-check"></i> Phone support</li>
              </ul>
            </div>
            
            <a href="mailto:sales@fiddupay.com" className={`${styles.planBtn} ${styles.secondaryBtn}`}>
              <i className="fas fa-envelope"></i>
              Contact Sales
            </a>
          </div>
        </div>

        <div className={styles.faq}>
          <h2>Frequently Asked Questions</h2>
          <div className={styles.faqGrid}>
            <div className={styles.faqItem}>
              <h3><i className="fas fa-question-circle"></i> Are there any hidden fees?</h3>
              <p>No hidden fees whatsoever. You only pay the {pricingData.transaction_fee_percentage}% transaction fee on successful payments. No setup fees, monthly fees, or cancellation fees.</p>
            </div>
            <div className={styles.faqItem}>
              <h3><i className="fas fa-question-circle"></i> When do I get charged?</h3>
              <p>You're only charged when you successfully receive a payment. Failed or expired payments are never charged.</p>
            </div>
            <div className={styles.faqItem}>
              <h3><i className="fas fa-question-circle"></i> What cryptocurrencies do you support?</h3>
              <p>We support SOL, ETH, BNB, MATIC, ARB, and USDT across Ethereum, BSC, Polygon, Arbitrum, and Solana networks.</p>
            </div>
            <div className={styles.faqItem}>
              <h3><i className="fas fa-question-circle"></i> Can I change plans later?</h3>
              <p>Yes! You can upgrade to Enterprise at any time. Contact our sales team to discuss volume discounts and custom pricing.</p>
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}

export default PricingPage
