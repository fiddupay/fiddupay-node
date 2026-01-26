import React from 'react'
import { Link } from 'react-router-dom'
import { MdSecurity, MdSpeed, MdAccountBalance, MdIntegrationInstructions } from 'react-icons/md'
import styles from './LandingPage.module.css'

const LandingPage: React.FC = () => {
  const features = [
    {
      icon: MdSecurity,
      title: 'Enterprise Security',
      description: 'enterprise-grade security with XSS protection, CSRF tokens, and real-time threat detection'
    },
    {
      icon: MdSpeed,
      title: 'Multi-Blockchain',
      description: 'Accept SOL, USDT on 5 networks: Solana, Ethereum, BSC, Polygon, Arbitrum'
    },
    {
      icon: MdAccountBalance,
      title: 'Automatic Forwarding',
      description: 'Funds automatically forward to your wallets minus fees - no manual processing'
    },
    {
      icon: MdIntegrationInstructions,
      title: 'Easy Integration',
      description: 'Simple REST API with webhooks, SDKs, and comprehensive documentation'
    }
  ]

  return (
    <div className={styles.container}>
      {/* Header */}
      <header className={styles.header}>
        <div className={styles.nav}>
          <div className={styles.logo}>
            <h2>FidduPay</h2>
            <span>by TechyTro</span>
          </div>
          <div className={styles.navLinks}>
            <Link to="/login" className={styles.loginBtn}>Login</Link>
            <Link to="/register" className={styles.signupBtn}>Get Started</Link>
          </div>
        </div>
      </header>

      {/* Hero Section */}
      <section className={styles.hero}>
        <div className={styles.heroContent}>
          <h1 className={styles.heroTitle}>
            Accept Crypto Payments
            <span className={styles.highlight}> Instantly</span>
          </h1>
          <p className={styles.heroSubtitle}>
            Production-ready cryptocurrency payment gateway with enterprise-grade security. 
            Accept payments across 5 blockchains with automatic forwarding.
          </p>
          <div className={styles.heroActions}>
            <Link to="/register" className={styles.primaryBtn}>
              Start Accepting Payments
            </Link>
            <a href="#features" className={styles.secondaryBtn}>
              Learn More
            </a>
          </div>
          <div className={styles.heroStats}>
            <div className={styles.stat}>
              <span className={styles.statNumber}>enterprise-grade</span>
              <span className={styles.statLabel}>Security Score</span>
            </div>
            <div className={styles.stat}>
              <span className={styles.statNumber}>5</span>
              <span className={styles.statLabel}>Blockchains</span>
            </div>
            <div className={styles.stat}>
              <span className={styles.statNumber}>99.9%</span>
              <span className={styles.statLabel}>Uptime</span>
            </div>
          </div>
        </div>
      </section>

      {/* Features Section */}
      <section id="features" className={styles.features}>
        <div className={styles.sectionContent}>
          <h2 className={styles.sectionTitle}>Why Choose FidduPay?</h2>
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
          <h2 className={styles.sectionTitle}>Supported Cryptocurrencies</h2>
          <div className={styles.currencyGrid}>
            <div className={styles.currencyCard}>
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
          </div>
        </div>
      </section>

      {/* Pricing Section */}
      <section className={styles.pricing}>
        <div className={styles.sectionContent}>
          <h2 className={styles.sectionTitle}>Simple, Transparent Pricing</h2>
          <p className={styles.pricingSubtitle}>No setup fees. No monthly fees. Pay only when you earn.</p>
          <div className={styles.pricingGrid}>
            <div className={styles.pricingCard}>
              <h3 className={styles.pricingTitle}>Starter</h3>
              <div className={styles.pricingPrice}>
                <span className={styles.priceNumber}>2.9%</span>
                <span className={styles.priceUnit}>per transaction</span>
              </div>
              <ul className={styles.pricingFeatures}>
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
              <h3 className={styles.pricingTitle}>Business</h3>
              <div className={styles.pricingPrice}>
                <span className={styles.priceNumber}>2.4%</span>
                <span className={styles.priceUnit}>per transaction</span>
              </div>
              <ul className={styles.pricingFeatures}>
                <li>✓ Everything in Starter</li>
                <li>✓ Advanced webhooks</li>
                <li>✓ Priority support</li>
                <li>✓ Custom integration help</li>
                <li>✓ Volume discounts</li>
              </ul>
              <Link to="/register" className={styles.pricingBtn}>Start Free Trial</Link>
            </div>
            <div className={styles.pricingCard}>
              <h3 className={styles.pricingTitle}>Enterprise</h3>
              <div className={styles.pricingPrice}>
                <span className={styles.priceNumber}>Custom</span>
                <span className={styles.priceUnit}>pricing</span>
              </div>
              <ul className={styles.pricingFeatures}>
                <li>✓ Everything in Business</li>
                <li>✓ Dedicated support</li>
                <li>✓ Custom features</li>
                <li>✓ SLA guarantees</li>
                <li>✓ White-label options</li>
              </ul>
              <a href="#contact" className={styles.pricingBtn}>Contact Sales</a>
            </div>
          </div>
        </div>
      </section>

      {/* Testimonials */}
      <section className={styles.testimonials}>
        <div className={styles.sectionContent}>
          <h2 className={styles.sectionTitle}>Trusted by Businesses Worldwide</h2>
          <div className={styles.testimonialsGrid}>
            <div className={styles.testimonialCard}>
              <p className={styles.testimonialText}>
                "FidduPay's security and reliability have been game-changing for our e-commerce platform. 
                The automatic forwarding saves us hours of manual processing."
              </p>
              <div className={styles.testimonialAuthor}>
                <strong>Sarah Chen</strong>
                <span>CTO, TechCommerce</span>
              </div>
            </div>
            <div className={styles.testimonialCard}>
              <p className={styles.testimonialText}>
                "Integration was seamless, and the enterprise-grade security gives our customers confidence. 
                We've processed over $2M in crypto payments without any issues."
              </p>
              <div className={styles.testimonialAuthor}>
                <strong>Marcus Rodriguez</strong>
                <span>Founder, CryptoMarket</span>
              </div>
            </div>
            <div className={styles.testimonialCard}>
              <p className={styles.testimonialText}>
                "The multi-blockchain support and instant notifications have streamlined our payment 
                processing. FidduPay is essential for any serious crypto business."
              </p>
              <div className={styles.testimonialAuthor}>
                <strong>Emily Watson</strong>
                <span>CFO, BlockchainCorp</span>
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* CTA Section */}
      <section id="contact" className={styles.cta}>
        <div className={styles.ctaContent}>
          <h2 className={styles.ctaTitle}>Ready to Start?</h2>
          <p className={styles.ctaSubtitle}>
            Join businesses already using FidduPay to accept cryptocurrency payments
          </p>
          <div className={styles.ctaActions}>
            <Link to="/register" className={styles.ctaBtn}>
              Create Account - Free
            </Link>
            <div className={styles.contactInfo}>
              <p>Need help? Contact our sales team:</p>
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
            <p>Enterprise cryptocurrency payment gateway</p>
          </div>
          <div className={styles.footerLinks}>
            <div className={styles.footerSection}>
              <h4>Product</h4>
              <a href="#features">Features</a>
              <Link to="/pricing">Pricing</Link>
              <a href="/docs">Documentation</a>
              <a href="/api">API Reference</a>
            </div>
            <div className={styles.footerSection}>
              <h4>Company</h4>
              <Link to="/about">About</Link>
              <Link to="/contact">Contact</Link>
              <a href="/support">Support</a>
              <a href="/blog">Blog</a>
            </div>
          </div>
        </div>
        <div className={styles.footerBottom}>
          <p>&copy; 2026 TechyTro Software. All rights reserved.</p>
        </div>
      </footer>
    </div>
  )
}

export default LandingPage
