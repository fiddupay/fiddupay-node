import styles from '@/styles/pages/AboutPage.module.css'
import React from 'react'
import { Link } from 'react-router-dom'
import SEO from '@/components/ui/SEO'

const AboutPage: React.FC = () => {
  return (
    <div className={styles.aboutPage}>
      <SEO 
        title="About Us | FidduPay" 
        description="Learn about FidduPay's mission to bridge the gap between traditional business and the decentralized economy through high-performance crypto payment infrastructure."
      />
      {/* Background Ambient Glow */}
      <div className={styles.ambientGlowContainer}>
        <div className={`${styles.blob} ${styles.blobPrimary}`}></div>
        <div className={`${styles.blob} ${styles.blobSecondary}`}></div>
      </div>

      <div className={styles.container}>
        {/* New Hero Section */}
        <section className={`${styles.hero} animate-fade-in-up`}>
          <div className={styles.heroContent}>
            <div className={styles.badge}>
              <i className="fas fa-globe"></i>
              <span>Global Payment Network</span>
            </div>
            <h1 className={styles.heroTitle}>Building the Future of <span className={styles.gradientText}>Digital Commerce</span></h1>
            <p className={styles.heroSubtitle}>
              FidduPay is a high-performance crypto payment gateway designed to bridge the gap between traditional businesses and the decentralized economy. 
            </p>
            <div className={styles.heroActions}>
              <Link to="/register" className={styles.primaryBtn}>Start Accepting Payments</Link>
              <Link to="/contact" className={styles.secondaryBtn}>Schedule a Demo</Link>
            </div>
          </div>
          <div className={styles.heroVisual}>
            <div className={styles.heroImageWrapper}>
              <img src="/about-hero-Photoroom.png" alt="Global Crypto Network" className={styles.heroImage} />
              <div className={styles.imageOverlay}></div>
            </div>
            <div className={styles.floatingCard}>
              <div className={styles.floatingIcon}><i className="fas fa-microchip"></i></div>
              <div className={styles.floatingText}>
                <span>L3 Monitoring</span>
                <strong>Active 24/7</strong>
              </div>
            </div>
          </div>
        </section>

        {/* Content Sections */}
        <div className={styles.contentGrid}>
          <section className={`${styles.infoSection} animate-slide-in-right`}>
            <div className={styles.sectionHeader}>
              <div className={styles.sectionIcon}><i className="fas fa-bullseye"></i></div>
              <h2>Our Mission</h2>
            </div>
            <div className={styles.sectionBody}>
              <p>
                FidduPay was founded with a singular focus: to make cryptocurrency as spendable and accessible as traditional fiat. We believe every business, regardless of its size, deserves access to secure, borderless, and low-fee financial rails.
              </p>
              <p>
                By supporting 6 major blockchain networks and providing institutional-grade security, we ensure that your customers can pay in their preferred assets while you receive the reliability you expect from a premium gateway.
              </p>
            </div>
          </section>

          <section className={`${styles.infoSection} animate-slide-in-right`} style={{ animationDelay: '0.2s' }}>
            <div className={styles.sectionHeader}>
              <div className={styles.sectionIcon}><i className="fas fa-shield-halved"></i></div>
              <h2>Security Foundation</h2>
            </div>
            <div className={styles.sectionBody}>
              <p>
                Security isn’t just a line item in our feature list—it’s the architecture of everything we build. From AES-256 encryption at rest to multi-layer signature validation, we protect every byte of data and every cent of capital.
              </p>
              <p>
                Our platform is SOC 2 compliant and undergoes continuous real-time auditing. We don't just process payments; we secure the future of your revenue.
              </p>
            </div>
          </section>
        </div>

        {/* Stats Grid */}
        <div className={styles.statsGrid}>
          <div className={styles.statCard}>
            <span className={styles.statLabel}>Security Score</span>
            <div className={styles.statValue}>100%</div>
            <p className={styles.statDesc}>Institutional Grade</p>
          </div>
          <div className={styles.statCard}>
            <span className={styles.statLabel}>Networks</span>
            <div className={styles.statValue}>6+</div>
            <p className={styles.statDesc}>Global Blockchains</p>
          </div>
          <div className={styles.statCard}>
            <span className={styles.statLabel}>Platform Uptime</span>
            <div className={styles.statValue}>99.9%</div>
            <p className={styles.statDesc}>Reliability Guaranteed</p>
          </div>
          <div className={styles.statCard}>
            <span className={styles.statLabel}>Integration Time</span>
            <div className={styles.statValue}>&lt;5m</div>
            <p className={styles.statDesc}>Developer Focused</p>
          </div>
        </div>

        {/* Team / Brand Section */}
        <section className={styles.teamSection}>
          <div className={styles.teamContainer}>
            <div className={styles.teamLogo}>
              <img src="/logo/logo-brandmark.svg" alt="FidduPay" style={{ height: '56px' }} />
            </div>
            <div className={styles.teamText}>
              <h2>Built by TechyTro Software</h2>
              <p>
                FidduPay is proudly developed and maintained by TechyTro Software—a team of veterans in financial technology and decentralized architecture. We are dedicated to creating high-trust, high-performance infrastructure for the next billion users.
              </p>
            </div>
          </div>
        </section>

        {/* CTA */}
        <div className={styles.ctaBanner}>
          <div className={styles.ctaGlow}></div>
          <h2 className={styles.ctaTitle}>Ready to join the revolution?</h2>
          <p className={styles.ctaSubtitle}>Scale your business globally with our borderless payment gateway.</p>
          <div className={styles.ctaActions}>
            <Link to="/register" className={styles.ctaPrimary}>Create Merchant Account</Link>
            <Link to="/contact" className={styles.ctaSecondary}>Talk to an Expert</Link>
          </div>
        </div>
      </div>
    </div>
  )
}

export default AboutPage
