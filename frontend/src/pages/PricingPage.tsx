import React, { useState, useEffect } from 'react'
import { Link } from 'react-router-dom'
import { publicAPI } from '@/services/apiService'
import styles from '@/styles/pages/PricingPage.module.css'
import SEO from '@/components/ui/SEO'

interface PricingData {
  transaction_fee_percentage: number
  daily_volume_limit_non_kyc_usd: string
  supported_networks: number
}

const PricingPage: React.FC = () => {
  const [pricingData, setPricingData] = useState<PricingData>({
    transaction_fee_percentage: 0.75,
    daily_volume_limit_non_kyc_usd: '...',
    supported_networks: 5
  })
  
  const [activeFaq, setActiveFaq] = useState<number | null>(null);

  useEffect(() => {
    loadPricingData()
  }, [])

  const loadPricingData = async () => {
    try {
      const response = await publicAPI.getPricing()
      if (response.status === 200) {
        const data = response.data
        setPricingData({
          transaction_fee_percentage: data.transaction_fee_percentage,
          daily_volume_limit_non_kyc_usd: data.daily_volume_limit_non_kyc_usd,
          supported_networks: data.supported_networks
        })
      }
    } catch (error) {
      console.error('Failed to load pricing data:', error)
    }
  }

  const toggleFaq = (index: number) => {
    setActiveFaq(activeFaq === index ? null : index);
  };

  return (
    <div className={styles.pricingPage}>
      <SEO 
        title="Pricing | FidduPay" 
        description="Transparent and simple pay-per-use pricing. No hidden fees, no setup costs, and no monthly subscriptions for accepting crypto payments."
      />
      {/* Background Ambient Glow */}
      <div className={styles.ambientGlowContainer}>
        <div className={`${styles.blob} ${styles.blobPrimary}`}></div>
        <div className={`${styles.blob} ${styles.blobSecondary}`}></div>
      </div>

      <div className={styles.container}>
        <div className={`${styles.header} animate-fade-in-up`}>
          <h1 className={styles.title}>Simple, Transparent <span className={styles.gradientText}>Pricing</span></h1>
          <p className={styles.subtitle}>
            Start accepting crypto payments today with our straightforward pricing.
            No hidden fees, no setup costs, no monthly subscriptions.
          </p>
        </div>

        <div className={styles.plans}>
          <div className={`${styles.plan} ${styles.planPopular} animate-slide-in-right`} style={{ animationDelay: '0.1s' }}>
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
              Start Accepting Payments
              <i className="fas fa-arrow-right"></i>
            </Link>
          </div>

          <div className={`${styles.plan} animate-slide-in-right`} style={{ animationDelay: '0.3s' }}>
            <div className={styles.planHeader}>
              <h3 className={styles.planName}>Enterprise</h3>
              <div className={styles.planPrice}>
                <span className={styles.priceCustom}>Custom</span>
                <span className={styles.period}>volume-based pricing</span>
              </div>
              <p className={styles.planDescription}>
                For high-volume businesses with custom requirements and dedicated support.
              </p>
            </div>

            <div className={styles.planFeatures}>
              <h4>Everything in Pay-Per-Use, plus:</h4>
              <ul className={styles.features}>
                <li><i className="fas fa-check lineSecondary"></i> Volume discounts available</li>
                <li><i className="fas fa-check lineSecondary"></i> Dedicated account manager</li>
                <li><i className="fas fa-check lineSecondary"></i> Priority support (24/7)</li>
                <li><i className="fas fa-check lineSecondary"></i> Custom integrations</li>
                <li><i className="fas fa-check lineSecondary"></i> Advanced analytics</li>
                <li><i className="fas fa-check lineSecondary"></i> White-label options</li>
                <li><i className="fas fa-check lineSecondary"></i> SLA guarantees</li>
                <li><i className="fas fa-check lineSecondary"></i> Multi-user accounts</li>
              </ul>
            </div>

            <a href="mailto:sales@fiddupay.com" className={`${styles.planBtn} ${styles.secondaryBtn}`}>
              Contact Sales
              <i className="fas fa-envelope"></i>
            </a>
          </div>
        </div>

        <div className={styles.faqSection}>
          <h2 className={styles.faqTitle}>Frequently Asked Questions</h2>
          <div className={styles.faqList}>
             {[
              { q: "Are there any hidden fees?", a: `No hidden fees whatsoever. You only pay the ${pricingData.transaction_fee_percentage}% transaction fee on successful payments. No setup fees, monthly fees, or cancellation fees.` },
              { q: "When do I get charged?", a: "You're only charged when you successfully receive a payment. Failed or expired payments are never charged." },
              { q: "What cryptocurrencies do you support?", a: "We support SOL, ETH, BNB, MATIC, ARB, and USDT across Ethereum, BSC, Polygon, Arbitrum, and Solana networks." },
              { q: "Can I change plans later?", a: "Yes! You can upgrade to Enterprise at any time. Contact our sales team to discuss volume discounts and custom pricing." },
            ].map((faq, index) => (
              <div key={index} className={styles.faqItem}>
                <button className={styles.faqQuestion} onClick={() => toggleFaq(index)}>
                  <span>{faq.q}</span>
                  <i className={`fas fa-chevron-down ${styles.faqChevron} ${activeFaq === index ? styles.faqChevronActive : ''}`}></i>
                </button>
                <div className={`${styles.faqAnswer} ${activeFaq === index ? styles.faqAnswerActive : ''}`}>
                  <p>{faq.a}</p>
                </div>
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  )
}

export default PricingPage
