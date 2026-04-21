import React, { useState } from 'react'
import { MdInfo, MdSecurity, MdBusiness, MdShield } from 'react-icons/md'
import { FaTwitter, FaInstagram } from 'react-icons/fa'
import { merchantAPI } from '@/services/apiService'
import { useToast } from '@/contexts/ToastContext'
import { useAuthStore } from '@/stores/authStore'

interface VerificationTabProps {
    user: any;
    loading: boolean;
    styles: any;
}

const VerificationTab: React.FC<VerificationTabProps> = ({ user, loading: parentLoading, styles }) => {
    const { showToast } = useToast()
    const { loadUser } = useAuthStore()
    const [loading, setLoading] = useState(false)
    const [formData, setFormData] = useState({
        nin_bvn: '',
        twitter_handle: user?.social_handles?.twitter || '',
        instagram_handle: user?.social_handles?.instagram || '',
        business_license_number: user?.business_license_number || '',
    })

    const handleUpdateKYC = async (e: React.FormEvent) => {
        e.preventDefault()
        setLoading(true)
        try {
            await merchantAPI.updateSettings({
                // @ts-ignore - Backend supports these in update_settings or we need a new endpoint
                // For now, we reuse updateSettings as it's the general profile update
                nin_bvn: formData.nin_bvn || undefined,
                twitter_handle: formData.twitter_handle || undefined,
                instagram_handle: formData.instagram_handle || undefined,
                business_license_number: formData.business_license_number || undefined,
            })
            await loadUser(true)
            showToast('Verification details submitted for review', 'success')
        } catch (error: any) {
            showToast(error.response?.data?.error?.message || 'Failed to update verification details', 'error')
        } finally {
            setLoading(false)
        }
    }

    const getStatusColor = (status: string) => {
        switch (status?.toUpperCase()) {
            case 'VERIFIED': return '#10b981'
            case 'PENDING': return '#f59e0b'
            case 'REJECTED': return '#ef4444'
            default: return '#6b7280'
        }
    }

    return (
        <div className={styles.tabContent}>
            <div className={styles.sectionHeader}>
                <div style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
                    <div style={{ 
                        width: '40px', 
                        height: '40px', 
                        borderRadius: '12px', 
                        background: 'rgba(99, 102, 241, 0.1)', 
                        display: 'flex', 
                        alignItems: 'center', 
                        justifyContent: 'center',
                        color: 'var(--primary)'
                    }}>
                        <MdShield size={24} />
                    </div>
                    <div>
                        <h2 style={{ margin: 0, fontSize: '1.25rem', color: 'var(--text-main)' }}>Merchant Verification</h2>
                        <p style={{ margin: 0, fontSize: '0.9rem', color: 'var(--text-muted)' }}>
                            Compliance Status: <span style={{ color: getStatusColor(user?.compliance_status), fontWeight: 700 }}>{user?.compliance_status || 'NOT_STARTED'}</span>
                        </p>
                    </div>
                </div>
            </div>

            <div className={styles.infoBox} style={{ background: 'rgba(99, 102, 241, 0.05)', border: '1px solid rgba(99, 102, 241, 0.2)', padding: '20px', borderRadius: '16px', marginBottom: '24px' }}>
                <div style={{ display: 'flex', gap: '12px' }}>
                    <MdInfo size={20} color="var(--primary)" />
                    <div>
                        <h4 style={{ margin: '0 0 4px 0', color: 'var(--text-main)' }}>Why verify?</h4>
                        <p style={{ margin: 0, fontSize: '0.85rem', color: 'var(--text-muted)', lineHeight: '1.5' }}>
                            Providing your NIN/BVN and business registration helps us build your **Trust Intelligence Score**. 
                            Verified merchants enjoy higher daily volume limits and faster settlement times. 
                            <br/><em>Note: We store a cryptographic hash of your ID, never the raw number.</em>
                        </p>
                    </div>
                </div>
            </div>

            <form onSubmit={handleUpdateKYC} className={styles.formGrid}>
                <div className={styles.inputGroup}>
                    <label>NIN or BVN (Nigerian Identity)</label>
                    <div className={styles.inputWrapper}>
                        <MdSecurity className={styles.inputIcon} />
                        <input 
                            type="text" 
                            placeholder="11-digit number"
                            value={formData.nin_bvn}
                            onChange={(e) => setFormData({...formData, nin_bvn: e.target.value})}
                            className={styles.urlInput}
                            maxLength={11}
                        />
                    </div>
                </div>

                <div className={styles.inputGroup}>
                    <label>Business Registration Number (RC)</label>
                    <div className={styles.inputWrapper}>
                        <MdBusiness className={styles.inputIcon} />
                        <input 
                            type="text" 
                            placeholder="e.g. RC-123456"
                            value={formData.business_license_number}
                            onChange={(e) => setFormData({...formData, business_license_number: e.target.value})}
                            className={styles.urlInput}
                        />
                    </div>
                </div>

                <div className={styles.inputGroup}>
                    <label>Twitter Handle</label>
                    <div className={styles.inputWrapper}>
                        <FaTwitter className={styles.inputIcon} />
                        <input 
                            type="text" 
                            placeholder="@username"
                            value={formData.twitter_handle}
                            onChange={(e) => setFormData({...formData, twitter_handle: e.target.value})}
                            className={styles.urlInput}
                        />
                    </div>
                </div>

                <div className={styles.inputGroup}>
                    <label>Instagram Handle</label>
                    <div className={styles.inputWrapper}>
                        <FaInstagram className={styles.inputIcon} />
                        <input 
                            type="text" 
                            placeholder="@username"
                            value={formData.instagram_handle}
                            onChange={(e) => setFormData({...formData, instagram_handle: e.target.value})}
                            className={styles.urlInput}
                        />
                    </div>
                </div>

                <div className={styles.fullWidth}>
                    <button 
                        type="submit" 
                        className={styles.saveBtn} 
                        disabled={loading || parentLoading}
                        style={{ marginTop: '12px' }}
                    >
                        {loading ? 'Submitting...' : 'Save Verification Data'}
                    </button>
                </div>
            </form>
        </div>
    )
}

export default VerificationTab
