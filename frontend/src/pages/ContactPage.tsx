import React, { useState } from 'react'
import { Link } from 'react-router-dom'
import { MdEmail, MdPhone, MdLocationOn, MdSend } from 'react-icons/md'
import styles from './ContactPage.module.css'

const ContactPage: React.FC = () => {
  const [formData, setFormData] = useState({
    name: '',
    email: '',
    company: '',
    subject: '',
    message: ''
  })

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    // Handle form submission
    console.log('Form submitted:', formData)
    alert('Thank you for your message! We\'ll get back to you within 24 hours.')
    setFormData({ name: '', email: '', company: '', subject: '', message: '' })
  }

  const handleChange = (e: React.ChangeEvent<HTMLInputElement | HTMLTextAreaElement | HTMLSelectElement>) => {
    setFormData({
      ...formData,
      [e.target.name]: e.target.value
    })
  }

  return (
    <div className={styles.container}>
      {/* Header */}
      <header className={styles.header}>
        <div className={styles.nav}>
          <Link to="/" className={styles.logo}>
            <h2>PayFlow</h2>
            <span>by TechyTro</span>
          </Link>
          <div className={styles.navLinks}>
            <Link to="/">Home</Link>
            <Link to="/about">About</Link>
            <Link to="/login" className={styles.loginBtn}>Login</Link>
          </div>
        </div>
      </header>

      {/* Hero */}
      <section className={styles.hero}>
        <div className={styles.heroContent}>
          <h1>Contact Us</h1>
          <p>Get in touch with our team for support, sales, or partnership inquiries</p>
        </div>
      </section>

      {/* Contact Info & Form */}
      <section className={styles.contact}>
        <div className={styles.contactContainer}>
          {/* Contact Info */}
          <div className={styles.contactInfo}>
            <h2>Get in Touch</h2>
            <p>We're here to help you succeed with cryptocurrency payments. Reach out to us through any of these channels:</p>
            
            <div className={styles.contactMethods}>
              <div className={styles.contactMethod}>
                <MdEmail className={styles.contactIcon} />
                <div>
                  <h3>Email</h3>
                  <p>sales@techytro.com</p>
                  <p>support@techytro.com</p>
                </div>
              </div>
              
              <div className={styles.contactMethod}>
                <MdPhone className={styles.contactIcon} />
                <div>
                  <h3>Phone</h3>
                  <p>+1 (555) PAY-FLOW</p>
                  <p>Mon-Fri 9AM-6PM EST</p>
                </div>
              </div>
              
              <div className={styles.contactMethod}>
                <MdLocationOn className={styles.contactIcon} />
                <div>
                  <h3>Office</h3>
                  <p>123 Blockchain Avenue</p>
                  <p>San Francisco, CA 94105</p>
                </div>
              </div>
            </div>

            <div className={styles.responseTime}>
              <h3>Response Times</h3>
              <ul>
                <li><strong>Sales Inquiries:</strong> Within 2 hours</li>
                <li><strong>Technical Support:</strong> Within 4 hours</li>
                <li><strong>General Questions:</strong> Within 24 hours</li>
              </ul>
            </div>
          </div>

          {/* Contact Form */}
          <div className={styles.contactForm}>
            <h2>Send us a Message</h2>
            <form onSubmit={handleSubmit}>
              <div className={styles.formRow}>
                <div className={styles.formGroup}>
                  <label htmlFor="name">Full Name *</label>
                  <input
                    type="text"
                    id="name"
                    name="name"
                    value={formData.name}
                    onChange={handleChange}
                    required
                  />
                </div>
                <div className={styles.formGroup}>
                  <label htmlFor="email">Email Address *</label>
                  <input
                    type="email"
                    id="email"
                    name="email"
                    value={formData.email}
                    onChange={handleChange}
                    required
                  />
                </div>
              </div>

              <div className={styles.formRow}>
                <div className={styles.formGroup}>
                  <label htmlFor="company">Company</label>
                  <input
                    type="text"
                    id="company"
                    name="company"
                    value={formData.company}
                    onChange={handleChange}
                  />
                </div>
                <div className={styles.formGroup}>
                  <label htmlFor="subject">Subject *</label>
                  <select
                    id="subject"
                    name="subject"
                    value={formData.subject}
                    onChange={handleChange}
                    required
                  >
                    <option value="">Select a subject</option>
                    <option value="sales">Sales Inquiry</option>
                    <option value="support">Technical Support</option>
                    <option value="partnership">Partnership</option>
                    <option value="billing">Billing Question</option>
                    <option value="other">Other</option>
                  </select>
                </div>
              </div>

              <div className={styles.formGroup}>
                <label htmlFor="message">Message *</label>
                <textarea
                  id="message"
                  name="message"
                  rows={6}
                  value={formData.message}
                  onChange={handleChange}
                  placeholder="Tell us about your project, questions, or how we can help..."
                  required
                />
              </div>

              <button type="submit" className={styles.submitBtn}>
                <MdSend />
                Send Message
              </button>
            </form>
          </div>
        </div>
      </section>

      {/* FAQ */}
      <section className={styles.faq}>
        <div className={styles.sectionContent}>
          <h2>Frequently Asked Questions</h2>
          <div className={styles.faqGrid}>
            <div className={styles.faqItem}>
              <h3>How quickly can I start accepting payments?</h3>
              <p>You can start accepting cryptocurrency payments within minutes of signing up. Our API is designed for quick integration.</p>
            </div>
            <div className={styles.faqItem}>
              <h3>What cryptocurrencies do you support?</h3>
              <p>We support SOL and USDT across 5 major blockchains: Solana, Ethereum, BSC, Polygon, and Arbitrum.</p>
            </div>
            <div className={styles.faqItem}>
              <h3>Is there a setup fee?</h3>
              <p>No setup fees, no monthly fees. You only pay a small percentage per successful transaction.</p>
            </div>
            <div className={styles.faqItem}>
              <h3>How secure is PayFlow?</h3>
              <p>PayFlow has achieved a perfect 10/10 security score with enterprise-grade protection including XSS prevention, CSRF protection, and real-time threat detection.</p>
            </div>
          </div>
        </div>
      </section>

      {/* Footer */}
      <footer className={styles.footer}>
        <div className={styles.footerContent}>
          <p>&copy; 2026 TechyTro Software. All rights reserved.</p>
          <div className={styles.footerLinks}>
            <Link to="/">Home</Link>
            <Link to="/about">About</Link>
            <a href="/privacy">Privacy</a>
            <a href="/terms">Terms</a>
          </div>
        </div>
      </footer>
    </div>
  )
}

export default ContactPage
