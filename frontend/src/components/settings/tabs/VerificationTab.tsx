import { useState, useEffect } from 'react'
import { useNavigate } from 'react-router-dom'
import { 
    MdSecurity, 
    MdBusiness, 
    MdShield, 
    MdCheckCircle, 
    MdArrowForward, 
    MdRocketLaunch, 
    MdPerson, 
    MdWarning,
    MdLockOpen,
    MdAutoGraph,
    MdOutlineFingerprint
} from 'react-icons/md'
import { FaTwitter, FaLinkedin, FaGlobe } from 'react-icons/fa'
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
    const navigate = useNavigate()
    const [loading, setLoading] = useState(false)
    const [step, setStep] = useState(user?.kyc_tier > 1 ? 3 : (user?.kyc_tier > 0 ? 2 : 1))
    
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

    useEffect(() => {
        if (user) {
            setIdData(prev => ({
                ...prev,
                username: user.username || prev.username
            }));
            setSocialData({
                twitter: user.social_handles?.twitter || '',
                instagram: user.social_handles?.instagram || '',
                linkedin: user.social_handles?.linkedin || '',
                website: user.social_handles?.website || '',
                business_license: user.business_license_number || '',
            });
            // Auto-advance step if tier increased
            if (user.kyc_tier > 1) setStep(3);
            else if (user.has_national_id || user.kyc_tier > 0) setStep(2);
        }
    }, [user]);

    const handleStartSmileID = () => {
        showToast('Initializing SmileID Secure Verification...', 'info');
        // Logic to trigger SmileID SDK would go here
    };

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
        if (!socialData.website) {
            showToast('Official Website is mandatory for your business profile', 'warning');
            return;
        }
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
            showToast('Profile intelligence updated! You have reached Gold Trust status.', 'success');
            await loadUser(true);
            // Navigate after a short delay to allow the user to see the success message
            setTimeout(() => {
                navigate('/app/dashboard');
            }, 2000);
        } catch (error: any) {
            showToast(error.response?.data?.error || 'Failed to update socials', 'error');
        } finally {
            setLoading(false);
        }
    };

    const getTierLabel = (tier: number) => {
        switch(tier) {
            case 0: return 'Untrusted (Sandbox)';
            case 1: return 'Verified Identity';
            case 2: return 'Gold Intelligence';
            case 3: return 'Institutional Elite';
            default: return 'Institutional';
        }
    }

    return (
        <div className={styles.verificationContainer}>
            {/* Sandbox Restriction Banner */}
            {user?.kyc_tier === 0 && (
                <div className={styles.verificationBanner}>
                    <div className={styles.bannerPulse} />
                    <div className={styles.bannerIcon}>
                        <MdWarning size={28} />
                    </div>
                    <div className={styles.bannerText}>
                        <h4>Sandbox-First Mode Active</h4>
                        <p>
                            Your account is restricted to the <strong>Sandbox Environment</strong>. 
                            Complete identity verification to unlock <strong style={{color: '#4ade80'}}>Live Settlements</strong> and higher volume limits.
                        </p>
                    </div>
                </div>
            )}

            {/* Trust Intelligence Header */}
            <div className={styles.trustHeader}>
                <div className={styles.trustInfo}>
                    <div className={styles.trustIcon}>
                        <MdShield size={36} />
                    </div>
                    <div className={styles.trustTitle}>
                        <h2>Trust Intelligence Layer</h2>
                        <div className={styles.tierBadgeWrap}>
                            <span className={styles.fieldLabel}>Self-Governing Level:</span>
                            <Badge className={`${user?.kyc_tier >= 2 ? 'bg-green-500/20 text-green-400' : 'bg-primary/20 text-primary'} border-none px-3 py-1 font-bold`}>
                                {getTierLabel(user?.kyc_tier || 0)}
                            </Badge>
                        </div>
                    </div>
                </div>

                <div className={styles.growthPath}>
                    <MdAutoGraph className="text-primary" />
                    <span className={styles.fieldLabel}>Growth Path</span>
                    <div className={styles.growthDots}>
                        {[0, 1, 2, 3].map((t) => (
                            <div 
                                key={t} 
                                className={`${styles.dot} ${user?.kyc_tier >= t ? styles.dotActive : ''}`} 
                            />
                        ))}
                    </div>
                </div>
            </div>

            {/* Main Multi-Step Form */}
            <div className={styles.stepLayout}>
                {/* Sidebar Progress */}
                <div className={styles.stepSidebar}>
                    {[
                        { id: 1, label: 'Identity & PayID', icon: <MdOutlineFingerprint />, desc: 'Claim your on-chain ID' },
                        { id: 2, label: 'Social Signals', icon: <MdRocketLaunch />, desc: 'Build your trust score' },
                        { id: 3, label: 'Biometric KYC', icon: <MdSecurity />, desc: 'SmileID Face & Doc Scan' }
                    ].map((s) => (
                        <button 
                            key={s.id}
                            onClick={() => setStep(s.id)}
                            className={`${styles.stepBtn} ${step === s.id ? styles.stepBtnActive : ''}`}
                        >
                            <div className={styles.stepBtnHeader}>
                                <span className={styles.stepIcon}>{s.icon}</span>
                                <span className={styles.stepLabel}>{s.label}</span>
                            </div>
                            <p className={styles.stepDesc}>{s.desc}</p>
                        </button>
                    ))}
                </div>
                {/* Step Content */}
                <div className={styles.stepContent}>
                    {step === 1 && (
                        <div className={styles.formCard}>
                            <div className={styles.formHeader}>
                                <h3>
                                    <MdLockOpen style={{marginRight: '8px', color: 'var(--primary)'}} />
                                    Identity Verification
                                </h3>
                                <p>Verifying your identity unlocks PayID and enables zero-fee interoperability within the FidduPay ecosystem.</p>
                            </div>
                            
                            {user?.has_national_id || user?.kyc_tier >= 1 ? (
                                <div style={{ textAlign: 'center', padding: '40px 20px' }}>
                                    <div style={{ width: '80px', height: '80px', background: 'rgba(34, 197, 94, 0.1)', borderRadius: '50%', display: 'flex', alignItems: 'center', justifyContent: 'center', margin: '0 auto 20px' }}>
                                        <MdCheckCircle size={48} color="#22c55e" />
                                    </div>
                                    <h4 style={{ fontSize: '20px', fontWeight: '800', color: 'var(--text-main)', marginBottom: '8px' }}>Identity Secured</h4>
                                    <p style={{ color: 'var(--text-muted)', marginBottom: '24px' }}>Your national identity has been cryptographicly verified.</p>
                                    
                                    <div style={{ background: 'var(--surface-hover)', padding: '16px', borderRadius: '12px', border: '1px solid var(--border)', display: 'inline-flex', alignItems: 'center', gap: '12px' }}>
                                        <MdPerson color="var(--primary)" />
                                        <span style={{ fontWeight: 'bold' }}>@{user.username}</span>
                                        <Badge className="bg-green-500/20 text-green-400 border-none">Verified</Badge>
                                    </div>

                                    <div style={{ marginTop: '32px' }}>
                                        <button onClick={() => setStep(2)} className={styles.submitBtn} style={{ maxWidth: '240px', margin: '0 auto' }}>
                                            Continue to Socials <MdArrowForward />
                                        </button>
                                    </div>
                                </div>
                            ) : (
                                <div className="space-y-8" style={{display: 'flex', flexDirection: 'column', gap: '32px'}}>
                                    {/* Username Claim */}
                                    <div className={styles.fieldGroup}>
                                        <label className={styles.fieldLabel}>Claim your @username</label>
                                        <div style={{display: 'flex', gap: '12px'}}>
                                            <div className={styles.inputContainer} style={{flex: 1}}>
                                                <div className={styles.inputIcon}>
                                                    <MdPerson size={20} />
                                                </div>
                                                <input 
                                                    type="text" 
                                                    placeholder="e.g. techy_store"
                                                    value={idData.username}
                                                    onChange={(e) => setIdData({...idData, username: e.target.value.toLowerCase().replace(/[^a-z0-9_]/g, '')})}
                                                    className={styles.inputField}
                                                    disabled={!!user?.username}
                                                />
                                            </div>
                                            <button 
                                                onClick={user?.username ? undefined : handleClaimUsername}
                                                disabled={loading || (!user?.username && !idData.username)}
                                                className={styles.saveBtn}
                                                style={{
                                                    borderRadius: '12px', 
                                                    minWidth: '80px',
                                                    background: user?.username ? '#22c55e' : 'var(--primary)',
                                                    cursor: user?.username ? 'default' : 'pointer'
                                                }}
                                            >
                                                {loading ? '...' : user?.username ? <MdCheckCircle size={20} /> : 'Claim'}
                                            </button>
                                        </div>
                                        {user?.has_national_id && (
                                            <div style={{display: 'flex', alignItems: 'center', gap: '8px', color: '#10b981', fontSize: '12px', fontWeight: 'bold', background: 'rgba(16,185,129,0.1)', padding: '8px 12px', borderRadius: '8px', width: 'fit-content'}}>
                                                <MdCheckCircle /> Identity Linked & Cryptographically Hashed
                                            </div>
                                        )}
                                    </div>

                                    {/* ID Number */}
                                    <div className={styles.fieldGroup}>
                                        <label className={styles.fieldLabel}>National Identity (NIN or BVN)</label>
                                        <div className={styles.inputContainer}>
                                            <div className={styles.inputIcon}>
                                                <MdSecurity size={20} />
                                            </div>
                                            <input 
                                                type="password" 
                                                placeholder="Enter 11-digit number"
                                                value={idData.nin_bvn}
                                                onChange={(e) => setIdData({...idData, nin_bvn: e.target.value})}
                                                className={styles.inputField}
                                                maxLength={11}
                                            />
                                        </div>
                                        <div className={styles.privacyNote}>
                                            <MdShield style={{color: '#3b82f6', flexShrink: 0}} />
                                            <p>
                                                FidduPay never stores raw ID numbers. They are immediately hashed into a one-way cryptographic signal for the Trust Intelligence Layer.
                                            </p>
                                        </div>
                                    </div>

                                    <button 
                                        onClick={handleSubmitTier1}
                                        disabled={loading || !idData.nin_bvn}
                                        className={styles.submitBtn}
                                    >
                                        {loading ? 'Processing Protocol...' : (
                                            <>Finalize Tier 1 Verification <MdArrowForward size={20} /></>
                                        )}
                                    </button>
                                </div>
                            )}
                        </div>
                    )}

                    {step === 2 && (
                        <div className={styles.formCard}>
                            <div className={styles.formHeader}>
                                <h3>
                                    <MdRocketLaunch style={{marginRight: '8px', color: 'var(--secondary)'}} />
                                    Social Trust Signals
                                </h3>
                                <p>Enhance your Merchant Trust Score by linking your business’s digital footprint.</p>
                            </div>
                            
                            <div className="space-y-6" style={{display: 'flex', flexDirection: 'column', gap: '24px'}}>
                                <div className={styles.inputGrid}>
                                    <div className={styles.fieldGroup}>
                                        <label className={styles.fieldLabel}>Twitter (X)</label>
                                        <div className={styles.inputContainer}>
                                            <FaTwitter className={styles.inputIcon} />
                                            <input 
                                                type="text" placeholder="@handle" value={socialData.twitter}
                                                onChange={(e) => setSocialData({...socialData, twitter: e.target.value})}
                                                className={styles.inputField}
                                            />
                                        </div>
                                    </div>

                                    <div className={styles.fieldGroup}>
                                        <label className={styles.fieldLabel}>LinkedIn</label>
                                        <div className={styles.inputContainer}>
                                            <FaLinkedin className={styles.inputIcon} />
                                            <input 
                                                type="text" placeholder="company/profile" value={socialData.linkedin}
                                                onChange={(e) => setSocialData({...socialData, linkedin: e.target.value})}
                                                className={styles.inputField}
                                            />
                                        </div>
                                    </div>

                                    <div className={styles.fieldGroup}>
                                        <label className={styles.fieldLabel}>Official Website</label>
                                        <div className={styles.inputContainer}>
                                            <FaGlobe className={styles.inputIcon} />
                                            <input 
                                                type="text" placeholder="https://..." value={socialData.website}
                                                onChange={(e) => setSocialData({...socialData, website: e.target.value})}
                                                className={styles.inputField}
                                            />
                                        </div>
                                    </div>

                                    <div className={styles.fieldGroup}>
                                        <label className={styles.fieldLabel}>Business License</label>
                                        <div className={`${styles.inputContainer} ${user?.business_license_update_count >= 2 ? styles.inputDisabled : ''}`}>
                                            <MdBusiness className={styles.inputIcon} />
                                            <input 
                                                type="text" 
                                                placeholder="CAC / RC Number" 
                                                value={socialData.business_license}
                                                onChange={(e) => setSocialData({...socialData, business_license: e.target.value})}
                                                className={styles.inputField}
                                                disabled={user?.business_license_update_count >= 2}
                                            />
                                        </div>
                                        {user?.business_license_update_count >= 2 && (
                                            <div style={{ marginTop: '8px', display: 'flex', alignItems: 'center', gap: '8px', color: '#f87171', fontSize: '12px', fontWeight: 'bold', background: 'rgba(239, 68, 68, 0.1)', padding: '8px 12px', borderRadius: '8px' }}>
                                                <MdWarning />
                                                <span>Limit reached. Contact support to update.</span>
                                            </div>
                                        )}
                                    </div>
                                </div>

                                <div style={{display: 'flex', gap: '16px', paddingTop: '16px'}}>
                                    <button 
                                        onClick={() => setStep(1)}
                                        className={styles.viewBtn}
                                        style={{padding: '12px 24px'}}
                                    >
                                        Back
                                    </button>
                                    <button 
                                        onClick={handleSubmitTier2}
                                        disabled={loading}
                                        className={styles.submitBtn}
                                        style={{background: 'linear-gradient(135deg, var(--secondary), #d97706)', boxShadow: '0 10px 20px -5px rgba(245, 158, 11, 0.4)'}}
                                    >
                                        {loading ? 'Optimizing Signals...' : 'Reach Gold Trust Status'}
                                    </button>
                                </div>
                            </div>
                        </div>
                    )}
                    {step === 3 && (
                        <div className={styles.formCard} style={{ textAlign: 'center', padding: '60px 40px' }}>
                            <div className={styles.formHeader}>
                                <img 
                                    src="https://smileidentity.com/wp-content/uploads/2021/01/Smile-Identity-Logo-Dark.png" 
                                    alt="SmileID" 
                                    style={{ height: '32px', marginBottom: '24px', opacity: 0.8, filter: 'brightness(2)' }} 
                                />
                                <h3>Institutional Biometric Verification</h3>
                                <p>Perform a high-security face and document scan to unlock the Institutional Elite status and limitless transaction volumes.</p>
                            </div>

                            <div style={{ margin: '40px 0', padding: '32px', background: 'rgba(99, 102, 241, 0.05)', borderRadius: '24px', border: '1px dashed var(--border)' }}>
                                <MdSecurity size={64} style={{ color: 'var(--primary)', marginBottom: '20px', opacity: 0.5 }} />
                                <div style={{ fontSize: '13px', color: 'var(--text-muted)', maxWidth: '400px', margin: '0 auto', lineHeight: '1.6' }}>
                                    This process uses **SmileID** to verify your live face against government databases. Please ensure you are in a well-lit area.
                                </div>
                            </div>

                            <button 
                                onClick={handleStartSmileID}
                                className={styles.submitBtn}
                                style={{ maxWidth: '300px', margin: '0 auto' }}
                            >
                                Start SmileID Verification
                            </button>

                            <div className={styles.privacyNote} style={{ marginTop: '32px', justifyContent: 'center' }}>
                                <MdShield style={{ color: '#3b82f6' }} />
                                <p>Bank-grade security. Your biometric data is encrypted and never stored by FidduPay.</p>
                            </div>
                        </div>
                    )}
                </div>
            </div>
        </div>
    )
}

export default VerificationTab
