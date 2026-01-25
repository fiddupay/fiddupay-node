import React, { useState } from 'react'
import { Link, Navigate } from 'react-router-dom'
import { useAuthStore } from '@/stores/authStore'
import { useToast } from '@/contexts/ToastContext'
import { useLoading } from '@/contexts/LoadingContext'
import styles from './LoginPage.module.css'

const LoginPage: React.FC = () => {
  const { login, error, isAuthenticated } = useAuthStore()
  const { showToast } = useToast()
  const { setLoading } = useLoading()
  const [showPassword, setShowPassword] = useState(false)
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
    
    if (!formData.email || !formData.password) {
      showToast('Please fill in all required fields', 'error')
      return
    }

    setLoading(true)
    try {
      await login(formData)
      showToast('Login successful!', 'success')
    } catch (error) {
      showToast('Login failed. Please check your credentials.', 'error')
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
      <div className={styles.container}>
        <div className={styles.loginCard}>
          <div className={styles.header}>
            <Link to="/" className={styles.logo}>FidduPay</Link>
            <h1 className={styles.title}>Welcome Back</h1>
            <p className={styles.subtitle}>Sign in to your merchant account</p>
          </div>

          <form onSubmit={handleSubmit} className={styles.form}>
            {error && (
              <div className={styles.errorAlert}>
                {error}
              </div>
            )}

            <div className={styles.inputGroup}>
              <label htmlFor="email">Email Address</label>
              <input
                type="email"
                id="email"
                name="email"
                value={formData.email}
                onChange={handleChange}
                placeholder="Enter your email"
                required
              />
            </div>

            <div className={styles.inputGroup}>
              <label htmlFor="password">Password</label>
              <div className={styles.passwordWrapper}>
                <input
                  type={showPassword ? "text" : "password"}
                  id="password"
                  name="password"
                  value={formData.password}
                  onChange={handleChange}
                  placeholder="Enter your password"
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
              <Link to="/forgot-password" className={styles.forgotPassword}>
                Forgot your password?
              </Link>
            </div>

            <div className={styles.inputGroup}>
              <label htmlFor="two_factor_code">2FA Code (Optional)</label>
              <input
                type="text"
                id="two_factor_code"
                name="two_factor_code"
                value={formData.two_factor_code}
                onChange={handleChange}
                placeholder="Enter 6-digit code"
              />
            </div>

            <button
              type="submit"
              className={styles.submitButton}
            >
              Sign In
            </button>
          </form>

          <div className={styles.footer}>
            <p>
              Don't have an account?{' '}
              <Link to="/register" className={styles.link}>
                Create Account
              </Link>
            </p>
            <p>
              <Link to="/forgot-password" className={styles.link}>
                Forgot your password?
              </Link>
            </p>
          </div>
        </div>
      </div>
    </div>
  )
}

export default LoginPage
