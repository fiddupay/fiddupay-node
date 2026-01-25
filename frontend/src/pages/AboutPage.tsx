import React from 'react'
import { Link } from 'react-router-dom'
import { MdSecurity, MdSpeed, MdPeople, MdTrendingUp } from 'react-icons/md'
import styles from './AboutPage.module.css'

const AboutPage: React.FC = () => {
  const stats = [
    { number: '10/10', label: 'Security Score' },
    { number: '99.9%', label: 'Uptime' },
    { number: '5', label: 'Blockchains' },
    { number: '1000+', label: 'Businesses' }
  ]

  const team = [
    {
      name: 'Alex Chen',
      role: 'CEO & Founder',
      bio: 'Former blockchain architect at major fintech companies. 10+ years in cryptocurrency and payment systems.'
    },
    {
      name: 'Sarah Rodriguez',
      role: 'CTO',
      bio: 'Security expert with extensive experience in enterprise payment processing and blockchain technology.'
    },
    {
      name: 'Marcus Johnson',
      role: 'Head of Product',
      bio: 'Product strategist focused on creating seamless payment experiences for businesses of all sizes.'
    }
  ]

  return (
    <div className={styles.container}>
      {/* Header */}
      <header className={styles.header}>
        <div className={styles.nav}>
          <Link to="/" className={styles.logo}>
            <h2>FidduPay</h2>
            <span>by TechyTro</span>
          </Link>
          <div className={styles.navLinks}>
            <Link to="/">Home</Link>
            <Link to="/contact">Contact</Link>
            <Link to="/login" className={styles.loginBtn}>Login</Link>
          </div>
        </div>
      </header>

      {/* Hero */}
      <section className={styles.hero}>
        <div className={styles.heroContent}>
          <h1>About FidduPay</h1>
          <p>We're building the future of cryptocurrency payments for businesses worldwide</p>
        </div>
      </section>

      {/* Mission */}
      <section className={styles.mission}>
        <div className={styles.sectionContent}>
          <h2>Our Mission</h2>
          <p className={styles.missionText}>
            At TechyTro Software, we believe cryptocurrency payments should be as simple and secure as traditional payments. 
            FidduPay was born from the need to bridge the gap between complex blockchain technology and everyday business operations.
          </p>
          <p className={styles.missionText}>
            We're committed to providing enterprise-grade security, seamless integration, and reliable service that businesses 
            can trust with their most critical payment processing needs.
          </p>
        </div>
      </section>

      {/* Stats */}
      <section className={styles.stats}>
        <div className={styles.sectionContent}>
          <div className={styles.statsGrid}>
            {stats.map((stat, index) => (
              <div key={index} className={styles.statCard}>
                <span className={styles.statNumber}>{stat.number}</span>
                <span className={styles.statLabel}>{stat.label}</span>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* Values */}
      <section className={styles.values}>
        <div className={styles.sectionContent}>
          <h2>Our Values</h2>
          <div className={styles.valuesGrid}>
            <div className={styles.valueCard}>
              <MdSecurity className={styles.valueIcon} />
              <h3>Security First</h3>
              <p>Every feature is built with security as the foundation, not an afterthought.</p>
            </div>
            <div className={styles.valueCard}>
              <MdSpeed className={styles.valueIcon} />
              <h3>Performance</h3>
              <p>Lightning-fast processing with 99.9% uptime for mission-critical operations.</p>
            </div>
            <div className={styles.valueCard}>
              <MdPeople className={styles.valueIcon} />
              <h3>Customer Success</h3>
              <p>Your success is our success. We provide the tools and support you need to thrive.</p>
            </div>
            <div className={styles.valueCard}>
              <MdTrendingUp className={styles.valueIcon} />
              <h3>Innovation</h3>
              <p>Continuously evolving to support new blockchains and payment technologies.</p>
            </div>
          </div>
        </div>
      </section>

      {/* Team */}
      <section className={styles.team}>
        <div className={styles.sectionContent}>
          <h2>Leadership Team</h2>
          <div className={styles.teamGrid}>
            {team.map((member, index) => (
              <div key={index} className={styles.teamCard}>
                <div className={styles.teamAvatar}>
                  {member.name.split(' ').map(n => n[0]).join('')}
                </div>
                <h3>{member.name}</h3>
                <h4>{member.role}</h4>
                <p>{member.bio}</p>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* CTA */}
      <section className={styles.cta}>
        <div className={styles.ctaContent}>
          <h2>Ready to Get Started?</h2>
          <p>Join the businesses already using FidduPay for their cryptocurrency payments</p>
          <Link to="/register" className={styles.ctaBtn}>Start Your Free Account</Link>
        </div>
      </section>

      {/* Footer */}
      <footer className={styles.footer}>
        <div className={styles.footerContent}>
          <p>&copy; 2026 TechyTro Software. All rights reserved.</p>
          <div className={styles.footerLinks}>
            <Link to="/">Home</Link>
            <Link to="/contact">Contact</Link>
            <a href="/privacy">Privacy</a>
            <a href="/terms">Terms</a>
          </div>
        </div>
      </footer>
    </div>
  )
}

export default AboutPage
