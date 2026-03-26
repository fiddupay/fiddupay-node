import React from 'react'
import { Link } from 'react-router-dom'
import { MdSecurity, MdSpeed, MdAccountBalance, MdIntegrationInstructions } from 'react-icons/md'
import styles from '@/styles/pages/LandingPage.module.css'

const LandingPage: React.FC = () => {
  const features = [
    {
      icon: MdSecurity,
      title: 'Institutional Security',
      description: 'Bank-grade protection with advanced XSS/CSRF mitigation and institutional-level threat intelligence.'
    },
    {
      icon: MdSpeed,
      title: 'Global Settlement',
      description: 'High-speed settlement across Bitcoin, Solana, Ethereum, BSC, Polygon, and Arbitrum protocols.'
    },
    {
      icon: MdAccountBalance,
      title: 'Automated Liquidity',
      description: 'Streamlined fund routing with automated wallet forwarding and real-time balance reconciliation.'
    },
    {
      icon: MdIntegrationInstructions,
      title: 'Robust Infrastructure',
      description: 'Comprehensive REST architecture, multi-language SDKs, and industry-standard documentation.'
    }
  ]

  return (
    <div className={styles.container}>
      {/* Header */}
      <header className={styles.header}>
        <div className={styles.nav}>
          <div className={styles.logo}>
            <h2>FidduPay</h2>
            <span>Enterprise Protocol</span>
          </div>
          <div className={styles.navLinks}>
            <Link to="/login" className={styles.loginBtn}>Merchant Login</Link>
            <Link to="/register" className={styles.signupBtn}>Create Account</Link>
          </div>
        </div>
      </header>

      {/* Hero Section */}
      <section className={styles.hero}>
        <div className={styles.heroContent}>
          <h1 className={styles.heroTitle}>
            Enterprise-Grade
            <span className={styles.highlight}> Crypto Infrastructure</span>
          </h1>
          <p className={styles.heroSubtitle}>
            The global settlement layer for modern digital commerce. Securely accept and route
            cryptocurrency payments across 6 major blockchains with institutional precision.
          </p>
          <div className={styles.heroActions}>
            <Link to="/register" className={styles.primaryBtn}>
              Deploy Gateway
            </Link>
            <a href="#features" className={styles.secondaryBtn}>
              Explore Infrastructure
            </a>
          </div>
          <div className={styles.heroStats}>
            <div className={styles.stat}>
              <span className={styles.statNumber}>10/10</span>
              <span className={styles.statLabel}>Security Audit Info</span>
            </div>
            <div className={styles.stat}>
              <span className={styles.statNumber}>6</span>
              <span className={styles.statLabel}>Active Protocols</span>
            </div>
            <div className={styles.stat}>
              <span className={styles.statNumber}>99.99%</span>
              <span className={styles.statLabel}>System Uptime</span>
            </div>
          </div>
        </div>
      </section>

      {/* Features Section */}
      <section id="features" className={styles.features}>
        <div className={styles.sectionContent}>
          <h2 className={styles.sectionTitle}>Precision-Engineered Protocol</h2>
          <div className={styles.featuresGrid}>
            {features.map((feature, index) => (
              <div key={index} className={styles.featureCard}>
                <div className={styles.featureIcon}>
                  <feature.icon />
                </div>
                <h3 className={styles.featureTitle}>{feature.title}</h3>
                <p className={styles.featureDescription}>{feature.description}</p>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* Supported Currencies */}
      <section className={styles.currencies}>
        <div className={styles.sectionContent}>
          <h2 className={styles.sectionTitle}>Multi-Protocol Support</h2>
          <div className={styles.currencyGrid}>
            <div className={styles.currencyCard}>
              <span className={styles.currencySymbol}>BTC</span>
              <span className={styles.currencyName}>Bitcoin</span>
            </div>
            <div className={styles.currencyCard}>
              <div className={styles.currencyIconBox} style={{ width: '32px', height: '32px', marginBottom: '8px' }}>
                <img src="/solana-sol-logo.png" alt="SOL" style={{ width: '100%', height: '100%', borderRadius: '50%' }} />
              </div>
              <span className={styles.currencySymbol}>SOL</span>
              <span className={styles.currencyName}>Solana</span>
            </div>
            <div className={styles.currencyCard}>
              <span className={styles.currencySymbol}>ETH</span>
              <span className={styles.currencyName}>Ethereum</span>
            </div>
            <div className={styles.currencyCard}>
              <span className={styles.currencySymbol}>BNB</span>
              <span className={styles.currencyName}>BSC</span>
            </div>
            <div className={styles.currencyCard}>
              <span className={styles.currencySymbol}>MATIC</span>
              <span className={styles.currencyName}>Polygon</span>
            </div>
            <div className={styles.currencyCard}>
              <span className={styles.currencySymbol}>ARB</span>
              <span className={styles.currencyName}>Arbitrum</span>
            </div>
            <div className={styles.currencyCard}>
              <span className={styles.currencySymbol}>USDT</span>
              <span className={styles.currencyName}>Multi-Chain</span>
            </div>
            <div className={styles.currencyCard}>
              <div className={styles.currencyIconBox} style={{ width: '32px', height: '32px', marginBottom: '8px' }}>
                <img src="/binance-usd-busd-logo.png" alt="BUSD" style={{ width: '100%', height: '100%', borderRadius: '50%' }} />
              </div>
              <span className={styles.currencySymbol}>BUSD</span>
              <span className={styles.currencyName}>Binance USD</span>
            </div>
          </div>
        </div>
      </section>

      {/* Pricing Section */}
      <section className={styles.pricing}>
        <div className={styles.sectionContent}>
          <h2 className={styles.sectionTitle}>Scalable Institutional Pricing</h2>
          <p className={styles.pricingSubtitle}>Optimized for high-volume digital asset settlement. No hidden fees.</p>
          <div className={styles.pricingGrid}>
            <div className={styles.pricingCard}>
              <h3 className={styles.pricingTitle}>Growth</h3>
              <div className={styles.pricingPrice}>
                <span className={styles.priceNumber}>2.9%</span>
                <span className={styles.priceUnit}>per transaction</span>
              </div>
              <ul className={styles.pricingFeatures}>
                <li> All active protocols</li>
                <li> Automated liquidity routing</li>
                <li> Webhook event notifications</li>
                <li> API infrastructure access</li>
                <li> 24/7 Priority support</li>
              </ul>
              <Link to="/register" className={styles.pricingBtn}>Scale Now</Link>
            </div>
            <div className={`${styles.pricingCard} ${styles.popular}`}>
              <div className={styles.popularBadge}>Most Popular</div>
              <h3 className={styles.pricingTitle}>Business Prime</h3>
              <div className={styles.pricingPrice}>
                <span className={styles.priceNumber}>2.4%</span>
                <span className={styles.priceUnit}>per transaction</span>
              </div>
              <ul className={styles.pricingFeatures}>
                <li> Everything in Growth</li>
                <li> Advanced reconciliation engine</li>
                <li> Dedicated account management</li>
                <li> Custom integration workflows</li>
                <li> Volume-based incentives</li>
              </ul>
              <Link to="/register" className={styles.pricingBtn}>Start Prime Trial</Link>
            </div>
            <div className={styles.pricingCard}>
              <h3 className={styles.pricingTitle}>Enterprise</h3>
              <div className={styles.pricingPrice}>
                <span className={styles.priceNumber}>Custom</span>
                <span className={styles.priceUnit}>Tier</span>
              </div>
              <ul className={styles.pricingFeatures}>
                <li> Everything in Prime</li>
                <li> Custom protocol development</li>
                <li> Guaranteed uptime SLAs</li>
                <li> White-label infrastructure</li>
                <li> On-premise deployment options</li>
              </ul>
              <a href="mailto:sales@techytro.com" className={styles.pricingBtn}>Contact Institutional Sales</a>
            </div>
          </div>
        </div>
      </section>

      {/* Testimonials */}
      <section className={styles.testimonials}>
        <div className={styles.sectionContent}>
          <h2 className={styles.sectionTitle}>Institutional Partners</h2>
          <div className={styles.testimonialsGrid}>
            <div className={styles.testimonialCard}>
              <p className={styles.testimonialText}>
                "FidduPay's infrastructure has provided our e-commerce platform with the reliability
                required for large-scale digital asset settlement. The security is unmatched."
              </p>
              <div className={styles.testimonialAuthor}>
                <strong>Sarah Chen</strong>
                <span>CTO, GlobalTech Solutions</span>
              </div>
            </div>
            <div className={styles.testimonialCard}>
              <p className={styles.testimonialText}>
                "Seamless integration and institutional-grade security protocols. We've settled over $2M
                in digital assets through FidduPay with complete technical confidence."
              </p>
              <div className={styles.testimonialAuthor}>
                <strong>Marcus Rodriguez</strong>
                <span>Managing Director, FinStream</span>
              </div>
            </div>
            <div className={styles.testimonialCard}>
              <p className={styles.testimonialText}>
                "Advanced multi-protocol support and real-time reconciliation have streamlined our treasury
                operations. FidduPay is the benchmark for crypto infrastructure."
              </p>
              <div className={styles.testimonialAuthor}>
                <strong>Emily Watson</strong>
                <span>Lead Architect, BlockScale</span>
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* CTA Section */}
      <section id="contact" className={styles.cta}>
        <div className={styles.ctaContent}>
          <h2 className={styles.ctaTitle}>Accelerate Your Digital Commerce</h2>
          <p className={styles.ctaSubtitle}>
            Partner with FidduPay to deploy enterprise-grade cryptocurrency infrastructure today.
          </p>
          <div className={styles.ctaActions}>
            <Link to="/register" className={styles.ctaBtn}>
              Initialize Account
            </Link>
            <div className={styles.contactInfo}>
              <p>Speak with our Institutional Sales team:</p>
              <a href="mailto:sales@techytro.com" className={styles.contactLink}>sales@techytro.com</a>
              <a href="tel:+1-555-fiddupay" className={styles.contactLink}>+1 (555) PAY-FLOW</a>
            </div>
          </div>
        </div>
      </section>

      {/* Footer */}
      <footer className={styles.footer}>
        <div className={styles.footerContent}>
          <div className={styles.footerBrand}>
            <h3>FidduPay</h3>
            <p>Institutional Digital Asset Infrastructure</p>
          </div>
          <div className={styles.footerLinks}>
            <div className={styles.footerSection}>
              <h4>Ecosystem</h4>
              <a href="#features">Infrastructure</a>
              <Link to="/pricing">Institutional Tiers</Link>
              <Link to="/docs">Documentation</Link>
              <a href="https://github.com/fiddupay/fiddupay-node/blob/main/API_REFERENCE.md" target="_blank" rel="noopener noreferrer">Technical Reference</a>
            </div>
            <div className={styles.footerSection}>
              <h4>Resources</h4>
              <Link to="/about">Our Vision</Link>
              <Link to="/contact">Support</Link>
              <a href="/legal">Compliance</a>
              <a href="/status">Network Status</a>
            </div>
          </div>
        </div>
        <div className={styles.footerBottom}>
          <p>&copy; 2026 TechyTro Software. Modernizing Financial Rails.</p>
        </div>
      </footer>
    </div>
  )
}

export default LandingPage
