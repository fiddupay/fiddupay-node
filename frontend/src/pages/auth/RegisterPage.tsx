import React, { useState } from 'react'
import { Link, useNavigate } from 'react-router-dom'
import { useToast } from '@/contexts/ToastContext'
import { useLoading } from '@/contexts/LoadingContext'
import { authAPI } from '@/services/apiService'
import styles from '@/styles/pages/auth/RegisterPage.module.css'

const RegisterPage: React.FC = () => {
  const [step, setStep] = useState(0)
  const [role, setRole] = useState<'merchant' | 'user' | null>(null)

  const [formData, setFormData] = useState({
    // Account
    email: '',
    password: '',
    confirmPassword: '',

    // Personal KYC (Step 1)
    firstName: '',
    lastName: '',
    gender: '',
    phoneNumber: '',
    country: '',
    applicantRole: '', // For Merchant
    nickname: '', // For User
    agreeToTerms: false,

    // Business KYC (Step 2 - Merchant Only)
    businessName: '',
    businessCountry: '',
    businessLicenseNumber: '',
    businessCertificateUrl: '', // Placeholder for now
  })

  const [showPassword, setShowPassword] = useState(false)
  const { showToast } = useToast()
  const { setLoading } = useLoading()
  const navigate = useNavigate()

  const handleRoleSelect = (selectedRole: 'merchant' | 'user') => {
    setRole(selectedRole)
    setStep(1)
  }

  const handleNext = () => {
    // Validation for Step 1
    if (step === 1) {
      if (!formData.firstName || !formData.lastName || !formData.email || !formData.password || !formData.country || !formData.phoneNumber) {
        showToast('Please fill in all mandatory fields', 'error')
        return
      }
      if (formData.password !== formData.confirmPassword) {
        showToast('Passwords do not match', 'error')
        return
      }
      if (!formData.agreeToTerms) {
        showToast('You must accept the terms and conditions', 'error')
        return
      }

      if (role === 'merchant') {
        if (!formData.applicantRole) {
          showToast('Please select your role in the company', 'error')
          return
        }
        setStep(2)
      } else {
        // For User, Step 1 is the final step before submission
        handleSubmit()
      }
    }
  }

  const handleBack = () => {
    setStep(step - 1)
  }

  const handleSubmit = async (e?: React.FormEvent) => {
    if (e) e.preventDefault()

    // Validation for Step 2 (Merchant Only)
    if (role === 'merchant') {
      if (!formData.businessName || !formData.businessCountry) {
        showToast('Business Name and Country are mandatory', 'error')
        return
      }
    }

    setLoading(true)
    try {
      if (role === 'merchant') {
        await authAPI.register({
          email: formData.email.toLowerCase(),
          password: formData.password,
          business_name: formData.businessName,
          first_name: formData.firstName,
          last_name: formData.lastName,
          gender: formData.gender,
          phone_number: formData.phoneNumber,
          country: formData.country,
          applicant_role: formData.applicantRole,
          terms_accepted: formData.agreeToTerms,
          business_country: formData.businessCountry,
          business_license_number: formData.businessLicenseNumber || null,
          business_certificate_url: formData.businessCertificateUrl || null,
        })
      } else {
        await authAPI.registerP2P({
          email: formData.email.toLowerCase(),
          password: formData.password,
          nickname: formData.nickname || `${formData.firstName}${formData.lastName[0]}`.toLowerCase(),
          first_name: formData.firstName,
          last_name: formData.lastName,
          gender: formData.gender,
          phone_number: formData.phoneNumber,
          country: formData.country,
          terms_accepted: formData.agreeToTerms,
        })
      }

      showToast('Registration successful! Redirecting...', 'success')
      setTimeout(() => navigate('/login'), 2000)
    } catch (error: any) {
      const message = error.response?.data?.error?.message || error.response?.data?.message || error.message || 'Registration failed'
      showToast(message, 'error')
    } finally {
      setLoading(false)
    }
  }

  const handleChange = (e: React.ChangeEvent<HTMLInputElement | HTMLSelectElement>) => {
    const { name, value, type } = e.target
    const checked = (e.target as HTMLInputElement).checked
    setFormData({
      ...formData,
      [name]: type === 'checkbox' ? checked : value
    })
  }

  return (
    <div className={styles.registerPage}>
      <div className={styles.container}>
        <div className={styles.registerCard}>
          <div className={styles.header}>
            <Link to="/" className={styles.logo}>FidduPay</Link>
            <h1 className={styles.title}>
              {step === 0 ? 'Choose Account Type' :
                step === 1 ? 'Personal Information' :
                  'Business Details'}
            </h1>
            <p className={styles.subtitle}>
              {step === 0 ? 'Select how you want to use FidduPay' :
                step === 1 ? 'Tell us a bit about yourself' :
                  'Provide your business information (Optional to complete now)'}
            </p>
          </div>

          {step > 0 && (
            <div className={styles.stepIndicator}>
              <div className={`${styles.step} ${step >= 1 ? styles.active : ''}`}></div>
              {role === 'merchant' && (
                <div className={`${styles.step} ${step >= 2 ? styles.active : ''}`}></div>
              )}
            </div>
          )}

          {step === 0 && (
            <div className={styles.roleSelection}>
              <div
                className={`${styles.roleCard} ${role === 'user' ? styles.selected : ''}`}
                onClick={() => handleRoleSelect('user')}
              >
                <div className={styles.roleIcon}>
                  <i className="fas fa-user"></i>
                </div>
                <h3>Individual User</h3>
                <p>Buy, sell and trade crypto P2P with ease.</p>
              </div>
              <div
                className={`${styles.roleCard} ${role === 'merchant' ? styles.selected : ''}`}
                onClick={() => handleRoleSelect('merchant')}
              >
                <div className={styles.roleIcon}>
                  <i className="fas fa-building"></i>
                </div>
                <h3>Merchant</h3>
                <p>Accept crypto payments for your business.</p>
              </div>
            </div>
          )}

          {step === 1 && (
            <form className={styles.form} onSubmit={(e) => { e.preventDefault(); handleNext(); }}>
              <div className={styles.inputRow}>
                <div className={styles.inputGroup}>
                  <label htmlFor="firstName" className={styles.mandatoryLabel}>First Name</label>
                  <input
                    type="text"
                    id="firstName"
                    name="firstName"
                    value={formData.firstName}
                    onChange={handleChange}
                    placeholder="John"
                    required
                  />
                </div>
                <div className={styles.inputGroup}>
                  <label htmlFor="lastName" className={styles.mandatoryLabel}>Last Name</label>
                  <input
                    type="text"
                    id="lastName"
                    name="lastName"
                    value={formData.lastName}
                    onChange={handleChange}
                    placeholder="Doe"
                    required
                  />
                </div>
              </div>

              <div className={styles.inputRow}>
                <div className={styles.inputGroup}>
                  <label htmlFor="gender">Gender</label>
                  <select
                    id="gender"
                    name="gender"
                    value={formData.gender}
                    onChange={handleChange as any}
                    className={styles.select}
                  >
                    <option value="">Select Gender</option>
                    <option value="male">Male</option>
                    <option value="female">Female</option>
                    <option value="other">Other</option>
                  </select>
                </div>
                <div className={styles.inputGroup}>
                  <label htmlFor="phoneNumber" className={styles.mandatoryLabel}>Phone Number</label>
                  <input
                    type="tel"
                    id="phoneNumber"
                    name="phoneNumber"
                    value={formData.phoneNumber}
                    onChange={handleChange}
                    placeholder="+1234567890"
                    required
                  />
                </div>
              </div>

              <div className={styles.inputGroup}>
                <label htmlFor="email" className={styles.mandatoryLabel}>Email Address</label>
                <input
                  type="email"
                  id="email"
                  name="email"
                  value={formData.email}
                  onChange={handleChange}
                  placeholder="john@example.com"
                  required
                />
              </div>

              <div className={styles.inputRow}>
                <div className={styles.inputGroup}>
                  <label htmlFor="password" className={styles.mandatoryLabel}>Password</label>
                  <div className={styles.passwordWrapper}>
                    <input
                      type={showPassword ? "text" : "password"}
                      id="password"
                      name="password"
                      value={formData.password}
                      onChange={handleChange}
                      placeholder="Min. 8 characters"
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
                <div className={styles.inputGroup}>
                  <label htmlFor="confirmPassword" className={styles.mandatoryLabel}>Confirm Password</label>
                  <input
                    type="password"
                    id="confirmPassword"
                    name="confirmPassword"
                    value={formData.confirmPassword}
                    onChange={handleChange}
                    placeholder="Repeat password"
                    required
                  />
                </div>
              </div>

              <div className={styles.inputRow}>
                <div className={styles.inputGroup}>
                  <label htmlFor="country" className={styles.mandatoryLabel}>Country</label>
                  <input
                    type="text"
                    id="country"
                    name="country"
                    value={formData.country}
                    onChange={handleChange}
                    placeholder="e.g. United States"
                    required
                  />
                </div>
                {role === 'merchant' ? (
                  <div className={styles.inputGroup}>
                    <label htmlFor="applicantRole" className={styles.mandatoryLabel}>Your Role</label>
                    <select
                      id="applicantRole"
                      name="applicantRole"
                      value={formData.applicantRole}
                      onChange={handleChange as any}
                      required
                    >
                      <option value="">Select Role</option>
                      <option value="founder">Founder/CEO</option>
                      <option value="cto">CTO</option>
                      <option value="manager">Manager</option>
                      <option value="other">Other</option>
                    </select>
                  </div>
                ) : (
                  <div className={styles.inputGroup}>
                    <label htmlFor="nickname">Nickname</label>
                    <input
                      type="text"
                      id="nickname"
                      name="nickname"
                      value={formData.nickname}
                      onChange={handleChange}
                      placeholder="Display name"
                    />
                  </div>
                )}
              </div>

              <div className={styles.checkboxGroup}>
                <input
                  type="checkbox"
                  id="agreeToTerms"
                  name="agreeToTerms"
                  checked={formData.agreeToTerms}
                  onChange={handleChange}
                  required
                />
                <label htmlFor="agreeToTerms">
                  I agree to the <Link to="/terms" className={styles.link}>Terms of Service</Link> and{' '}
                  <Link to="/privacy" className={styles.link}>Privacy Policy</Link>
                </label>
              </div>

              <div className={styles.formActions}>
                <button type="button" onClick={handleBack} className={styles.backButton}>Back</button>
                <button type="submit" className={styles.submitButton}>
                  {role === 'merchant' ? 'Next: Business Details' : 'Create Account'}
                </button>
              </div>
            </form>
          )}

          {step === 2 && role === 'merchant' && (
            <form className={styles.form} onSubmit={(e) => { e.preventDefault(); handleSubmit(); }}>
              <div className={styles.inputGroup}>
                <label htmlFor="businessName" className={styles.mandatoryLabel}>Business Name</label>
                <input
                  type="text"
                  id="businessName"
                  name="businessName"
                  value={formData.businessName}
                  onChange={handleChange}
                  placeholder="Company Legal Name"
                  required
                />
              </div>

              <div className={styles.inputGroup}>
                <label htmlFor="businessCountry" className={styles.mandatoryLabel}>Business Country</label>
                <input
                  type="text"
                  id="businessCountry"
                  name="businessCountry"
                  value={formData.businessCountry}
                  onChange={handleChange}
                  placeholder="Registration Country"
                  required
                />
              </div>

              <div className={styles.inputGroup}>
                <label htmlFor="businessLicenseNumber" className={styles.optionalLabel}>Business License / Registration Number</label>
                <input
                  type="text"
                  id="businessLicenseNumber"
                  name="businessLicenseNumber"
                  value={formData.businessLicenseNumber}
                  onChange={handleChange}
                  placeholder="e.g. RC123456"
                />
              </div>

              <div className={styles.inputGroup}>
                <label htmlFor="businessCertificateUrl" className={styles.optionalLabel}>Business Certificate (CAC/Incorporation)</label>
                <input
                  type="text"
                  id="businessCertificateUrl"
                  name="businessCertificateUrl"
                  value={formData.businessCertificateUrl}
                  onChange={handleChange}
                  placeholder="Upload link or leave empty to complete later"
                />
                <p className={styles.helperText}>You can provide this later in your dashboard settings.</p>
              </div>

              <div className={styles.formActions}>
                <button type="button" onClick={handleBack} className={styles.backButton}>Back</button>
                <button type="submit" className={styles.submitButton}>Complete Registration</button>
              </div>
            </form>
          )}

          <div className={styles.footer}>
            <p>
              Already have an account?{' '}
              <Link to="/login" className={styles.link}>Sign In</Link>
            </p>
          </div>
        </div>
      </div>
    </div>
  )
}

export default RegisterPage
