import React, { useState, useMemo } from 'react'
import { Link, useNavigate } from 'react-router-dom'
import { useToast } from '@/contexts/ToastContext'
import { useLoading } from '@/contexts/LoadingContext'
import { authAPI } from '@/services/apiService'
import CustomSelect from '@/components/ui/CustomSelect'
import styles from '@/styles/pages/auth/RegisterPage.module.css'
import SEO from '@/components/ui/SEO'

interface StrengthResult {
  score: number;
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
    customRole: '',
    agreeToTerms: false,

    // Business KYC
    businessName: '',
    businessCountry: '',
    businessLicenseNumber: '',
    businessCertificateUrl: '',
    website: '',

    // Compliance / Socials
    nin_bvn: '',
    twitter_handle: '',
    instagram_handle: '',
  })

  const [showPassword, setShowPassword] = useState(false)
  const [showConfirmPassword, setShowConfirmPassword] = useState(false)
  const { showToast } = useToast()
  const { setLoading } = useLoading()
  const navigate = useNavigate()

  const strength: StrengthResult = useMemo(() => {
    const pwd = formData.password
    if (!pwd) return { score: 0, label: 'Empty', color: '#374151' }

    let score = 0
    if (pwd.length >= 8) score++
    if (/[A-Z]/.test(pwd)) score++
    if (/[0-9]/.test(pwd)) score++
    if (/[^A-Za-z0-9]/.test(pwd)) score++

    const labels = ['Weak', 'Weak', 'Fair', 'Good', 'Strong']
    const colors = ['#ef4444', '#ef4444', '#f59e0b', '#3b82f6', '#2dd4bf']

    return { score, label: labels[score], color: colors[score] }
  }, [formData.password])

  const passwordsMatch = formData.password && formData.confirmPassword 
    ? formData.password === formData.confirmPassword 
    : null

  const handleRoleSelect = (selectedRole: 'merchant' | 'user') => {
    if (selectedRole === 'user') {
      showToast('Personal accounts are currently disabled. Join as a merchant!', 'info')
      return
    }
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

  const handleBack = () => setStep(step - 1)

  const handleSubmit = async (e?: React.FormEvent) => {
    if (e) e.preventDefault()
    if (role === 'merchant' && step === 3) {
      if (!formData.businessName || !formData.businessCountry || !formData.businessLicenseNumber || !formData.website) {
        showToast('Business details including Registration Number and Website are mandatory', 'error')
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
          applicant_role: formData.applicantRole === 'other' ? formData.customRole : formData.applicantRole,
          terms_accepted: formData.agreeToTerms,
          business_country: formData.businessCountry,
          business_license_number: formData.businessLicenseNumber,
          business_certificate_url: formData.businessCertificateUrl || null,
          website_url: formData.website,
          nin_bvn: formData.country === 'Nigeria' ? (formData.nin_bvn || null) : null,
          twitter_handle: formData.twitter_handle || null,
          instagram_handle: formData.instagram_handle || null,
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
    setFormData(prev => ({ ...prev, [name]: type === 'checkbox' ? checked : value }))
  }

  const renderStepper = () => (
    <div className={styles.stepper}>
      {[
        { id: 1, label: 'Account' },
        { id: 2, label: 'Identity' },
        { id: 3, label: 'Business' },
      ].map((s, i) => (
        <React.Fragment key={s.id}>
          <div className={styles.stepWrapper}>
            <div className={`${styles.stepCircle} ${step >= s.id ? styles.active : ''} ${step > s.id ? styles.completed : ''}`}>
              {step > s.id ? <i className="fas fa-check"></i> : s.id}
            </div>
            <span className={`${styles.stepLabel} ${step === s.id ? styles.activeLabel : ''}`}>{s.label}</span>
          </div>
          {i < 2 && <div className={`${styles.connector} ${step > s.id ? styles.connectorActive : ''}`}></div>}
        </React.Fragment>
      ))}
    </div>
  )

  return (
    <div className={styles.registerPage}>
      <SEO 
        title="Register | Start Accepting Crypto" 
        description="Join FidduPay today to start accepting Bitcoin, Solana, and Stablecoin payments for your business. Fast merchant onboarding and enterprise-grade security."
      />
      <div className={styles.ambientGlowContainer}>
        <div className={`${styles.blob} ${styles.blobPrimary}`}></div>
        <div className={`${styles.blob} ${styles.blobSecondary}`}></div>
      </div>

      <div className={styles.container}>
        <div className={styles.registerCard}>
          <div className={styles.header}>
            <h1 className={styles.title}>
              {step === 0 ? 'Join FidduPay' :
                step === 1 ? 'Step 1: Account' :
                  step === 2 ? 'Step 2: Verification' :
                    'Step 3: Business'}
            </h1>
            <p className={styles.subtitle}>
              {step === 0 ? 'Select your merchant path to get started' :
                'Help us secure your high-performance gateway account'}
            </p>
          </div>

          {step > 0 && renderStepper()}

          <div className={styles.stepContent}>
            {step === 0 && (
              <div className={styles.roleSelection}>
                 <div className={`${styles.roleCard} ${styles.disabled}`}>
                  <div className={styles.comingSoon}>Beta Soon</div>
                  <div className={styles.roleIcon}><i className="fas fa-user"></i></div>
                  <h3>Personal Account</h3>
                  <p>Pay peers and manage personal crypto portfolios.</p>
                </div>
                <div className={styles.roleCard} onClick={() => handleRoleSelect('merchant')}>
                  <div className={styles.roleIcon}><i className="fas fa-building"></i></div>
                  <h3>Merchant Account</h3>
                  <p>Accept crypto at scale for your business with L3 monitoring.</p>
                  <div className={styles.selectArrow}><i className="fas fa-arrow-right"></i></div>
                </div>
              </div>
            )}

            {step === 1 && (
              <div className={styles.formSection}>
                <div className={styles.inputGroup}>
                  <label>Business Email</label>
                  <div className={styles.inputWrapper}>
                    <i className="fas fa-envelope"></i>
                    <input type="email" name="email" value={formData.email} onChange={handleChange} placeholder="name@company.com" />
                  </div>
                </div>
                <div className={styles.inputGroup}>
                  <label>Password</label>
                  <div className={styles.inputWrapper}>
                    <i className="fas fa-lock"></i>
                    <input type={showPassword ? "text" : "password"} name="password" value={formData.password} onChange={handleChange} placeholder="Secure password" />
                    <button type="button" onClick={() => setShowPassword(!showPassword)}><i className={`fas ${showPassword ? 'fa-eye-slash' : 'fa-eye'}`}></i></button>
                  </div>
                  <div className={styles.strengthMeter}>
                    <div className={styles.strengthBar} style={{ width: `${(strength.score + 1) * 20}%`, backgroundColor: strength.color }}></div>
                    <span style={{ color: strength.color }}>Level: {strength.label}</span>
                  </div>
                </div>
                <div className={styles.inputGroup}>
                  <label>Confirm Password</label>
                  <div className={styles.inputWrapper}>
                    <i className="fas fa-check-double"></i>
                    <input type={showConfirmPassword ? "text" : "password"} name="confirmPassword" value={formData.confirmPassword} onChange={handleChange} placeholder="Repeat password" />
                    <button
                      type="button"
                      className={styles.passwordToggle}
                      onClick={() => setShowConfirmPassword(!showConfirmPassword)}
                    >
                      <i className={`fas ${showConfirmPassword ? 'fa-eye-slash' : 'fa-eye'}`}></i>
                    </button>
                  </div>
                  {passwordsMatch !== null && (
                    <span className={passwordsMatch ? styles.match : styles.mismatch}>
                      {passwordsMatch ? 'Passwords match' : 'Passwords do not match'}
                    </span>
                  )}
                </div>
                <div className={styles.actions}>
                  <button onClick={handleBack} className={styles.backBtn}>Back</button>
                  <button onClick={handleNext} className={styles.nextBtn}>Continue</button>
                </div>
              </div>
            )}

            {step === 2 && (
              <div className={styles.formSection}>
                <div className={styles.inputRow}>
                  <div className={styles.inputGroup}>
                    <label>First Name</label>
                    <input type="text" name="firstName" value={formData.firstName} onChange={handleChange} placeholder="John" />
                  </div>
                  <div className={styles.inputGroup}>
                    <label>Last Name</label>
                    <input type="text" name="lastName" value={formData.lastName} onChange={handleChange} placeholder="Doe" />
                  </div>
                </div>
                <div className={styles.inputRow}>
                  <div className={styles.inputGroup}>
                    <CustomSelect
                      label="Gender"
                      options={[{ value: 'male', label: 'Male' }, { value: 'female', label: 'Female' }, { value: 'other', label: 'Other' }]}
                      value={formData.gender}
                      onChange={(v) => setFormData(p => ({ ...p, gender: v }))}
                      placeholder="Select gender"
                    />
                  </div>
                  <div className={styles.inputGroup}>
                    <label>Phone Number</label>
                    <input type="tel" name="phoneNumber" value={formData.phoneNumber} onChange={handleChange} placeholder="+234..." />
                  </div>
                </div>
                <div className={styles.inputGroup}>
                   <CustomSelect
                      label="Country of Residence"
                      options={[
                        { value: 'Nigeria', label: 'Nigeria' },
                        { value: 'United States', label: 'United States' },
                        { value: 'United Kingdom', label: 'United Kingdom' },
                        { value: 'Canada', label: 'Canada' },
                        { value: 'Germany', label: 'Germany' },
                        { value: 'France', label: 'France' },
                        { value: 'China', label: 'China' },
                        { value: 'India', label: 'India' },
                        { value: 'South Africa', label: 'South Africa' },
                        { value: 'Ghana', label: 'Ghana' },
                        { value: 'Kenya', label: 'Kenya' },
                        { value: 'United Arab Emirates', label: 'United Arab Emirates' },
                        { value: 'Australia', label: 'Australia' },
                        { value: 'Other', label: 'Other' },
                      ]}
                      value={formData.country}
                      onChange={(v) => setFormData(p => ({ ...p, country: v }))}
                      placeholder="Select your country"
                    />
                </div>
                {formData.country === 'Nigeria' && (
                  <div className={styles.inputGroup} style={{ marginTop: '0.5rem' }}>
                    <label>Identity Number (NIN or BVN)</label>
                    <div className={styles.inputWrapper}>
                      <i className="fas fa-id-card"></i>
                      <input 
                        type="password" 
                        name="nin_bvn" 
                        value={formData.nin_bvn} 
                        onChange={handleChange} 
                        placeholder="11-digit identification number" 
                        maxLength={11}
                      />
                    </div>
                    <p className="text-[10px] text-gray-500 mt-1 italic">* Encrypted and hashed for your security.</p>
                  </div>
                )}
                <div className={styles.inputGroup}>
                    <CustomSelect
                      label="Company Designation"
                      options={[
                        { value: 'founder', label: 'Founder / CEO' },
                        { value: 'director', label: 'Director' },
                        { value: 'cto', label: 'Chief Technology Officer (CTO)' },
                        { value: 'cfo', label: 'Chief Financial Officer (CFO)' },
                        { value: 'manager', label: 'Operations Manager' },
                        { value: 'product', label: 'Product Manager' },
                        { value: 'developer', label: 'Lead Developer' },
                        { value: 'legal', label: 'Legal Counsel' },
                        { value: 'compliance', label: 'Compliance Officer' },
                        { value: 'marketing', label: 'Marketing Director' },
                        { value: 'sales', label: 'Sales Lead' },
                        { value: 'agent', label: 'Authorized Agent' },
                        { value: 'other', label: 'Other...' },
                      ]}
                      value={formData.applicantRole}
                      onChange={(v) => {
                        setFormData(p => ({ ...p, applicantRole: v }));
                        if (v !== 'other') {
                          setFormData(p => ({ ...p, customRole: '' }));
                        }
                      }}
                      placeholder="Select your role..."
                    />
                  </div>
                  {formData.applicantRole === 'other' && (
                    <div className={styles.inputGroup} style={{ marginTop: '-0.5rem' }}>
                      <label>Specify Role</label>
                      <input 
                        type="text" 
                        name="customRole" 
                        value={(formData as any).customRole || ''} 
                        onChange={handleChange} 
                        placeholder="e.g. Compliance Officer" 
                      />
                    </div>
                  )}
                <div className={styles.checkboxGroup}>
                  <label className={styles.checkboxContainer}>
                    <input type="checkbox" name="agreeToTerms" checked={formData.agreeToTerms} onChange={handleChange} />
                    <span className={styles.checkmark}></span>
                    I agree to the <Link to="/terms">Terms</Link> and <Link to="/privacy">Privacy Policies</Link>
                  </label>
                </div>
                <div className={styles.actions}>
                  <button onClick={handleBack} className={styles.backBtn}>Back</button>
                  <button onClick={handleNext} className={styles.nextBtn}>Connect Business</button>
                </div>
              </div>
            )}

            {step === 3 && (
              <div className={styles.formSection}>
                <div className={styles.inputGroup}>
                  <label>Business Legal Name</label>
                  <input type="text" name="businessName" value={formData.businessName} onChange={handleChange} placeholder="Acme Corp Ltd" />
                </div>
                <div className={styles.inputRow}>
                  <div className={styles.inputGroup}>
                    <label>Registration Country</label>
                    <input type="text" name="businessCountry" value={formData.businessCountry} onChange={handleChange} placeholder="United Kingdom" />
                  </div>
                  <div className={styles.inputGroup}>
                    <label>Registration Number (RC)</label>
                    <input type="text" name="businessLicenseNumber" value={formData.businessLicenseNumber} onChange={handleChange} placeholder="RC-123456" />
                  </div>
                </div>
                <div className={styles.inputGroup}>
                  <label>Business Website</label>
                  <input type="url" name="website" value={formData.website} onChange={handleChange} placeholder="https://yourcompany.com" />
                </div>
                <div className={styles.inputGroup}>
                  <label>Certificate Link (Optional)</label>
                  <input type="text" name="businessCertificateUrl" value={formData.businessCertificateUrl} onChange={handleChange} placeholder="https://drive.google.com/..." />
                </div>
                <div className={styles.inputRow}>
                   <div className={styles.inputGroup}>
                    <label>Twitter (Optional)</label>
                    <div className={styles.inputWrapper}>
                      <i className="fab fa-twitter"></i>
                      <input type="text" name="twitter_handle" value={formData.twitter_handle} onChange={handleChange} placeholder="@business" />
                    </div>
                  </div>
                  <div className={styles.inputGroup}>
                    <label>Instagram (Optional)</label>
                    <div className={styles.inputWrapper}>
                      <i className="fab fa-instagram"></i>
                      <input type="text" name="instagram_handle" value={formData.instagram_handle} onChange={handleChange} placeholder="@business" />
                    </div>
                  </div>
                </div>
                <div className={styles.actions}>
                  <button onClick={handleBack} className={styles.backBtn}>Back</button>
                  <button onClick={() => handleSubmit()} className={styles.nextBtn}>Launch Gateway</button>
                </div>
              </div>
            )}
          </div>

          <div className={styles.footer}>
            <p>Already have an account? <Link to="/login">Sign In</Link></p>
          </div>
        </div>
      </div>
    </div>
  )
}

export default RegisterPage
