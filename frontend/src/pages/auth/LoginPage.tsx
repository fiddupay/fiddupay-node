import { useLoading } from '@/contexts/LoadingContext'
import { useToast } from '@/contexts/ToastContext'
import { useAuthStore } from '@/stores/authStore'
import styles from '@/styles/pages/auth/LoginPage.module.css'
import React, { useState } from 'react'
import { Link, Navigate } from 'react-router-dom'

const LoginPage: React.FC = () => {
  const { login, isAuthenticated } = useAuthStore()
  const { showToast } = useToast()
  const { setLoading } = useLoading()
  const [showPassword, setShowPassword] = useState(false)
  const [rememberMe, setRememberMe] = useState(false)
  const [formData, setFormData] = useState({
    email: '',
    password: '',
    two_factor_code: ''
  })

  if (isAuthenticated) {
    return <Navigate to="/app/dashboard" replace />
  }

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()

    if (!formData.email.trim()) {
      showToast('Email is required', 'error')
      return
    }

    const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/
    if (!emailRegex.test(formData.email)) {
      showToast('Please enter a valid email address', 'error')
      return
    }

    if (!formData.password) {
      showToast('Password is required', 'error')
      return
    }

    setLoading(true)
    try {
      await login({
        email: formData.email.trim().toLowerCase(),
        password: formData.password,
        two_factor_code: formData.two_factor_code.trim() || undefined,
        remember_me: rememberMe
      })
      showToast('Login successful!', 'success')
    } catch (error: any) {
      const errorMessage = error.response?.data?.error?.message || error.response?.data?.message || error.response?.data?.error || error.message || 'Login failed. Please check your credentials.'
      showToast(errorMessage, 'error')
    } finally {
      setLoading(false)
    }
  }

  const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    setFormData({
      ...formData,
      [e.target.name]: e.target.value
    })
  }

  return (
    <div className={styles.loginPage}>
      {/* Background Ambient Glow */}
      <div className={styles.ambientGlowContainer}>
        <div className={`${styles.blob} ${styles.blobPrimary}`}></div>
        <div className={`${styles.blob} ${styles.blobSecondary}`}></div>
      </div>

      <div className={styles.splitScreen}>
        {/* Left Aspect: Illustration & Branding */}
        <div className={`${styles.visualSide} animate-fade-in`}>
          <div className={styles.heroContent}>
            <h1 className={styles.heroTitle}>The Gateway to <span className={styles.gradientText}>Financial Freedom</span></h1>
            <p className={styles.heroSubtitle}>
              Experience secure, lighting-fast crypto payments with enterprise-grade protection and 24/7 monitoring.
            </p>
            
            <div className={styles.visualGrid}>
              <div className={styles.visualCard}>
                <div className={styles.cardIcon}>
                  <i className="fas fa-shield-halved"></i>
                </div>
                <h3>Military-Grade Security</h3>
                <p>AES-256 encryption and multi-signature cold storage for your assets.</p>
              </div>

              <div className={styles.visualCard}>
                <div className={styles.cardIcon}>
                  <i className="fas fa-bolt"></i>
                </div>
                <h3>Settled in Seconds</h3>
                <p>Real-time transaction confirmation across 5+ blockchain networks.</p>
              </div>

              <div className={styles.visualCard}>
                <div className={styles.cardIcon}>
                  <i className="fas fa-chart-line"></i>
                </div>
                <h3>Smart Analytics</h3>
                <p>Comprehensive dashboard for tracking revenue and payment history.</p>
              </div>

              <div className={styles.visualCard}>
                <div className={styles.cardIcon}>
                  <i className="fas fa-globe"></i>
                </div>
                <h3>Global Reach</h3>
                <p>Accept payments from anyone, anywhere, with zero currency boundaries.</p>
              </div>
            </div>

            <div className={styles.visualFooter}>
              <div className={styles.footerStat}>
                <strong>99.9%</strong>
                <span>Uptime</span>
              </div>
              <div className={styles.footerStat}>
                <strong>0.1%</strong>
                <span>Fraud Rate</span>
              </div>
              <div className={styles.footerStat}>
                <strong>24/7</strong>
                <span>Support</span>
              </div>
            </div>
          </div>
        </div>

        {/* Right Aspect: Login Form */}
        <div className={`${styles.formSide} animate-slide-in-right`}>
          <div className={styles.loginCard}>
            <div className={styles.cardHeader}>
              <div className={styles.cardLogo}>
                <img src="/logo/logo-brandmark.svg" alt="FidduPay" style={{ height: '36px', marginBottom: '1.5rem' }} />
              </div>
              <h2 className={styles.title}>Welcome Back</h2>
              <p className={styles.subtitle}>Sign in to manage your merchant portal</p>
            </div>

            <form onSubmit={handleSubmit} className={styles.form}>
              <div className={styles.inputGroup}>
                <label htmlFor="email">Email Address</label>
                <div className={styles.inputWrapper}>
                  <i className="fas fa-envelope"></i>
                  <input
                    type="email"
                    id="email"
                    name="email"
                    value={formData.email}
                    onChange={handleChange}
                    placeholder="name@business.com"
                    required
                  />
                </div>
              </div>

              <div className={styles.inputGroup}>
                <div className={styles.labelRow}>
                  <label htmlFor="password">Password</label>
                  <Link to="/forgot-password" className={styles.forgotLink}>Forgot?</Link>
                </div>
                <div className={styles.inputWrapper}>
                  <i className="fas fa-lock"></i>
                  <input
                    type={showPassword ? "text" : "password"}
                    id="password"
                    name="password"
                    value={formData.password}
                    onChange={handleChange}
                    placeholder="Enter password"
                    required
                  />
                  <button
                    type="button"
                    className={styles.passwordToggle}
                    onClick={() => setShowPassword(!showPassword)}
                  >
                    <i className={`fas ${showPassword ? 'fa-eye-slash' : 'fa-eye'}`}></i>
                  </button>
                </div>
              </div>

              <div className={styles.formOptions}>
                <label className={styles.checkboxContainer}>
                  <input
                    type="checkbox"
                    checked={rememberMe}
                    onChange={(e) => setRememberMe(e.target.checked)}
                  />
                  <span className={styles.checkmark}></span>
                  Remember this device
                </label>
              </div>

              <div className={styles.inputGroup}>
                <label htmlFor="two_factor_code">2FA Code <span className={styles.optional}>(Optional)</span></label>
                <div className={styles.inputWrapper}>
                  <i className="fas fa-key"></i>
                  <input
                    type="text"
                    id="two_factor_code"
                    name="two_factor_code"
                    value={formData.two_factor_code}
                    onChange={handleChange}
                    placeholder="6-digit code"
                  />
                </div>
              </div>

              <button type="submit" className={styles.submitButton}>
                <span>Sign In to Account</span>
                <i className="fas fa-arrow-right"></i>
              </button>
            </form>

            <div className={styles.cardFooter}>
              <p>
                Don't have an account?{' '}
                <Link to="/register" className={styles.signupLink}>
                  Start for Free
                </Link>
              </p>
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}

export default LoginPage
