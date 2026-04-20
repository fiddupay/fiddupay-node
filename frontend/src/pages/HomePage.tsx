import styles from '@/styles/pages/HomePage.module.css';
import React, { useState } from 'react';
import { Link } from 'react-router-dom';

const HomePage: React.FC = () => {
  const [activeFaq, setActiveFaq] = useState<number | null>(null);

  const toggleFaq = (index: number) => {
    if (activeFaq === index) {
      setActiveFaq(null);
    } else {
      setActiveFaq(index);
    }
  };

  return (
    <div className={styles.homePage}>
      {/* Background Ambient Glow */}
      <div className={styles.ambientGlowContainer}>
        <div className={`${styles.blob} ${styles.blobPrimary}`}></div>
        <div className={`${styles.blob} ${styles.blobSecondary}`}></div>
        <div className={`${styles.blob} ${styles.blobIndigo}`}></div>
      </div>

      <div className={styles.container}>
        
        {/* Hero Section */}
        <section className={styles.heroSection}>
          <div className={`${styles.heroTextContent} animate-fade-in-up`}>
            <div className={styles.badge}>
              <span className={styles.badgePing}></span>
              <span className={styles.badgeDot}></span>
              V1.0 Now Live: Lightning Fast Payments
            </div>
            
            <h1 className={styles.heroTitle}>
              Accept Crypto <br />
              <span className={styles.heroTitleGradient}>Without Limits.</span>
            </h1>
            
            <p className={styles.heroSubtitle}>
              Empower your business with FidduPay. Accept BTC, ETH, SOL, and Stablecoins seamlessly across 6 global blockchain networks with instant settlement and absolute zero chargebacks.
            </p>
            
            <div className={styles.heroActions}>
              <Link to="/login" className={styles.btnPrimary}>
                Get Started Free
              </Link>
              <Link to="/docs" className={styles.btnSecondary}>
                Read Documentation
              </Link>
            </div>
            
            <div className={styles.heroChecks}>
              <div><i className="fas fa-check"></i> No setup fees</div>
              <div><i className="fas fa-check"></i> API Integrations</div>
            </div>
          </div>
          
          <div className={`${styles.heroVisual} animate-slide-in-right`}>
            <div className={styles.heroImageWrapper}>
              <img 
                src="/hero3.png" 
                alt="FidduPay Dashboard floating with crypto coins" 
                className={styles.heroImage}
                onError={(e) => {
                  e.currentTarget.src = "https://images.unsplash.com/photo-1639762681485-074b7f4ec651?ixlib=rb-4.0.3&auto=format&fit=crop&w=1200&q=80";
                }}
              />
            </div>
            
            <div className={styles.floatingCard}>
              <div className={styles.floatingCardIcon}>
                <img src="/logo/logo-symbol.svg" alt="FidduPay" style={{ width: '24px', height: '24px' }} />
              </div>
              <div className={styles.floatingCardText}>
                <span className={styles.floatingCardLabel}>Payment Received</span>
                <span className={styles.floatingCardAmount}>+1.25 ETH</span>
              </div>
            </div>
          </div>
        </section>

        {/* Supported Networks */}
        <section className={styles.networksSection}>
          <div className={styles.sectionHeader}>
            <h2 className={styles.sectionTitle}>Supported Networks</h2>
            <p className={styles.sectionSubtitle}>Process payments natively across leading Layer 1 and Layer 2 chains.</p>
          </div>
          
          <div className={styles.networksGrid}>
            {[
              { name: 'Bitcoin', symbol: 'BTC', img: 'https://raw.githubusercontent.com/spothq/cryptocurrency-icons/master/128/color/btc.png' },
              { name: 'Ethereum', symbol: 'ETH', img: 'https://raw.githubusercontent.com/spothq/cryptocurrency-icons/master/128/color/eth.png' },
              { name: 'Solana', symbol: 'SOL', img: 'https://raw.githubusercontent.com/spothq/cryptocurrency-icons/master/128/color/sol.png' },
              { name: 'Binance', symbol: 'BNB', img: 'https://raw.githubusercontent.com/spothq/cryptocurrency-icons/master/128/color/bnb.png' },
              { name: 'Polygon', symbol: 'MATIC', img: 'https://raw.githubusercontent.com/spothq/cryptocurrency-icons/master/128/color/matic.png' },
              { name: 'Tether', symbol: 'USDT', img: 'https://raw.githubusercontent.com/spothq/cryptocurrency-icons/master/128/color/usdt.png' },
              { name: 'Arbitrum', symbol: 'ARB', img: '/arbitrum-arb-logo.png' },
              { name: 'USD Coin', symbol: 'USDC', img: 'https://raw.githubusercontent.com/spothq/cryptocurrency-icons/master/128/color/usdc.png' }
            ].map((coin, index) => (
              <div key={index} className={styles.networkCard}>
                <img src={coin.img} alt={coin.name} className={styles.networkIcon} />
                <h3 className={styles.networkName}>{coin.name}</h3>
                <p className={styles.networkSymbol}>{coin.symbol}</p>
              </div>
            ))}
          </div>
        </section>

        {/* Features */}
        <section className={styles.featuresSection}>
          <div className={styles.sectionHeader}>
            <h2 className={styles.sectionTitle}>Built for the Modern Web</h2>
            <p className={styles.sectionSubtitle}>Everything you need to scale your revenue globally without the traditional banking barriers.</p>
          </div>

          <div className={styles.featuresGrid}>
            <div className={styles.featureCard}>
               <div className={styles.featureWatermark}><i className="fas fa-bolt"></i></div>
               <div className={`${styles.featureIcon} ${styles.iconPrimary}`}><i className="fas fa-bolt"></i></div>
               <h3 className={styles.featureTitle}>Instant Settlements</h3>
               <p className={styles.featureDesc}>Transactions are verified dynamically and funds are swept instantly to your secure cold or hot wallets.</p>
            </div>

            <div className={styles.featureCard}>
               <div className={styles.featureWatermark}><i className="fas fa-shield-halved"></i></div>
               <div className={`${styles.featureIcon} ${styles.iconSecondary}`}><i className="fas fa-shield-halved"></i></div>
               <h3 className={styles.featureTitle}>Bank-Grade Security</h3>
               <p className={styles.featureDesc}>Built with robust cryptography. Enjoy absolute control of your keys and robust DDoS protection on our API endpoints.</p>
            </div>

            <div className={styles.featureCard}>
               <div className={styles.featureWatermark}><i className="fas fa-code"></i></div>
               <div className={`${styles.featureIcon} ${styles.iconPrimary}`}><i className="fas fa-code"></i></div>
               <h3 className={styles.featureTitle}>Developer API & SDKs</h3>
               <p className={styles.featureDesc}>Integrate natively using our NodeJS, Python, or Go SDKs. Complete documentation with ready-to-deploy webhooks.</p>
            </div>
          </div>
        </section>

        {/* FAQ Section */}
        <section className={styles.faqSection}>
          <div className={styles.sectionHeader}>
            <h2 className={styles.sectionTitle}>Frequently Asked Questions</h2>
          </div>
          <div className={styles.faqList}>
            {[
              { q: "How fast are payments processed?", a: "Crypto payments are confirmed as fast as the native blockchain allows." },
              { q: "Do you charge setup or monthly fees?", a: "No! FidduPay operates on a transparent transaction fee model." },
              { q: "Can I receive payments directly to my cold wallet?", a: "Absolutely. Our auto-forwarding engine sweeps your received funds directly." },
              { q: "Are there chargebacks?", a: "Never. Cryptocurrency transactions are immutable." },
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
        </section>

        {/* CTA */}
        <section className={styles.ctaSection}>
          <div className={styles.ctaCard}>
            <h2 className={styles.ctaTitle}>Ready to modernize your checkout?</h2>
            <p className={styles.ctaSubtitle}>Join thousands of forward-thinking merchants accepting global payments.</p>
            <div className={styles.ctaActions}>
              <Link to="/register" className={styles.btnWhite}>Create Free Account</Link>
              <Link to="/contact" className={styles.btnOutline}>Contact Sales</Link>
            </div>
          </div>
        </section>

      </div>
    </div>
  )
}

export default HomePage
