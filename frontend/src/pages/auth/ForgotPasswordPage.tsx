import React, { useState } from 'react'
import { Link } from 'react-router-dom'
import { useToast } from '@/contexts/ToastContext'
import { useLoading } from '@/contexts/LoadingContext'
import styles from '@/styles/pages/auth/ForgotPasswordPage.module.css'

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
    } catch (error: any) {
      showToast('Failed to send reset instructions. Please try again.', 'error')
    } finally {
      setLoading(false)
    }
  }

  return (
    <div className={styles.forgotPasswordPage}>
      {/* Background Ambient Glow */}
      <div className={styles.ambientGlowContainer}>
        <div className={`${styles.blob} ${styles.blobPrimary}`}></div>
        <div className={`${styles.blob} ${styles.blobSecondary}`}></div>
      </div>

      <div className={styles.container}>
        <div className={`${styles.card} animate-fade-in-up`}>
          <div className={styles.header}>
            <Link to="/" className={styles.logo}>
              <img src="/logo/logo-brandmark.svg" alt="FidduPay" style={{ height: '36px' }} />
            </Link>
            
            {isSubmitted ? (
               <h1>Check Your Inbox</h1>
            ) : (
               <h1>Reset Password</h1>
            )}
            
            <p>
              {isSubmitted 
                ? `We've sent recovery instructions to ${email}`
                : "Enter your secure email to receive a password reset link."}
            </p>
          </div>

          {isSubmitted ? (
            <div className={styles.successMessage}>
              <i className="fas fa-envelope-circle-check"></i>
              <h2>Instructions Sent!</h2>
              <p>
                Please check your inbox (and spam folder) for the reset link. 
                The link will expire in 60 minutes for security purposes.
              </p>
              <button
                onClick={() => setIsSubmitted(false)}
                className={styles.linkButton}
              >
                Send to a different email
              </button>
            </div>
          ) : (
            <form onSubmit={handleSubmit} className={styles.form}>
              <div className={styles.inputGroup}>
                <label htmlFor="email">Work Email</label>
                <div className={styles.inputWrapper}>
                  <i className="fas fa-envelope"></i>
                  <input
                    type="email"
                    id="email"
                    value={email}
                    onChange={(e) => setEmail(e.target.value)}
                    placeholder="name@company.com"
                    required
                  />
                </div>
              </div>

              <button type="submit" className={styles.submitBtn}>
                <span>Send Instructions</span>
                <i className="fas fa-paper-plane"></i>
              </button>
            </form>
          )}

          <div className={styles.footer}>
            <Link to="/login" className={styles.backLink}>
              <i className="fas fa-arrow-left"></i>
              Back to Login
            </Link>
            <span className={styles.divider}>•</span>
            <Link to="/register" className={styles.registerLink}>
              Join Beta
            </Link>
          </div>
        </div>
      </div>
    </div>
  )
}

export default ForgotPasswordPage
