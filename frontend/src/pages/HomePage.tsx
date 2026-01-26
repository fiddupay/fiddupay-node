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
              <div className={styles.cryptoIcon}>
                <img src="https://raw.githubusercontent.com/spothq/cryptocurrency-icons/master/128/color/sol.png" alt="Solana" className={styles.cryptoImg} />
              </div>
              <h3>Solana</h3>
              <p>Fast & Low Cost</p>
            </div>
            <div className={styles.cryptoItem}>
              <div className={styles.cryptoIcon}>
                <img src="https://raw.githubusercontent.com/spothq/cryptocurrency-icons/master/128/color/eth.png" alt="Ethereum" className={styles.cryptoImg} />
              </div>
              <h3>Ethereum</h3>
              <p>Smart Contracts</p>
            </div>
            <div className={styles.cryptoItem}>
              <div className={styles.cryptoIcon}>
                <img src="https://raw.githubusercontent.com/spothq/cryptocurrency-icons/master/128/color/bnb.png" alt="BNB" className={styles.cryptoImg} />
              </div>
              <h3>BSC</h3>
              <p>Binance Chain</p>
            </div>
            <div className={styles.cryptoItem}>
              <div className={styles.cryptoIcon}>
                <img src="https://raw.githubusercontent.com/spothq/cryptocurrency-icons/master/128/color/matic.png" alt="MATIC" className={styles.cryptoImg} />
              </div>
              <h3>Polygon</h3>
              <p>Layer 2 Scaling</p>
            </div>
            <div className={styles.cryptoItem}>
              <div className={styles.cryptoIcon}>
                <img src="https://cryptologos.cc/logos/arbitrum-arb-logo.png" alt="ARB" className={styles.cryptoImg} />
              </div>
              <h3>Arbitrum</h3>
              <p>L2 Solution</p>
            </div>
            <div className={styles.cryptoItem}>
              <div className={styles.cryptoIcon}>
                <img src="https://raw.githubusercontent.com/spothq/cryptocurrency-icons/master/128/color/usdt.png" alt="USDT" className={styles.cryptoImg} />
              </div>
              <h3>Tether</h3>
              <p>5 Networks</p>
            </div>
          </div>
        </div>
      </section>

      {/* Features Section */}
      <section className={styles.features}>
        <div className={styles.container}>
          <div className={styles.featuresHeader}>
            <h2 className={styles.sectionTitle}>Why Choose FidduPay?</h2>
            <p className={styles.sectionSubtitle}>Everything you need to accept cryptocurrency payments with confidence</p>
          </div>
          
          <div className={styles.featuresGrid}>
            <div className={styles.featureCard}>
              <div className={styles.featureIconWrapper}>
                <div className={styles.featureIconBg}>
                  <i className="fas fa-bolt"></i>
                </div>
              </div>
              <div className={styles.featureContent}>
                <h3>Instant Processing</h3>
                <p>Real-time payment confirmation and automatic forwarding to your wallets within seconds</p>
                <ul className={styles.featureList}>
                  <li>Real-time confirmations</li>
                  <li>Automatic forwarding</li>
                  <li>Zero delays</li>
                </ul>
              </div>
            </div>

            <div className={styles.featureCard}>
              <div className={styles.featureIconWrapper}>
                <div className={styles.featureIconBg}>
                  <i className="fas fa-shield-alt"></i>
                </div>
              </div>
              <div className={styles.featureContent}>
                <h3>Enterprise Security</h3>
                <p>Advanced encryption, rate limiting, and threat detection protect your transactions</p>
                <ul className={styles.featureList}>
                  <li>Advanced encryption</li>
                  <li>Rate limiting</li>
                  <li>Threat detection</li>
                </ul>
              </div>
            </div>

            <div className={styles.featureCard}>
              <div className={styles.featureIconWrapper}>
                <div className={styles.featureIconBg}>
                  <i className="fas fa-network-wired"></i>
                </div>
              </div>
              <div className={styles.featureContent}>
                <h3>Multi-Network Support</h3>
                <p>Accept payments across 5 major blockchain networks with unified management</p>
                <ul className={styles.featureList}>
                  <li>5 blockchain networks</li>
                  <li>10 cryptocurrencies</li>
                  <li>Unified dashboard</li>
                </ul>
              </div>
            </div>

            <div className={styles.featureCard}>
              <div className={styles.featureIconWrapper}>
                <div className={styles.featureIconBg}>
                  <i className="fas fa-chart-line"></i>
                </div>
              </div>
              <div className={styles.featureContent}>
                <h3>Real-Time Analytics</h3>
                <p>Monitor payments, track performance, and manage your crypto business with detailed insights</p>
                <ul className={styles.featureList}>
                  <li>Payment tracking</li>
                  <li>Performance metrics</li>
                  <li>Business insights</li>
                </ul>
              </div>
            </div>

            <div className={styles.featureCard}>
              <div className={styles.featureIconWrapper}>
                <div className={styles.featureIconBg}>
                  <i className="fas fa-code"></i>
                </div>
              </div>
              <div className={styles.featureContent}>
                <h3>Developer Friendly</h3>
                <p>Simple API integration with comprehensive documentation and SDKs for popular languages</p>
                <ul className={styles.featureList}>
                  <li>RESTful API</li>
                  <li>Multiple SDKs</li>
                  <li>Complete docs</li>
                </ul>
              </div>
            </div>

            <div className={styles.featureCard}>
              <div className={styles.featureIconWrapper}>
                <div className={styles.featureIconBg}>
                  <i className="fas fa-headset"></i>
                </div>
              </div>
              <div className={styles.featureContent}>
                <h3>24/7 Support</h3>
                <p>Get help when you need it with our dedicated support team and comprehensive resources</p>
                <ul className={styles.featureList}>
                  <li>24/7 availability</li>
                  <li>Expert support</li>
                  <li>Quick response</li>
                </ul>
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* FAQ Section */}
      <section className={styles.faq}>
        <div className={styles.container}>
          <div className={styles.faqContent}>
            <div className={styles.faqLeft}>
              <div className={styles.faqIllustration}>
                <div className={styles.faqIconWrapper}>
                  <div className={styles.faqIconBg}>
                    <i className="fas fa-question-circle"></i>
                  </div>
                  <div className={styles.faqDecorations}>
                    <div className={styles.faqCoin}>₿</div>
                    <div className={styles.faqStar}>✨</div>
                    <div className={styles.faqStar}>⭐</div>
                  </div>
                </div>
              </div>
              <h2>Frequently Asked Questions</h2>
              <p>Find answers to common questions about crypto payments and our platform.</p>
            </div>
            <div className={styles.faqRight}>
              <div className={styles.faqItem}>
                <div className={styles.faqQuestion}>
                  <h3>How do I start accepting crypto payments?</h3>
                  <span className={styles.faqToggle}>+</span>
                </div>
                <div className={styles.faqAnswer}>
                  <p>Simply sign up, complete verification, and integrate our API or use our hosted checkout. You'll be accepting payments in minutes.</p>
                </div>
              </div>
              <div className={styles.faqItem}>
                <div className={styles.faqQuestion}>
                  <h3>What cryptocurrencies do you support?</h3>
                  <span className={styles.faqToggle}>+</span>
                </div>
                <div className={styles.faqAnswer}>
                  <p>We support SOL, ETH, BNB, MATIC, ARB, and USDT across Ethereum, BSC, Polygon, Arbitrum, and Solana networks.</p>
                </div>
              </div>
              <div className={styles.faqItem}>
                <div className={styles.faqQuestion}>
                  <h3>How fast are payments processed?</h3>
                  <span className={styles.faqToggle}>+</span>
                </div>
                <div className={styles.faqAnswer}>
                  <p>Payments are confirmed in real-time and automatically forwarded to your wallets within seconds of blockchain confirmation.</p>
                </div>
              </div>
              <div className={styles.faqItem}>
                <div className={styles.faqQuestion}>
                  <h3>Is my money and data safe?</h3>
                  <span className={styles.faqToggle}>+</span>
                </div>
                <div className={styles.faqAnswer}>
                  <p>Yes, we use enterprise-grade security with advanced encryption, rate limiting, and real-time threat detection.</p>
                </div>
              </div>
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
