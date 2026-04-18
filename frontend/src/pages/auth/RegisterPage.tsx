import React, { useState, useMemo } from 'react'
import { Link, useNavigate } from 'react-router-dom'
import { useToast } from '@/contexts/ToastContext'
import { useLoading } from '@/contexts/LoadingContext'
import { authAPI } from '@/services/apiService'
import styles from '@/styles/pages/auth/RegisterPage.module.css'

interface StrengthResult {
  score: number; // 0-4
  label: string;
  color: string;
}

const RegisterPage: React.FC = () => {
  const [step, setStep] = useState(0)
  const [role, setRole] = useState<'merchant' | 'user' | null>(null)

  const [formData, setFormData] = useState({
    // Account
    email: '',
    password: '',
    confirmPassword: '',

    // Personal KYC
    firstName: '',
    lastName: '',
    gender: '',
    phoneNumber: '',
    country: '',
    applicantRole: '',
    agreeToTerms: false,

    // Business KYC
    businessName: '',
    businessCountry: '',
    businessLicenseNumber: '',
    businessCertificateUrl: '',
  })

  const [showPassword, setShowPassword] = useState(false)
  const [showConfirmPassword, setShowConfirmPassword] = useState(false)
  const { showToast } = useToast()
  const { setLoading } = useLoading()
  const navigate = useNavigate()

  // --- Password Strength Logic ---
  const strength: StrengthResult = useMemo(() => {
    const pwd = formData.password
    if (!pwd) return { score: 0, label: 'Empty', color: '#e2e8f0' }

    let score = 0
    if (pwd.length >= 8) score++
    if (/[A-Z]/.test(pwd)) score++
    if (/[0-9]/.test(pwd)) score++
    if (/[^A-Za-z0-9]/.test(pwd)) score++

    const labels = ['Weak', 'Weak', 'Fair', 'Good', 'Strong']
    const colors = ['#ef4444', '#ef4444', '#f59e0b', '#3b82f6', '#059669']

    return {
      score,
      label: labels[score],
      color: colors[score]
    }
  }, [formData.password])

  const passwordsMatch = formData.password && formData.confirmPassword 
    ? formData.password === formData.confirmPassword 
    : null

  const handleRoleSelect = (selectedRole: 'merchant' | 'user') => {
    if (selectedRole === 'user') return // Disabled for now
    setRole(selectedRole)
    setStep(1)
  }

  const handleNext = () => {
    if (step === 1) {
      if (!formData.email || !formData.password || !formData.confirmPassword) {
        showToast('Please fill in all account fields', 'error')
        return
      }
      if (strength.score < 2) {
        showToast('Please use a stronger password', 'error')
        return
      }
      if (formData.password !== formData.confirmPassword) {
        showToast('Passwords do not match', 'error')
        return
      }
      setStep(2)
    } else if (step === 2) {
      if (!formData.firstName || !formData.lastName || !formData.phoneNumber || !formData.country) {
        showToast('Please fill in all mandatory personal details', 'error')
        return
      }
      if (!formData.agreeToTerms) {
        showToast('You must accept the terms to continue', 'error')
        return
      }
      
      if (role === 'merchant') {
        if (!formData.applicantRole) {
          showToast('Please select your role in the company', 'error')
          return
        }
        setStep(3)
      } else {
        handleSubmit()
      }
    }
  }

  const handleBack = () => {
    setStep(step - 1)
  }

  const handleSubmit = async (e?: React.FormEvent) => {
    if (e) e.preventDefault()

    if (role === 'merchant' && step === 3) {
      if (!formData.businessName || !formData.businessCountry) {
        showToast('Business name and country are mandatory', 'error')
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
      }

      showToast('Registration successful!', 'success')
      setTimeout(() => navigate('/login'), 1500)
    } catch (error: any) {
      const message = error.response?.data?.error?.message || error.message || 'Registration failed'
      showToast(message, 'error')
    } finally {
      setLoading(false)
    }
  }

  const handleChange = (e: React.ChangeEvent<HTMLInputElement | HTMLSelectElement>) => {
    const { name, value, type } = e.target
    const checked = (e.target as HTMLInputElement).checked
    setFormData(prev => ({
      ...prev,
      [name]: type === 'checkbox' ? checked : value
    }))
  }

  // --- Sub-components (Render Helpers) ---
  const renderStepper = () => (
    <div className={styles.stepper}>
      <div className={styles.stepWrapper}>
        <div className={`${styles.stepCircle} ${step >= 1 ? styles.active : ''} ${step > 1 ? styles.completed : ''}`}>
          {step > 1 ? <i className="fas fa-check"></i> : '1'}
        </div>
        <span className={`${styles.stepLabel} ${step === 1 ? styles.active : ''}`}>Account</span>
      </div>
      <div className={styles.stepWrapper}>
        <div className={`${styles.stepCircle} ${step >= 2 ? styles.active : ''} ${step > 2 ? styles.completed : ''}`}>
          {step > 2 ? <i className="fas fa-check"></i> : '2'}
        </div>
        <span className={`${styles.stepLabel} ${step === 2 ? styles.active : ''}`}>Personal</span>
      </div>
      <div className={styles.stepWrapper}>
        <div className={`${styles.stepCircle} ${step >= 3 ? styles.active : ''}`}>
          3
        </div>
        <span className={`${styles.stepLabel} ${step === 3 ? styles.active : ''}`}>Business</span>
      </div>
    </div>
  )

  return (
    <div className={styles.registerPage}>
      <div className={styles.container}>
        <div className={styles.registerCard}>
          <div className={styles.header}>
            <Link to="/" className={styles.logo}>FidduPay</Link>
            <h1 className={styles.title}>
              {step === 0 ? 'Create Account' :
                step === 1 ? 'Setup Account' :
                  step === 2 ? 'Personal KYC' :
                    'Business Details'}
            </h1>
            <p className={styles.subtitle}>
              {step === 0 ? 'Select your path to continue' :
                step === 1 ? 'Secure your merchant identity' :
                  step === 2 ? 'Help us verify your primary contact' :
                    'Complete your business registration'}
            </p>
          </div>

          {step > 0 && renderStepper()}

          {step === 0 && (
            <div className={styles.roleSelection}>
              <div
                className={`${styles.roleCard} ${styles.disabled}`}
                title="Coming Soon"
              >
                <div className={styles.comingSoonBadge}>Coming Soon</div>
                <div className={styles.roleIcon}>
                  <i className="fas fa-user-circle" style={{ color: '#94a3b8' }}></i>
                </div>
                <h3>Personal Account</h3>
                <p>Personal crypto usage & management.</p>
              </div>
              <div
                className={`${styles.roleCard} ${role === 'merchant' ? styles.selected : ''}`}
                onClick={() => handleRoleSelect('merchant')}
              >
                <div className={styles.roleIcon}>
                  <i className="fas fa-briefcase" style={{ color: '#1e40af' }}></i>
                </div>
                <h3>Merchant</h3>
                <p>Accept crypto payments for your business.</p>
              </div>
            </div>
          )}

          {step === 1 && (
            <div className={styles.form}>
              <div className={styles.inputGroup}>
                <label htmlFor="email" className={styles.mandatoryLabel}>Official Email Address</label>
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

              <div className={styles.inputGroup}>
                <label htmlFor="password" className={styles.mandatoryLabel}>Secure Password</label>
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
                {/* Strength Indicator */}
                <div className={styles.strengthContainer}>
                  <div className={styles.strengthBar}>
                    {[1, 2, 3, 4].map((s) => (
                      <div
                        key={s}
                        className={styles.strengthSegment}
                        style={{ 
                          backgroundColor: s <= strength.score ? strength.color : '#e2e8f0' 
                        }}
                      />
                    ))}
                  </div>
                  <div className={styles.strengthLabel}>
                    <span className={styles.strengthText} style={{ color: strength.color }}>
                      Strength: {strength.label}
                    </span>
                  </div>
                </div>
              </div>

              <div className={styles.inputGroup}>
                <label htmlFor="confirmPassword" className={styles.mandatoryLabel}>Confirm Password</label>
                <div className={styles.passwordWrapper}>
                  <input
                    type={showConfirmPassword ? "text" : "password"}
                    id="confirmPassword"
                    name="confirmPassword"
                    value={formData.confirmPassword}
                    onChange={handleChange}
                    placeholder="Repeat password"
                    required
                  />
                  <button
                    type="button"
                    className={styles.passwordToggle}
                    onClick={() => setShowConfirmPassword(!showConfirmPassword)}
                  >
                    <i className={`fas ${showConfirmPassword ? 'fa-eye-slash' : 'fa-eye'}`}></i>
                  </button>
                </div>
                {passwordsMatch === false && (
                  <p className={`${styles.matchIndicator} ${styles.error}`}>
                    <i className="fas fa-times-circle"></i> Passwords do not match
                  </p>
                )}
                {passwordsMatch === true && (
                  <p className={`${styles.matchIndicator} ${styles.success}`}>
                    <i className="fas fa-check-circle"></i> Passwords match
                  </p>
                )}
              </div>

              <div className={styles.formActions}>
                <button type="button" onClick={handleBack} className={styles.backButton}>Change Role</button>
                <button onClick={handleNext} className={styles.submitButton}>Continue to Identity</button>
              </div>
            </div>
          )}

          {step === 2 && (
            <div className={styles.form}>
              <div className={styles.inputRow}>
                <div className={styles.inputGroup}>
                  <label htmlFor="firstName" className={styles.mandatoryLabel}>First Name</label>
                  <input
                    type="text"
                    name="firstName"
                    value={formData.firstName}
                    onChange={handleChange}
                    placeholder="Enter first name"
                    required
                  />
                </div>
                <div className={styles.inputGroup}>
                  <label htmlFor="lastName" className={styles.mandatoryLabel}>Last Name</label>
                  <input
                    type="text"
                    name="lastName"
                    value={formData.lastName}
                    onChange={handleChange}
                    placeholder="Enter last name"
                    required
                  />
                </div>
              </div>

              <div className={styles.inputRow}>
                <div className={styles.inputGroup}>
                  <label htmlFor="gender">Gender</label>
                  <select name="gender" value={formData.gender} onChange={handleChange as any}>
                    <option value="">Choose Gender</option>
                    <option value="male">Male</option>
                    <option value="female">Female</option>
                    <option value="other">Other</option>
                  </select>
                </div>
                <div className={styles.inputGroup}>
                  <label htmlFor="phoneNumber" className={styles.mandatoryLabel}>Mobile Number</label>
                  <input
                    type="tel"
                    name="phoneNumber"
                    value={formData.phoneNumber}
                    onChange={handleChange}
                    placeholder="+234..."
                    required
                  />
                </div>
              </div>

              <div className={styles.inputRow}>
                <div className={styles.inputGroup}>
                  <label htmlFor="country" className={styles.mandatoryLabel}>Country of Residence</label>
                  <input
                    type="text"
                    name="country"
                    value={formData.country}
                    onChange={handleChange}
                    placeholder="e.g. Nigeria"
                    required
                  />
                </div>
                <div className={styles.inputGroup}>
                  <label htmlFor="applicantRole" className={styles.mandatoryLabel}>Your Designation</label>
                  <select name="applicantRole" value={formData.applicantRole} onChange={handleChange as any} required>
                    <option value="">Select Position</option>
                    <option value="founder">Founder / CEO</option>
                    <option value="director">Director</option>
                    <option value="manager">Operations Manager</option>
                    <option value="agent">Authorized Agent</option>
                  </select>
                </div>
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
                  Confirm that I agree to the <Link to="/terms" className={styles.link}>Terms</Link> and <Link to="/privacy" className={styles.link}>Privacy Policies</Link>.
                </label>
              </div>

              <div className={styles.formActions}>
                <button type="button" onClick={handleBack} className={styles.backButton}>Back to Account</button>
                <button onClick={handleNext} className={styles.submitButton}>
                  {role === 'merchant' ? 'Next: Business details' : 'Complete Registration'}
                </button>
              </div>
            </div>
          )}

          {step === 3 && role === 'merchant' && (
            <form className={styles.form} onSubmit={handleSubmit}>
              <div className={styles.inputGroup}>
                <label htmlFor="businessName" className={styles.mandatoryLabel}>Registered Business Name</label>
                <input
                  type="text"
                  name="businessName"
                  value={formData.businessName}
                  onChange={handleChange}
                  placeholder="Official legal name"
                  required
                />
              </div>

              <div className={styles.inputRow}>
                <div className={styles.inputGroup}>
                  <label htmlFor="businessCountry" className={styles.mandatoryLabel}>Registration Country</label>
                  <input
                    type="text"
                    name="businessCountry"
                    value={formData.businessCountry}
                    onChange={handleChange}
                    placeholder="e.g. United Kingdom"
                    required
                  />
                </div>
                <div className={styles.inputGroup}>
                  <label htmlFor="businessLicenseNumber" className={styles.optionalLabel}>Registration Number</label>
                  <input
                    type="text"
                    name="businessLicenseNumber"
                    value={formData.businessLicenseNumber}
                    onChange={handleChange}
                    placeholder="RC / Tax ID"
                  />
                </div>
              </div>

              <div className={styles.inputGroup}>
                <label htmlFor="businessCertificateUrl" className={styles.optionalLabel}>Registration Document Link</label>
                <input
                  type="text"
                  name="businessCertificateUrl"
                  value={formData.businessCertificateUrl}
                  onChange={handleChange}
                  placeholder="Cloud link to your certificate"
                />
                <p className={styles.helperText}>Verification documents can be uploaded later via the security hub.</p>
              </div>

              <div className={styles.formActions}>
                <button type="button" onClick={handleBack} className={styles.backButton}>Back to Identity</button>
                <button type="submit" className={styles.submitButton}>Open Merchant Account</button>
              </div>
            </form>
          )}

          <div className={styles.footer}>
            <p>
              Returning to FidduPay?{' '}
              <Link to="/login" className={styles.link}>Sign In Here</Link>
            </p>
          </div>
        </div>
      </div>
    </div>
  )
}

export default RegisterPage
