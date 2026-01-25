import React, { useState } from 'react'
import { Link } from 'react-router-dom'
import { useToast } from '@/contexts/ToastContext'
import { useLoading } from '@/contexts/LoadingContext'
import styles from './ForgotPasswordPage.module.css'

const ForgotPasswordPage: React.FC = () => {
  const [email, setEmail] = useState('')
  const [isSubmitted, setIsSubmitted] = useState(false)
  const { showToast } = useToast()
  const { setLoading } = useLoading()

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    
    if (!email) {
      showToast('Please enter your email address', 'error')
      return
    }

    setLoading(true)
    
    try {
      // Simulate API call
      await new Promise(resolve => setTimeout(resolve, 2000))
      
      setIsSubmitted(true)
      showToast('Password reset instructions sent to your email', 'success')
    } catch (error) {
      showToast('Failed to send reset instructions. Please try again.', 'error')
    } finally {
      setLoading(false)
    }
  }

  if (isSubmitted) {
    return (
      <div className={styles.forgotPasswordPage}>
        <div className={styles.container}>
          <div className={styles.card}>
            <div className={styles.header}>
              <Link to="/" className={styles.logo}>FidduPay</Link>
              <h1>Check Your Email</h1>
              <p>We've sent password reset instructions to <strong>{email}</strong></p>
            </div>

            <div className={styles.successMessage}>
              <i className="fas fa-envelope-open"></i>
              <h2>Email Sent!</h2>
              <p>
                If an account with that email exists, you'll receive password reset 
                instructions within a few minutes.
              </p>
              <p>
                Didn't receive the email? Check your spam folder or{' '}
                <button 
                  onClick={() => setIsSubmitted(false)}
                  className={styles.linkButton}
                >
                  try again
                </button>
              </p>
            </div>

            <div className={styles.footer}>
              <Link to="/login" className={styles.backLink}>
                <i className="fas fa-arrow-left"></i>
                Back to Login
              </Link>
            </div>
          </div>
        </div>
      </div>
    )
  }

  return (
    <div className={styles.forgotPasswordPage}>
      <div className={styles.container}>
        <div className={styles.card}>
          <div className={styles.header}>
            <Link to="/" className={styles.logo}>FidduPay</Link>
            <h1>Reset Your Password</h1>
            <p>Enter your email address and we'll send you instructions to reset your password.</p>
          </div>

          <form onSubmit={handleSubmit} className={styles.form}>
            <div className={styles.inputGroup}>
              <label htmlFor="email">Email Address</label>
              <div className={styles.inputWrapper}>
                <i className="fas fa-envelope"></i>
                <input
                  type="email"
                  id="email"
                  value={email}
                  onChange={(e) => setEmail(e.target.value)}
                  placeholder="Enter your email address"
                  required
                />
              </div>
            </div>

            <button type="submit" className={styles.submitBtn}>
              <i className="fas fa-paper-plane"></i>
              Send Reset Instructions
            </button>
          </form>

          <div className={styles.footer}>
            <Link to="/login" className={styles.backLink}>
              <i className="fas fa-arrow-left"></i>
              Back to Login
            </Link>
            <span className={styles.divider}>•</span>
            <Link to="/register" className={styles.registerLink}>
              Create Account
            </Link>
          </div>
        </div>
      </div>
    </div>
  )
}

export default ForgotPasswordPage
