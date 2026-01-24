import React, { useState } from 'react'
import styles from './ContactPage.module.css'

const ContactPage: React.FC = () => {
  const [formData, setFormData] = useState({
    name: '',
    email: '',
    subject: '',
    message: ''
  })

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    // Handle form submission
    console.log('Form submitted:', formData)
  }

  const handleChange = (e: React.ChangeEvent<HTMLInputElement | HTMLTextAreaElement | HTMLSelectElement>) => {
    setFormData({
      ...formData,
      [e.target.name]: e.target.value
    })
  }

  return (
    <div className={styles.contactPage}>
      <div className={styles.container}>
        <div className={styles.header}>
          <h1 className={styles.title}>Get in Touch</h1>
          <p className={styles.subtitle}>
            Have questions? We're here to help you get started with PayFlow
          </p>
        </div>

        <div className={styles.content}>
          <div className={styles.contactInfo}>
            <h2>Contact Information</h2>
            
            <div className={styles.contactItem}>
              <i className="fas fa-envelope"></i>
              <div>
                <h3>Email Support</h3>
                <p>support@payflow.com</p>
                <span>Response within 24 hours</span>
              </div>
            </div>

            <div className={styles.contactItem}>
              <i className="fas fa-phone"></i>
              <div>
                <h3>Phone Support</h3>
                <p>+1 (555) 123-4567</p>
                <span>Mon-Fri, 9AM-6PM EST</span>
              </div>
            </div>

            <div className={styles.contactItem}>
              <i className="fas fa-comments"></i>
              <div>
                <h3>Live Chat</h3>
                <p>Available in dashboard</p>
                <span>Real-time support</span>
              </div>
            </div>

            <div className={styles.contactItem}>
              <i className="fas fa-building"></i>
              <div>
                <h3>Enterprise Sales</h3>
                <p>sales@payflow.com</p>
                <span>Custom solutions</span>
              </div>
            </div>
          </div>

          <div className={styles.contactForm}>
            <h2>Send us a Message</h2>
            <form onSubmit={handleSubmit}>
              <div className={styles.formGroup}>
                <label htmlFor="name">Name</label>
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
                <label htmlFor="email">Email</label>
                <input
                  type="email"
                  id="email"
                  name="email"
                  value={formData.email}
                  onChange={handleChange}
                  required
                />
              </div>

              <div className={styles.formGroup}>
                <label htmlFor="subject">Subject</label>
                <select
                  id="subject"
                  name="subject"
                  value={formData.subject}
                  onChange={handleChange}
                  required
                >
                  <option value="">Select a subject</option>
                  <option value="general">General Inquiry</option>
                  <option value="technical">Technical Support</option>
                  <option value="billing">Billing Question</option>
                  <option value="partnership">Partnership</option>
                  <option value="other">Other</option>
                </select>
              </div>

              <div className={styles.formGroup}>
                <label htmlFor="message">Message</label>
                <textarea
                  id="message"
                  name="message"
                  rows={5}
                  value={formData.message}
                  onChange={handleChange}
                  required
                ></textarea>
              </div>

              <button type="submit" className={styles.submitBtn}>
                <i className="fas fa-paper-plane"></i>
                Send Message
              </button>
            </form>
          </div>
        </div>
      </div>
    </div>
  )
}

export default ContactPage
