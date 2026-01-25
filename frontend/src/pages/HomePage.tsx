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
            <div className={`${styles.heroText} animate-slide-in-left`}>
              <h1 className={styles.heroTitle}>
                Accept Crypto Payments
                <span className={styles.highlight}> Across 5 Networks</span>
              </h1>
              <p className={styles.heroDescription}>
                Modern payment gateway supporting SOL, ETH, BNB, MATIC, ARB, and USDT across 
                Ethereum, BSC, Polygon, Arbitrum, and Solana networks with automatic forwarding.
              </p>
              <div className={styles.heroActions}>
                <Link to="/login" className={`${styles.primaryBtn} hover-lift`}>
                  Get Started
                </Link>
                <Link to="/pricing" className={`${styles.secondaryBtn} hover-lift`}>
                  View Pricing
                </Link>
              </div>
            </div>
            <div className={`${styles.heroVisual} animate-slide-in-right`}>
              <div className={`${styles.card} hover-lift`}>
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
        </div>
      </section>

      {/* Supported Cryptocurrencies Section */}
      <section className={styles.cryptoSection}>
        <div className={styles.container}>
          <h2 className={styles.sectionTitle}>Supported Cryptocurrencies</h2>
          <p className={styles.sectionSubtitle}>Accept payments in multiple cryptocurrencies across 5 major blockchain networks</p>
          
          <div className={styles.cryptoShowcase}>
            <div className={styles.cryptoItem}>
              <img src="https://cryptologos.cc/logos/solana-sol-logo.png" alt="Solana" className={styles.cryptoLogo} />
              <h3>SOL</h3>
              <p>Solana</p>
            </div>
            <div className={styles.cryptoItem}>
              <img src="https://cryptologos.cc/logos/ethereum-eth-logo.png" alt="Ethereum" className={styles.cryptoLogo} />
              <h3>ETH</h3>
              <p>Ethereum</p>
            </div>
            <div className={styles.cryptoItem}>
              <img src="https://cryptologos.cc/logos/bnb-bnb-logo.png" alt="BNB" className={styles.cryptoLogo} />
              <h3>BNB</h3>
              <p>BSC</p>
            </div>
            <div className={styles.cryptoItem}>
              <img src="https://cryptologos.cc/logos/polygon-matic-logo.png" alt="Polygon" className={styles.cryptoLogo} />
              <h3>MATIC</h3>
              <p>Polygon</p>
            </div>
            <div className={styles.cryptoItem}>
              <img src="https://cryptologos.cc/logos/arbitrum-arb-logo.png" alt="Arbitrum" className={styles.cryptoLogo} />
              <h3>ARB</h3>
              <p>Arbitrum</p>
            </div>
            <div className={styles.cryptoItem}>
              <img src="https://cryptologos.cc/logos/tether-usdt-logo.png" alt="USDT" className={styles.cryptoLogo} />
              <h3>USDT</h3>
              <p>5 Networks</p>
            </div>
          </div>
        </div>
      </section>

      {/* Features Section */}
      <section className={styles.features}>
        <div className={styles.container}>
          <h2 className={styles.sectionTitle}>Why Choose FidduPay?</h2>
          <div className={`${styles.featuresGrid} stagger-children`}>
            <div className={`${styles.feature} animate-slide-up hover-lift`} style={{'--index': 0} as React.CSSProperties}>
              <div className={styles.featureIcon}><i className="fas fa-bolt"></i></div>
              <h3>Instant Processing</h3>
              <p>Real-time payment confirmation and automatic forwarding to your wallets</p>
            </div>
            <div className={`${styles.feature} animate-slide-up hover-lift`} style={{'--index': 1} as React.CSSProperties}>
              <div className={styles.featureIcon}><i className="fas fa-shield-alt"></i></div>
              <h3>Bank-Level Security</h3>
              <p>Advanced encryption, rate limiting, and threat detection protect your transactions</p>
            </div>
            <div className={`${styles.feature} animate-slide-up hover-lift`} style={{'--index': 2} as React.CSSProperties}>
              <div className={styles.featureIcon}><i className="fas fa-network-wired"></i></div>
              <h3>Multi-Network Support</h3>
              <p>Accept USDT on Ethereum, BSC, Polygon, Arbitrum, and Solana networks</p>
            </div>
            <div className={`${styles.feature} animate-slide-up hover-lift`} style={{'--index': 3} as React.CSSProperties}>
              <div className={styles.featureIcon}><i className="fas fa-chart-line"></i></div>
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
            <p>Join thousands of businesses using FidduPay for secure cryptocurrency payments</p>
            <Link to="/login" className={styles.ctaBtn}>
              Get Started
            </Link>
          </div>
        </div>
      </section>
    </div>
  )
}

export default HomePage
