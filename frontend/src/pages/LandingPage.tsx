import React from 'react'
import { Link } from 'react-router-dom'
import { MdSecurity, MdSpeed, MdAccountBalance, MdIntegrationInstructions } from 'react-icons/md'
import styles from './LandingPage.module.css'

const LandingPage: React.FC = () => {
  const features = [
    {
      icon: MdSecurity,
      title: 'Enterprise Security',
      description: '10/10 security score with XSS protection, CSRF tokens, and real-time threat detection'
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
            <h2>PayFlow</h2>
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
            Production-ready cryptocurrency payment gateway with 10/10 security score. 
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
              <span className={styles.statNumber}>10/10</span>
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
          <h2 className={styles.sectionTitle}>Why Choose PayFlow?</h2>
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
              <span className={styles.currencySymbol}>USDT</span>
              <span className={styles.currencyName}>Ethereum</span>
            </div>
            <div className={styles.currencyCard}>
              <span className={styles.currencySymbol}>USDT</span>
              <span className={styles.currencyName}>BSC</span>
            </div>
            <div className={styles.currencyCard}>
              <span className={styles.currencySymbol}>USDT</span>
              <span className={styles.currencyName}>Polygon</span>
            </div>
            <div className={styles.currencyCard}>
              <span className={styles.currencySymbol}>USDT</span>
              <span className={styles.currencyName}>Arbitrum</span>
            </div>
          </div>
        </div>
      </section>

      {/* CTA Section */}
      <section className={styles.cta}>
        <div className={styles.ctaContent}>
          <h2 className={styles.ctaTitle}>Ready to Start?</h2>
          <p className={styles.ctaSubtitle}>
            Join businesses already using PayFlow to accept cryptocurrency payments
          </p>
          <Link to="/register" className={styles.ctaBtn}>
            Create Account - Free
          </Link>
        </div>
      </section>

      {/* Footer */}
      <footer className={styles.footer}>
        <div className={styles.footerContent}>
          <div className={styles.footerBrand}>
            <h3>PayFlow</h3>
            <p>Enterprise cryptocurrency payment gateway</p>
          </div>
          <div className={styles.footerLinks}>
            <div className={styles.footerSection}>
              <h4>Product</h4>
              <a href="#features">Features</a>
              <a href="/docs">Documentation</a>
              <a href="/api">API Reference</a>
            </div>
            <div className={styles.footerSection}>
              <h4>Company</h4>
              <a href="/about">About</a>
              <a href="/contact">Contact</a>
              <a href="/support">Support</a>
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
