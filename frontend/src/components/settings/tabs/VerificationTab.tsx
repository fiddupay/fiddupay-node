import React, { useState } from 'react'
import { MdSecurity, MdBusiness, MdShield, MdCheckCircle, MdArrowForward, MdRocketLaunch, MdPerson } from 'react-icons/md'
import { FaTwitter, FaInstagram, FaLinkedin, FaGlobe } from 'react-icons/fa'
import { merchantAPI } from '@/services/apiService'
import { useToast } from '@/contexts/ToastContext'
import { useAuthStore } from '@/stores/authStore'
import { Badge } from '@/components/ui/badge'

interface VerificationTabProps {
    user: any;
    loading: boolean;
    styles: any;
}

const VerificationTab: React.FC<VerificationTabProps> = ({ user, loading: _parentLoading, styles }) => {
    const { showToast } = useToast()
    const { loadUser } = useAuthStore()
    const [loading, setLoading] = useState(false)
    const [step, setStep] = useState(user?.kyc_tier > 0 ? 2 : 1)
    
    const [idData, setIdData] = useState({
        nin_bvn: '',
        username: user?.username || '',
    })

    const [socialData, setSocialData] = useState({
        twitter: user?.social_handles?.twitter || '',
        instagram: user?.social_handles?.instagram || '',
        linkedin: user?.social_handles?.linkedin || '',
        website: user?.social_handles?.website || '',
        business_license: user?.business_license_number || '',
    })

    const handleClaimUsername = async () => {
        if (!idData.username) return;
        setLoading(true);
        try {
            await merchantAPI.claimUsername(idData.username);
            showToast('Username claimed successfully!', 'success');
            await loadUser(true);
        } catch (error: any) {
            showToast(error.response?.data?.error || 'Failed to claim username', 'error');
        } finally {
            setLoading(false);
        }
    };

    const handleSubmitTier1 = async () => {
        if (!idData.nin_bvn) {
            showToast('Please enter your NIN or BVN', 'warning');
            return;
        }
        setLoading(true);
        try {
            await merchantAPI.updateKycDraft({
                nin_bvn: idData.nin_bvn,
            });
            showToast('Identity verification submitted', 'success');
            await loadUser(true);
            setStep(2);
        } catch (error: any) {
            showToast(error.response?.data?.error || 'Failed to submit identity', 'error');
        } finally {
            setLoading(false);
        }
    };

    const handleSubmitTier2 = async () => {
        setLoading(true);
        try {
            await merchantAPI.updateKycDraft({
                social_handles: {
                    twitter: socialData.twitter,
                    instagram: socialData.instagram,
                    linkedin: socialData.linkedin,
                    website: socialData.website
                },
                business_license_number: socialData.business_license || undefined
            });
            showToast('Profile intelligence updated!', 'success');
            await loadUser(true);
        } catch (error: any) {
            showToast(error.response?.data?.error || 'Failed to update socials', 'error');
        } finally {
            setLoading(false);
        }
    };

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
                        <h2 style={{ margin: 0, fontSize: '1.25rem', color: 'var(--text-main)' }}>Trust Intelligence Layer</h2>
                        <div className="flex items-center gap-2 mt-1">
                            <span style={{ fontSize: '0.9rem', color: 'var(--text-muted)' }}>
                                Self-Governing Level:
                            </span>
                            <Badge className={`${user?.kyc_tier >= 2 ? 'bg-green-500/20 text-green-500' : 'bg-primary/20 text-primary'} border-none px-2 py-0.5`}>
                                Tier {user?.kyc_tier || 0}
                            </Badge>
                        </div>
                    </div>
                </div>
            </div>

            {/* Stepper Indicator */}
            <div className="flex items-center gap-4 mb-8 bg-white/5 p-4 rounded-xl border border-white/10">
                <div className={`flex items-center gap-2 ${step >= 1 ? 'text-primary' : 'text-gray-500'}`}>
                    <div className={`w-8 h-8 rounded-full flex items-center justify-center font-bold ${step === 1 ? 'bg-primary text-white' : (step > 1 ? 'bg-green-500 text-white' : 'bg-gray-700')}`}>
                        {step > 1 ? <MdCheckCircle size={20} /> : '1'}
                    </div>
                    <span className="text-sm font-bold">Identity & PayID</span>
                </div>
                <div className="flex-1 h-px bg-white/10" />
                <div className={`flex items-center gap-2 ${step >= 2 ? 'text-primary' : 'text-gray-500'}`}>
                    <div className={`w-8 h-8 rounded-full flex items-center justify-center font-bold ${step === 2 ? 'bg-primary text-white' : (step > 2 ? 'bg-green-500 text-white' : 'bg-gray-700')}`}>
                         {step > 2 ? <MdCheckCircle size={20} /> : '2'}
                    </div>
                    <span className="text-sm font-bold">Social Signals</span>
                </div>
            </div>

            {step === 1 && (
                <div className="space-y-6 animate-in fade-in slide-in-from-bottom-4 duration-500">
                    <div className="bg-primary/5 border border-primary/20 p-4 rounded-xl flex gap-4">
                        <MdRocketLaunch className="text-primary shrink-0 mt-1" size={24} />
                        <div>
                            <h4 className="text-white font-bold mb-1">Claim your unique Identity</h4>
                            <p className="text-sm text-gray-400">Claim your username to enable 0-fee payments from other FidduPay users and get your unique PayID.</p>
                        </div>
                    </div>

                    <div className={styles.inputGroup}>
                        <label>FidduPay Username</label>
                        <div className="flex gap-2">
                             <div className={styles.inputWrapper} style={{ flex: 1 }}>
                                <MdPerson className={styles.inputIcon} />
                                <input 
                                    type="text" 
                                    placeholder="e.g. techy_store"
                                    value={idData.username}
                                    onChange={(e) => setIdData({...idData, username: e.target.value.toLowerCase().replace(/[^a-z0-9_]/g, '')})}
                                    className={styles.urlInput}
                                    disabled={!!user?.username}
                                />
                            </div>
                            {!user?.username && (
                                <button 
                                    onClick={handleClaimUsername}
                                    disabled={loading || !idData.username}
                                    className="bg-primary hover:bg-primary-hover text-white px-6 rounded-xl font-bold transition-all disabled:opacity-50"
                                >
                                    Claim
                                </button>
                            )}
                        </div>
                        {user?.username && <p className="text-xs text-green-500 mt-1 flex items-center gap-1"><MdCheckCircle /> Verified: @{user.username}</p>}
                    </div>

                    <div className={styles.inputGroup}>
                        <label>Identity Number (NIN or BVN)</label>
                        <div className={styles.inputWrapper}>
                            <MdSecurity className={styles.inputIcon} />
                            <input 
                                type="password" 
                                placeholder="11-digit number (Hashed for safety)"
                                value={idData.nin_bvn}
                                onChange={(e) => setIdData({...idData, nin_bvn: e.target.value})}
                                className={styles.urlInput}
                                maxLength={11}
                            />
                        </div>
                        <p className="text-[10px] text-gray-500 mt-1 italic">* We never store raw ID numbers. They are immediately hashed into a one-way cryptographic signal.</p>
                    </div>

                    <button 
                        onClick={handleSubmitTier1}
                        disabled={loading || !idData.nin_bvn}
                        className="w-full bg-gradient-to-r from-primary to-primary-hover text-white py-4 rounded-xl font-bold flex items-center justify-center gap-2 shadow-lg shadow-primary/20 hover:scale-[1.02] active:scale-[0.98] transition-all"
                    >
                        {loading ? 'Processing...' : (
                            <>Verify Identity & Proceed <MdArrowForward /></>
                        )}
                    </button>
                </div>
            )}

            {step === 2 && (
                <div className="space-y-6 animate-in fade-in slide-in-from-right-4 duration-500">
                    <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                        <div className={styles.inputGroup}>
                            <label>Twitter Handle</label>
                            <div className={styles.inputWrapper}>
                                <FaTwitter className={styles.inputIcon} />
                                <input 
                                    type="text" 
                                    placeholder="@username"
                                    value={socialData.twitter}
                                    onChange={(e) => setSocialData({...socialData, twitter: e.target.value})}
                                    className={styles.urlInput}
                                />
                            </div>
                        </div>

                        <div className={styles.inputGroup}>
                            <label>LinkedIn Profile</label>
                            <div className={styles.inputWrapper}>
                                <FaLinkedin className={styles.inputIcon} />
                                <input 
                                    type="text" 
                                    placeholder="linkedin.com/in/..."
                                    value={socialData.linkedin}
                                    onChange={(e) => setSocialData({...socialData, linkedin: e.target.value})}
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
                                    value={socialData.instagram}
                                    onChange={(e) => setSocialData({...socialData, instagram: e.target.value})}
                                    className={styles.urlInput}
                                />
                            </div>
                        </div>

                        <div className={styles.inputGroup}>
                            <label>Official Website</label>
                            <div className={styles.inputWrapper}>
                                <FaGlobe className={styles.inputIcon} />
                                <input 
                                    type="text" 
                                    placeholder="https://..."
                                    value={socialData.website}
                                    onChange={(e) => setSocialData({...socialData, website: e.target.value})}
                                    className={styles.urlInput}
                                />
                            </div>
                        </div>
                    </div>

                    <div className={styles.inputGroup}>
                        <label>Business Registration (CAC Number)</label>
                        <div className={styles.inputWrapper}>
                            <MdBusiness className={styles.inputIcon} />
                            <input 
                                type="text" 
                                placeholder="RC-1234567"
                                value={socialData.business_license}
                                onChange={(e) => setSocialData({...socialData, business_license: e.target.value})}
                                className={styles.urlInput}
                            />
                        </div>
                    </div>

                    <div className="flex gap-4">
                        <button 
                            onClick={() => setStep(1)}
                            className="px-6 py-4 rounded-xl border border-white/10 text-gray-400 hover:text-white transition-all font-bold"
                        >
                            Back
                        </button>
                        <button 
                            onClick={handleSubmitTier2}
                            disabled={loading}
                            className="flex-1 bg-gradient-to-r from-primary to-primary-hover text-white py-4 rounded-xl font-bold shadow-lg shadow-primary/20 hover:scale-[1.02] active:scale-[0.98] transition-all"
                        >
                            {loading ? 'Saving...' : 'Reach Gold Trust Status'}
                        </button>
                    </div>
                </div>
            )}
        </div>
    )
}

export default VerificationTab
