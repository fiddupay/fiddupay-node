import React from 'react'
import { Link } from 'react-router-dom'
import styles from './AboutPage.module.css'

const AboutPage: React.FC = () => {
  return (
    <div className={styles.aboutPage}>
      <div className={styles.container}>
        <div className={styles.hero}>
          <h1>About FidduPay</h1>
          <p className={styles.heroSubtitle}>
            We're building the future of cryptocurrency payments, making it simple 
            for businesses to accept digital currencies securely and efficiently.
          </p>
        </div>

        <div className={styles.content}>
          <div className={styles.section}>
            <div className={styles.textContent}>
              <h2>Our Mission</h2>
              <p>
                FidduPay was founded with a simple mission: to bridge the gap between 
                traditional business operations and the revolutionary world of cryptocurrency. 
                We believe that every business, regardless of size, should have access to 
                secure, fast, and reliable crypto payment processing.
              </p>
              <p>
                Our platform supports payments across 5 major blockchain networks, 
                ensuring your customers can pay with their preferred cryptocurrency 
                while you receive the security and reliability you need.
              </p>
            </div>
            <div className={styles.imageContent}>
              <img 
                src="https://images.unsplash.com/photo-1551434678-e076c223a692?w=500&h=400&fit=crop" 
                alt="Team collaboration" 
              />
            </div>
          </div>

          <div className={styles.section}>
            <div className={styles.imageContent}>
              <img 
                src="https://images.unsplash.com/photo-1563013544-824ae1b704d3?w=500&h=400&fit=crop" 
                alt="Security and technology" 
              />
            </div>
            <div className={styles.textContent}>
              <h2>Security First</h2>
              <p>
                Security isn't just a feature for us—it's our foundation. With a 10/10 
                security score, we implement bank-level encryption, advanced threat 
                detection, and comprehensive monitoring to protect every transaction.
              </p>
              <p>
                Our platform is SOC 2 compliant and undergoes regular security audits 
                to ensure we meet the highest standards of data protection and 
                financial security.
              </p>
            </div>
          </div>

          <div className={styles.stats}>
            <div className={styles.statCard}>
              <div className={styles.statNumber}>10/10</div>
              <div className={styles.statLabel}>Security Score</div>
            </div>
            <div className={styles.statCard}>
              <div className={styles.statNumber}>5</div>
              <div className={styles.statLabel}>Blockchain Networks</div>
            </div>
            <div className={styles.statCard}>
              <div className={styles.statNumber}>99.9%</div>
              <div className={styles.statLabel}>Uptime</div>
            </div>
            <div className={styles.statCard}>
              <div className={styles.statNumber}>24/7</div>
              <div className={styles.statLabel}>Support</div>
            </div>
          </div>

          <div className={styles.team}>
            <h2>Built by TechyTro Software</h2>
            <p>
              FidduPay is proudly developed by TechyTro Software, a team of experienced 
              developers and blockchain specialists dedicated to creating innovative 
              financial technology solutions.
            </p>
            <p>
              Our team combines years of experience in traditional finance, blockchain 
              technology, and software development to deliver a platform that businesses 
              can trust and rely on.
            </p>
          </div>

          <div className={styles.cta}>
            <h2>Ready to Get Started?</h2>
            <p>Join thousands of businesses already using FidduPay for their crypto payments</p>
            <div className={styles.ctaButtons}>
              <Link to="/register" className={styles.primaryBtn}>
                Start Accepting Payments
              </Link>
              <Link to="/contact" className={styles.secondaryBtn}>
                Contact Sales
              </Link>
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}

export default AboutPage
