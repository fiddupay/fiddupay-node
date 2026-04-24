import React, { useState } from 'react'
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
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'

interface VerificationTabProps {
    user: any;
    loading: boolean;
    styles: any;
}

const VerificationTab: React.FC<VerificationTabProps> = ({ user, loading: _parentLoading, styles: _ }) => {
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
            showToast('Profile intelligence updated!', 'success');
            await loadUser(true);
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
            default: return 'Institutional';
        }
    }

    return (
        <div className="space-y-8 max-w-4xl animate-in fade-in duration-700">
            {/* Sandbox Restriction Banner */}
            {user?.kyc_tier === 0 && (
                <div className="relative overflow-hidden group">
                    <div className="absolute inset-0 bg-gradient-to-r from-orange-500/10 to-transparent animate-pulse" />
                    <div className="relative bg-orange-500/5 border border-orange-500/20 rounded-2xl p-6 flex gap-5 items-center">
                        <div className="w-12 h-12 rounded-full bg-orange-500/20 flex items-center justify-center text-orange-500 shrink-0 shadow-[0_0_15px_rgba(249,115,22,0.3)]">
                            <MdWarning size={28} />
                        </div>
                        <div className="flex-1">
                            <h4 className="text-orange-200 font-bold text-lg mb-1">Sandbox-First Mode Active</h4>
                            <p className="text-orange-200/60 text-sm leading-relaxed">
                                Your account is restricted to the <strong className="text-orange-400">Sandbox Environment</strong>. 
                                Complete identity verification to unlock <strong className="text-green-400">Live Settlements</strong> and higher volume limits.
                            </p>
                        </div>
                    </div>
                </div>
            )}

            {/* Trust Intelligence Header */}
            <div className="flex flex-col md:flex-row md:items-end justify-between gap-6 pb-6 border-b border-white/5">
                <div className="flex gap-4 items-center">
                    <div className="w-16 h-16 rounded-2xl bg-primary/10 border border-primary/20 flex items-center justify-center text-primary shadow-[0_0_30px_rgba(99,102,241,0.15)]">
                        <MdShield size={36} />
                    </div>
                    <div>
                        <h2 className="text-2xl font-black text-white tracking-tight">Trust Intelligence Layer</h2>
                        <div className="flex items-center gap-2 mt-1">
                            <span className="text-sm text-gray-400">Self-Governing Level:</span>
                            <Badge className={`${user?.kyc_tier >= 2 ? 'bg-green-500/20 text-green-400' : 'bg-primary/20 text-primary'} border-none px-3 py-1 font-bold`}>
                                {getTierLabel(user?.kyc_tier || 0)}
                            </Badge>
                        </div>
                    </div>
                </div>

                <div className="flex items-center gap-3 bg-white/5 px-4 py-2 rounded-full border border-white/10">
                    <MdAutoGraph className="text-primary" />
                    <span className="text-xs font-bold text-gray-400 uppercase tracking-widest">Growth Path</span>
                    <div className="flex gap-1">
                        {[0, 1, 2].map((t) => (
                            <div 
                                key={t} 
                                className={`w-2 h-2 rounded-full ${user?.kyc_tier >= t ? 'bg-primary shadow-[0_0_8px_var(--primary)]' : 'bg-white/10'}`} 
                            />
                        ))}
                    </div>
                </div>
            </div>

            {/* Main Multi-Step Form */}
            <div className="grid grid-cols-1 lg:grid-cols-12 gap-8">
                {/* Sidebar Progress */}
                <div className="lg:col-span-3 space-y-4">
                    {[
                        { id: 1, label: 'Identity & PayID', icon: <MdOutlineFingerprint />, desc: 'Claim your on-chain ID' },
                        { id: 2, label: 'Social Signals', icon: <MdRocketLaunch />, desc: 'Build your trust score' }
                    ].map((s) => (
                        <button 
                            key={s.id}
                            onClick={() => s.id <= (user?.kyc_tier + 1) && setStep(s.id)}
                            className={`w-full text-left p-4 rounded-2xl border transition-all ${
                                step === s.id 
                                ? 'bg-primary/10 border-primary/40 shadow-lg' 
                                : 'bg-white/20 border-transparent opacity-40 hover:opacity-100'
                            }`}
                        >
                            <div className="flex items-center gap-3 mb-1">
                                <span className={`text-xl ${step === s.id ? 'text-primary' : 'text-gray-400'}`}>{s.icon}</span>
                                <span className="text-xs font-black uppercase tracking-wider">{s.label}</span>
                            </div>
                            <p className="text-[10px] text-gray-500 font-medium pl-8">{s.desc}</p>
                        </button>
                    ))}
                </div>

                {/* Step Content */}
                <div className="lg:col-span-9">
                    {step === 1 && (
                        <Card className="bg-white/5 border-white/10 backdrop-blur-sm animate-in slide-in-from-right-8 duration-500">
                            <CardHeader>
                                <CardTitle className="flex items-center gap-2 text-white">
                                    <MdLockOpen className="text-primary" />
                                    Identity Verification
                                </CardTitle>
                                <p className="text-sm text-gray-400">Verifying your identity unlocks PayID and enables zero-fee interoperability within the FidduPay ecosystem.</p>
                            </CardHeader>
                            <CardContent className="space-y-8">
                                {/* Username Claim */}
                                <div className="space-y-3">
                                    <label className="text-xs font-bold text-gray-500 uppercase tracking-widest">Claim your @username</label>
                                    <div className="flex gap-3">
                                        <div className="relative flex-1 group">
                                            <div className="absolute left-4 top-1/2 -translate-y-1/2 text-gray-500 group-focus-within:text-primary transition-colors">
                                                <MdPerson size={20} />
                                            </div>
                                            <input 
                                                type="text" 
                                                placeholder="e.g. techy_store"
                                                value={idData.username}
                                                onChange={(e) => setIdData({...idData, username: e.target.value.toLowerCase().replace(/[^a-z0-9_]/g, '')})}
                                                className="w-full bg-black/40 border border-white/10 rounded-xl py-4 pl-12 pr-4 text-white focus:outline-none focus:border-primary transition-all font-mono"
                                                disabled={!!user?.username}
                                            />
                                        </div>
                                        {!user?.username && (
                                            <button 
                                                onClick={handleClaimUsername}
                                                disabled={loading || !idData.username}
                                                className="bg-primary hover:bg-primary-hover text-white px-8 rounded-xl font-black transition-all disabled:opacity-50 shadow-lg shadow-primary/20"
                                            >
                                                Claim
                                            </button>
                                        )}
                                    </div>
                                    {user?.username && (
                                        <div className="flex items-center gap-2 text-green-500 text-xs font-bold bg-green-500/10 p-2 rounded-lg w-fit">
                                            <MdCheckCircle /> Identity Linked: @{user.username}
                                        </div>
                                    )}
                                </div>

                                {/* ID Number */}
                                <div className="space-y-3">
                                    <label className="text-xs font-bold text-gray-500 uppercase tracking-widest">National Identity (NIN or BVN)</label>
                                    <div className="relative group">
                                        <div className="absolute left-4 top-1/2 -translate-y-1/2 text-gray-500 group-focus-within:text-primary transition-colors">
                                            <MdSecurity size={20} />
                                        </div>
                                        <input 
                                            type="password" 
                                            placeholder="Enter 11-digit number"
                                            value={idData.nin_bvn}
                                            onChange={(e) => setIdData({...idData, nin_bvn: e.target.value})}
                                            className="w-full bg-black/40 border border-white/10 rounded-xl py-4 pl-12 pr-4 text-white focus:outline-none focus:border-primary transition-all font-mono tracking-widest"
                                            maxLength={11}
                                        />
                                    </div>
                                    <div className="flex gap-2 items-center bg-blue-500/5 p-3 rounded-xl border border-blue-500/10">
                                        <MdShield className="text-blue-500 shrink-0" />
                                        <p className="text-[10px] text-blue-300/60 italic leading-relaxed">
                                            FidduPay never stores raw ID numbers. They are immediately hashed into a one-way cryptographic signal for the Trust Intelligence Layer.
                                        </p>
                                    </div>
                                </div>

                                <button 
                                    onClick={handleSubmitTier1}
                                    disabled={loading || !idData.nin_bvn}
                                    className="w-full bg-gradient-to-r from-primary to-indigo-600 text-white py-5 rounded-2xl font-black flex items-center justify-center gap-3 shadow-2xl shadow-primary/30 hover:scale-[1.01] active:scale-[0.99] transition-all disabled:opacity-50"
                                >
                                    {loading ? 'Processing Protocol...' : (
                                        <>Finalize Tier 1 Verification <MdArrowForward size={20} /></>
                                    )}
                                </button>
                            </CardContent>
                        </Card>
                    )}

                    {step === 2 && (
                        <Card className="bg-white/5 border-white/10 backdrop-blur-sm animate-in slide-in-from-right-8 duration-500">
                            <CardHeader>
                                <CardTitle className="flex items-center gap-2 text-white">
                                    <MdRocketLaunch className="text-secondary" />
                                    Social Trust Signals
                                </CardTitle>
                                <p className="text-sm text-gray-400">Enhance your Merchant Trust Score by linking your business’s digital footprint.</p>
                            </CardHeader>
                            <CardContent className="space-y-6">
                                <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                                    <div className="space-y-2">
                                        <label className="text-[10px] font-bold text-gray-500 uppercase tracking-widest">Twitter (X)</label>
                                        <div className="relative group">
                                            <FaTwitter className="absolute left-4 top-1/2 -translate-y-1/2 text-gray-500 group-focus-within:text-[#1DA1F2]" />
                                            <input 
                                                type="text" placeholder="@handle" value={socialData.twitter}
                                                onChange={(e) => setSocialData({...socialData, twitter: e.target.value})}
                                                className="w-full bg-black/40 border border-white/10 rounded-xl py-3 pl-12 pr-4 text-white focus:outline-none focus:border-[#1DA1F2]"
                                            />
                                        </div>
                                    </div>

                                    <div className="space-y-2">
                                        <label className="text-[10px] font-bold text-gray-500 uppercase tracking-widest">LinkedIn</label>
                                        <div className="relative group">
                                            <FaLinkedin className="absolute left-4 top-1/2 -translate-y-1/2 text-gray-500 group-focus-within:text-[#0A66C2]" />
                                            <input 
                                                type="text" placeholder="company/profile" value={socialData.linkedin}
                                                onChange={(e) => setSocialData({...socialData, linkedin: e.target.value})}
                                                className="w-full bg-black/40 border border-white/10 rounded-xl py-3 pl-12 pr-4 text-white focus:outline-none focus:border-[#0A66C2]"
                                            />
                                        </div>
                                    </div>

                                    <div className="space-y-2">
                                        <label className="text-[10px] font-bold text-gray-500 uppercase tracking-widest">Official Website</label>
                                        <div className="relative group">
                                            <FaGlobe className="absolute left-4 top-1/2 -translate-y-1/2 text-gray-500 group-focus-within:text-primary" />
                                            <input 
                                                type="text" placeholder="https://..." value={socialData.website}
                                                onChange={(e) => setSocialData({...socialData, website: e.target.value})}
                                                className="w-full bg-black/40 border border-white/10 rounded-xl py-3 pl-12 pr-4 text-white focus:outline-none focus:border-primary"
                                            />
                                        </div>
                                    </div>

                                    <div className="space-y-2">
                                        <label className="text-[10px] font-bold text-gray-500 uppercase tracking-widest">Business License</label>
                                        <div className="relative group">
                                            <MdBusiness className="absolute left-4 top-1/2 -translate-y-1/2 text-gray-500 group-focus-within:text-secondary" />
                                            <input 
                                                type="text" placeholder="CAC / RC Number" value={socialData.business_license}
                                                onChange={(e) => setSocialData({...socialData, business_license: e.target.value})}
                                                className="w-full bg-black/40 border border-white/10 rounded-xl py-3 pl-12 pr-4 text-white focus:outline-none focus:border-secondary"
                                            />
                                        </div>
                                    </div>
                                </div>

                                <div className="flex gap-4 pt-4">
                                    <button 
                                        onClick={() => setStep(1)}
                                        className="px-8 py-4 rounded-2xl border border-white/10 text-gray-500 hover:text-white hover:bg-white/5 transition-all font-black uppercase text-xs tracking-widest"
                                    >
                                        Back
                                    </button>
                                    <button 
                                        onClick={handleSubmitTier2}
                                        disabled={loading}
                                        className="flex-1 bg-gradient-to-r from-secondary to-amber-600 text-white py-4 rounded-2xl font-black shadow-2xl shadow-secondary/20 hover:scale-[1.01] active:scale-[0.99] transition-all disabled:opacity-50"
                                    >
                                        {loading ? 'Optimizing Signals...' : 'Reach Gold Trust Status'}
                                    </button>
                                </div>
                            </CardContent>
                        </Card>
                    )}
                </div>
            </div>
        </div>
    )
}

export default VerificationTab
