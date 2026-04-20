import React, { useState } from 'react'
import { useToast } from '@/contexts/ToastContext'
import { useLoading } from '@/contexts/LoadingContext'
import { publicAPI } from '@/services/apiService'
import CustomSelect from '@/components/ui/CustomSelect'
import styles from '@/styles/pages/ContactPage.module.css'

const ContactPage: React.FC = () => {
  const [formData, setFormData] = useState({
    name: '',
    email: '',
    subject: '',
    message: ''
  })

  const { showToast } = useToast()
  const { setLoading } = useLoading()

  const subjectOptions = [
    { value: 'general', label: 'General Inquiry' },
    { value: 'technical', label: 'Technical Support' },
    { value: 'billing', label: 'Billing Question' },
    { value: 'partnership', label: 'Partnership' },
    { value: 'other', label: 'Other' },
  ]

  const sanitizeInput = (input: string): string => {
    return input
      .replace(/[<>]/g, '')
      .replace(/javascript:/gi, '')
      .replace(/on\w+=/gi, '')
      .trim()
  }

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()

    if (!formData.name || !formData.email || !formData.subject || !formData.message) {
      showToast('Please fill in all required fields', 'error')
      return
    }

    const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/
    if (!emailRegex.test(formData.email)) {
      showToast('Please enter a valid email address', 'error')
      return
    }

    setLoading(true)
    try {
      await publicAPI.contact({
        name: sanitizeInput(formData.name),
        email: sanitizeInput(formData.email),
        subject: sanitizeInput(formData.subject),
        message: sanitizeInput(formData.message)
      })

      showToast('Message sent successfully! We\'ll get back to you soon.', 'success')
      setFormData({ name: '', email: '', subject: '', message: '' })
    } catch (error: any) {
      const message = error.response?.data?.error?.message || error.response?.data?.error || error.message || 'Failed to send message. Please try again.'
      showToast(message, 'error')
    } finally {
      setLoading(false)
    }
  }

  const handleChange = (e: React.ChangeEvent<HTMLInputElement | HTMLTextAreaElement>) => {
    const { name, value } = e.target
    setFormData(prev => ({
      ...prev,
      [name]: sanitizeInput(value)
    }))
  }

  const handleSubjectChange = (value: string) => {
    setFormData(prev => ({
      ...prev,
      subject: value
    }))
  }

  return (
    <div className={styles.contactPage}>
      {/* Background Ambient Glow */}
      <div className={styles.ambientGlowContainer}>
        <div className={`${styles.blob} ${styles.blobPrimary}`}></div>
        <div className={`${styles.blob} ${styles.blobSecondary}`}></div>
      </div>

      <div className={styles.container}>
        <div className={`${styles.header} animate-fade-in-up`}>
          <h1 className={styles.title}>Get in <span className={styles.gradientText}>Touch</span></h1>
          <p className={styles.subtitle}>
            Have questions? We're here to help you get started with FidduPay
          </p>
        </div>

        <div className={styles.content}>
          <div className={`${styles.contactInfo} animate-slide-in-right`}>
            <h2>Contact Information</h2>
            <div className={styles.infoGrid}>
              <div className={styles.contactItem}>
                <div className={styles.iconBox}><i className="fas fa-envelope"></i></div>
                <div className={styles.itemText}>
                  <h3>Email Support</h3>
                  <p>support@fiddupay.com</p>
                  <span>Response within 24 hours</span>
                </div>
              </div>

              <div className={styles.contactItem}>
                <div className={styles.iconBox}><i className="fas fa-phone"></i></div>
                <div className={styles.itemText}>
                  <h3>Phone Support</h3>
                  <p>+234 (806) 802-2509</p>
                  <span>Mon-Fri, 9AM-6PM EST</span>
                </div>
              </div>

              <div className={styles.contactItem}>
                <div className={styles.iconBox}><i className="fas fa-comments"></i></div>
                <div className={styles.itemText}>
                  <h3>Live Chat</h3>
                  <p>Coming soon in dashboard</p>
                  <span>Real-time support</span>
                </div>
              </div>

              <div className={styles.contactItem}>
                <div className={styles.iconBox}><i className="fas fa-building"></i></div>
                <div className={styles.itemText}>
                  <h3>Enterprise Sales</h3>
                  <p>sales@fiddupay.com</p>
                  <span>Custom solutions</span>
                </div>
              </div>
            </div>
          </div>

          <div className={`${styles.contactFormContainer} animate-slide-in-right`} style={{ animationDelay: '0.2s' }}>
            <h2>Send us a Message</h2>
            <form onSubmit={handleSubmit} className={styles.form}>
              <div className={styles.formGroup}>
                <label htmlFor="name">Name</label>
                <input
                  type="text"
                  id="name"
                  name="name"
                  value={formData.name}
                  onChange={handleChange}
                  required
                  className={styles.input}
                  placeholder="John Doe"
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
                  className={styles.input}
                  placeholder="john@example.com"
                />
              </div>

              <div className={styles.formGroup}>
                <CustomSelect
                  label="Subject"
                  options={subjectOptions}
                  value={formData.subject}
                  onChange={handleSubjectChange}
                  placeholder="Select a subject"
                  required
                />
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
                  className={styles.textarea}
                  placeholder="How can we help you today?"
                ></textarea>
              </div>

              <button type="submit" className={styles.submitBtn}>
                <span>Send Message</span>
                <i className="fas fa-paper-plane"></i>
              </button>
            </form>
          </div>
        </div>
      </div>
    </div>
  )
}

export default ContactPage
