import React, { useState, useEffect } from 'react';
import { 
    MdCheckCircle, 
    MdCloudDone, 
    MdLock, 
    MdWarning, 
    MdClose, 
    MdRefresh, 
    MdBolt, 
    MdSecurity, 
    MdAutorenew,
    MdArrowForward,
    MdShield,
    MdForward
} from 'react-icons/md';
import { merchantAPI, securityAPI, addressOnlyAPI } from '@/services/apiService';
import { useToast } from '@/contexts/ToastContext';
import { useAuthStore } from '@/stores/authStore';

interface SettlementTabProps {
    user: any;
    styles: any;
}

const SettlementTab: React.FC<SettlementTabProps> = ({
    user,
    styles
}) => {
    const { showToast } = useToast();
    const { loadUser } = useAuthStore();
    const [loading, setLoading] = useState(false);
    
    const [selectedMode, setSelectedMode] = useState<'forwarding' | 'managed'>(user?.settlement_mode || 'managed');
    const [autoSettlementEnabled, setAutoSettlementEnabled] = useState(user?.auto_settlement_enabled ?? true);
    const [addressOnlyCustomerPaysFee, setAddressOnlyCustomerPaysFee] = useState(false);
    const [walletsLocked, setWalletsLocked] = useState(user?.wallets_locked || false);
    const [customerWalletsLocked, setCustomerWalletsLocked] = useState(user?.customer_wallets_locked || false);
    
    const [passwordConfirm, setPasswordConfirm] = useState<{
        show: boolean;
        target: 'wallet' | 'customer' | null;
        newLockState: boolean;
        password: string;
    }>({
        show: false,
        target: null,
        newLockState: false,
        password: ''
    });

    useEffect(() => {
        if (user) {
            setSelectedMode(user.settlement_mode || 'managed');
            setAutoSettlementEnabled(user.auto_settlement_enabled ?? true);
            setWalletsLocked(user.wallets_locked || false);
            setCustomerWalletsLocked(user.customer_wallets_locked || false);
            fetchExtraSettings();
        }
    }, [user]);

    const fetchExtraSettings = async () => {
        try {
            const aoFeeRes = await addressOnlyAPI.getFeeSetting();
            setAddressOnlyCustomerPaysFee(aoFeeRes.data.customer_pays_fee);
        } catch (err) {
            console.warn('Address-only settings not available:', err);
        }
    };

    const handleToggleAutoSettlement = async (enabled: boolean) => {
        try {
            setLoading(true);
            await merchantAPI.updateSettings({ auto_settlement_enabled: enabled });
            setAutoSettlementEnabled(enabled);
            await loadUser(true);
            showToast(`Auto-settlement ${enabled ? 'enabled' : 'disabled'} successfully`, 'success');
        } catch (error: any) {
            showToast(error.response?.data?.error || 'Failed to update auto-settlement preference', 'error');
        } finally {
            setLoading(false);
        }
    };

    const handleUpdateSettlementMode = async (mode: 'forwarding' | 'managed') => {
        try {
            setLoading(true);
            await merchantAPI.updateSettings({ settlement_mode: mode });
            setSelectedMode(mode);
            await loadUser(true);
            showToast('Settlement mode updated successfully', 'success');
        } catch (error: any) {
            showToast(error.response?.data?.error || 'Failed to update settlement mode', 'error');
        } finally {
            setLoading(false);
        }
    };

    const handleUpdateAddressOnlyFeeSetting = async (customerPays: boolean) => {
        try {
            setLoading(true);
            await addressOnlyAPI.updateFeeSetting({ customer_pays_fee: customerPays });
            setAddressOnlyCustomerPaysFee(customerPays);
            showToast(`Forwarding fees updated: ${customerPays ? 'Customer' : 'Merchant'} pays`, 'success');
        } catch (error: any) {
            showToast('Failed to update forwarding fee preference', 'error');
        } finally {
            setLoading(false);
        }
    };

    const handleToggleWalletLock = () => {
        if (!user) return;
        setPasswordConfirm({
            show: true,
            target: 'wallet',
            newLockState: !user.wallets_locked,
            password: ''
        });
    };

    const handleToggleCustomerWalletLock = () => {
        if (!user) return;
        setPasswordConfirm({
            show: true,
            target: 'customer',
            newLockState: !user.customer_wallets_locked,
            password: ''
        });
    };

    const confirmLockAction = async () => {
        if (!passwordConfirm.target || !passwordConfirm.password) {
            showToast('Please enter your password to confirm', 'error');
            return;
        }

        try {
            setLoading(true);
            if (passwordConfirm.target === 'wallet') {
                await securityAPI.toggleWalletLock(passwordConfirm.newLockState, passwordConfirm.password);
                setWalletsLocked(passwordConfirm.newLockState);
                showToast(`Wallets ${passwordConfirm.newLockState ? 'locked' : 'unlocked'} successfully`, 'success');
            } else {
                await securityAPI.toggleCustomerWalletLock(passwordConfirm.newLockState, passwordConfirm.password);
                setCustomerWalletsLocked(passwordConfirm.newLockState);
                showToast(`Customer wallets ${passwordConfirm.newLockState ? 'locked' : 'unlocked'} successfully`, 'success');
            }
            
            setPasswordConfirm({ show: false, target: null, newLockState: false, password: '' });

            try {
                await loadUser(true);
            } catch (profileErr) {
                console.error('Profile sync failed but action succeeded:', profileErr);
            }
        } catch (error: any) {
            showToast(error.response?.data?.error || 'Security verification failed', 'error');
        } finally {
            setLoading(false);
        }
    };

    const MdLockOpen = (props: any) => (
        <svg stroke="currentColor" fill="currentColor" strokeWidth="0" viewBox="0 0 24 24" height="1em" width="1em" xmlns="http://www.w3.org/2000/svg" {...props}>
            <path fill="none" d="M0 0h24v24H0V0z"></path>
            <path d="M18 8h-1V6c0-2.76-2.24-5-5-5S7 3.24 7 6h2c0-1.66 1.34-3 3-3s3 1.34 3 3v2H6c-1.1 0-2 .9-2 2v10c0 1.1.9 2 2 2h12c1.1 0 2-.9 2-2V10c0-1.1-.9-2-2-2zm0 12H6V10h12v10zm-6-3c1.1 0 2-.9 2-2s-.9-2-2-2-2 .9-2 2 .9 2 2 2z"></path>
        </svg>
    );

    return (
        <section className={styles.section} style={{ display: 'flex', flexDirection: 'column', gap: '24px' }}>
            {/* Header Title */}
            <div>
                <div style={{
                    display: 'inline-flex',
                    alignItems: 'center',
                    gap: '6px',
                    padding: '4px 12px',
                    borderRadius: '20px',
                    background: 'rgba(99, 102, 241, 0.12)',
                    border: '1px solid rgba(99, 102, 241, 0.25)',
                    color: '#818cf8',
                    fontSize: '12px',
                    fontWeight: 700,
                    letterSpacing: '0.05em',
                    marginBottom: '8px'
                }}>
                    <MdBolt style={{ fontSize: '14px' }} /> SETTLEMENT ENGINE & SECURITY CONTROL
                </div>
                <h2 style={{ margin: '0 0 4px', fontSize: '22px', fontWeight: 800, color: 'var(--text-main)' }}>
                    Settlement & Safeguard Settings
                </h2>
                <p style={{ margin: 0, fontSize: '14px', color: 'var(--text-muted)' }}>
                    Configure how customer deposits are credited off-chain and manage wallet security locks.
                </p>
            </div>

            {/* Settlement Mode Selection Cards */}
            <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(280px, 1fr))', gap: '16px' }}>
                <div 
                    onClick={() => handleUpdateSettlementMode('managed')}
                    style={{
                        background: selectedMode === 'managed' ? 'rgba(99, 102, 241, 0.08)' : 'rgba(255, 255, 255, 0.02)',
                        border: selectedMode === 'managed' ? '2px solid #6366f1' : '1px solid var(--border)',
                        borderRadius: '20px',
                        padding: '24px',
                        cursor: 'pointer',
                        transition: 'all 0.3s cubic-bezier(0.16, 1, 0.3, 1)',
                        position: 'relative',
                        boxShadow: selectedMode === 'managed' ? '0 0 24px rgba(99, 102, 241, 0.15)' : 'none'
                    }}
                >
                    {selectedMode === 'managed' && (
                        <MdCheckCircle style={{ position: 'absolute', top: '16px', right: '16px', color: '#6366f1', fontSize: '22px' }} />
                    )}
                    <div style={{
                        width: '44px',
                        height: '44px',
                        borderRadius: '14px',
                        background: 'linear-gradient(135deg, #6366f1 0%, #8b5cf6 100%)',
                        display: 'flex',
                        alignItems: 'center',
                        justifyContent: 'center',
                        color: 'white',
                        fontSize: '22px',
                        marginBottom: '16px',
                        boxShadow: '0 4px 14px rgba(99, 102, 241, 0.35)'
                    }}>
                        <MdCloudDone />
                    </div>
                    <h3 style={{ margin: '0 0 6px', fontSize: '17px', fontWeight: 700, color: 'var(--text-main)' }}>
                        Managed Wallet
                    </h3>
                    <span style={{ fontSize: '13px', color: 'var(--text-muted)', lineHeight: '1.5', display: 'block' }}>
                        Customer deposits are securely credited to your FidduPay available balance instantly off-chain.
                    </span>
                </div>

                {!user?.managed_mode_only && (
                    <div 
                        onClick={() => handleUpdateSettlementMode('forwarding')}
                        style={{
                            background: selectedMode === 'forwarding' ? 'rgba(99, 102, 241, 0.08)' : 'rgba(255, 255, 255, 0.02)',
                            border: selectedMode === 'forwarding' ? '2px solid #6366f1' : '1px solid var(--border)',
                            borderRadius: '20px',
                            padding: '24px',
                            cursor: 'pointer',
                            transition: 'all 0.3s cubic-bezier(0.16, 1, 0.3, 1)',
                            position: 'relative'
                        }}
                    >
                        {selectedMode === 'forwarding' && (
                            <MdCheckCircle style={{ position: 'absolute', top: '16px', right: '16px', color: '#6366f1', fontSize: '22px' }} />
                        )}
                        <div style={{
                            width: '44px',
                            height: '44px',
                            borderRadius: '14px',
                            background: 'linear-gradient(135deg, #ec4899 0%, #8b5cf6 100%)',
                            display: 'flex',
                            alignItems: 'center',
                            justifyContent: 'center',
                            color: 'white',
                            fontSize: '22px',
                            marginBottom: '16px'
                        }}>
                            <MdForward />
                        </div>
                        <h3 style={{ margin: '0 0 6px', fontSize: '17px', fontWeight: 700, color: 'var(--text-main)' }}>
                            Forwarding Bridge (Beta)
                        </h3>
                        <span style={{ fontSize: '13px', color: 'var(--text-muted)', lineHeight: '1.5', display: 'block' }}>
                            Auto-forwards received funds directly to your registered external wallet addresses.
                        </span>
                    </div>
                )}
            </div>

            {selectedMode === 'forwarding' && (
                <div style={{ padding: '20px', background: 'rgba(255, 255, 255, 0.02)', borderRadius: '16px', border: '1px solid var(--border)' }}>
                    <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                        <div>
                            <h4 style={{ margin: 0, color: 'var(--text-main)', fontSize: '15px', fontWeight: 700 }}>Forwarding Fee Preference</h4>
                            <p style={{ margin: '4px 0 0', fontSize: '13px', color: 'var(--text-muted)' }}>
                                Choose who pays network fees for forwarding transactions.
                            </p>
                        </div>
                        <div style={{ display: 'flex', background: 'rgba(0, 0, 0, 0.3)', padding: '4px', borderRadius: '10px', border: '1px solid var(--border)' }}>
                            <button
                                style={{
                                    padding: '6px 16px',
                                    borderRadius: '8px',
                                    border: 'none',
                                    fontSize: '13px',
                                    fontWeight: 600,
                                    cursor: 'pointer',
                                    background: !addressOnlyCustomerPaysFee ? '#6366f1' : 'transparent',
                                    color: !addressOnlyCustomerPaysFee ? 'white' : 'var(--text-muted)'
                                }}
                                onClick={() => handleUpdateAddressOnlyFeeSetting(false)}
                                disabled={loading}
                            >
                                Merchant
                            </button>
                            <button
                                style={{
                                    padding: '6px 16px',
                                    borderRadius: '8px',
                                    border: 'none',
                                    fontSize: '13px',
                                    fontWeight: 600,
                                    cursor: 'pointer',
                                    background: addressOnlyCustomerPaysFee ? '#6366f1' : 'transparent',
                                    color: addressOnlyCustomerPaysFee ? 'white' : 'var(--text-muted)'
                                }}
                                onClick={() => handleUpdateAddressOnlyFeeSetting(true)}
                                disabled={loading}
                            >
                                Customer
                            </button>
                        </div>
                    </div>
                </div>
            )}

            {/* Auto-Settlement Preference Switch Card */}
            <div style={{
                background: 'rgba(255, 255, 255, 0.02)',
                border: '1px solid var(--border)',
                borderRadius: '20px',
                padding: '24px',
                boxShadow: '0 8px 32px rgba(0, 0, 0, 0.12)',
                display: 'flex',
                flexDirection: 'column',
                gap: '20px'
            }}>
                <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start', gap: '20px' }}>
                    <div style={{ display: 'flex', gap: '16px', alignItems: 'flex-start' }}>
                        <div style={{
                            width: '44px',
                            height: '44px',
                            borderRadius: '14px',
                            background: autoSettlementEnabled
                                ? 'linear-gradient(135deg, #10b981 0%, #059669 100%)'
                                : 'linear-gradient(135deg, #4b5563 0%, #374151 100%)',
                            display: 'flex',
                            alignItems: 'center',
                            justifyContent: 'center',
                            color: 'white',
                            fontSize: '22px',
                            boxShadow: autoSettlementEnabled
                                ? '0 4px 14px rgba(16, 185, 129, 0.35)'
                                : 'none',
                            flexShrink: 0,
                            transition: 'all 0.3s'
                        }}>
                            <MdAutorenew />
                        </div>
                        <div>
                            <div style={{ display: 'flex', alignItems: 'center', gap: '10px', marginBottom: '4px' }}>
                                <h3 style={{ margin: 0, fontSize: '17px', fontWeight: 700, color: 'var(--text-main)' }}>
                                    Automatic Off-Chain Settlement
                                </h3>
                                <span style={{
                                    padding: '2px 8px',
                                    borderRadius: '12px',
                                    fontSize: '11px',
                                    fontWeight: 700,
                                    background: autoSettlementEnabled ? 'rgba(16, 185, 129, 0.15)' : 'rgba(156, 163, 175, 0.15)',
                                    color: autoSettlementEnabled ? '#34d399' : '#9ca3af',
                                    border: autoSettlementEnabled ? '1px solid rgba(16, 185, 129, 0.3)' : '1px solid rgba(156, 163, 175, 0.3)'
                                }}>
                                    {autoSettlementEnabled ? 'RECOMMENDED' : 'MANUAL MODE'}
                                </span>
                            </div>
                            <p style={{ margin: 0, fontSize: '13.5px', color: 'var(--text-muted)', lineHeight: '1.5', maxWidth: '600px' }}>
                                When enabled, customer deposits automatically deduct the platform fee and credit your available balance off-chain as soon as confirmed on-chain. An automated background worker reconciles any pending customer balances hourly.
                            </p>
                        </div>
                    </div>

                    {/* Toggle Button */}
                    <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'flex-end', gap: '6px' }}>
                        <label style={{ position: 'relative', display: 'inline-block', width: '56px', height: '30px', cursor: loading ? 'not-allowed' : 'pointer' }}>
                            <input
                                type="checkbox"
                                checked={autoSettlementEnabled}
                                onChange={(e) => handleToggleAutoSettlement(e.target.checked)}
                                disabled={loading}
                                style={{ opacity: 0, width: 0, height: 0 }}
                            />
                            <span style={{
                                position: 'absolute',
                                top: 0, left: 0, right: 0, bottom: 0,
                                backgroundColor: autoSettlementEnabled ? '#10b981' : '#374151',
                                transition: 'all 0.3s cubic-bezier(0.16, 1, 0.3, 1)',
                                borderRadius: '34px',
                                boxShadow: autoSettlementEnabled ? '0 0 12px rgba(16, 185, 129, 0.4)' : 'none'
                            }}>
                                <span style={{
                                    position: 'absolute',
                                    height: '22px',
                                    width: '22px',
                                    left: autoSettlementEnabled ? '30px' : '4px',
                                    bottom: '4px',
                                    backgroundColor: 'white',
                                    transition: 'all 0.3s cubic-bezier(0.16, 1, 0.3, 1)',
                                    borderRadius: '50%',
                                    boxShadow: '0 2px 4px rgba(0,0,0,0.2)'
                                }} />
                            </span>
                        </label>
                        <span style={{ fontSize: '11px', fontWeight: 600, color: autoSettlementEnabled ? '#34d399' : 'var(--text-muted)' }}>
                            {autoSettlementEnabled ? 'ACTIVE' : 'PAUSED'}
                        </span>
                    </div>
                </div>

                {/* Settlement Pipeline Visualizer */}
                <div style={{
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'space-between',
                    flexWrap: 'wrap',
                    gap: '12px',
                    padding: '14px 18px',
                    borderRadius: '14px',
                    background: 'rgba(0, 0, 0, 0.25)',
                    border: '1px solid rgba(255, 255, 255, 0.05)'
                }}>
                    <div style={{ display: 'flex', alignItems: 'center', gap: '8px', fontSize: '12.5px', color: 'var(--text-muted)' }}>
                        <span style={{ width: '22px', height: '22px', borderRadius: '50%', background: 'rgba(99, 102, 241, 0.2)', color: '#818cf8', display: 'flex', alignItems: 'center', justifyContent: 'center', fontSize: '11px', fontWeight: 700 }}>1</span>
                        <span>On-Chain Deposit Confirmed</span>
                    </div>
                    <MdArrowForward style={{ color: 'var(--text-muted)', fontSize: '16px' }} />
                    <div style={{ display: 'flex', alignItems: 'center', gap: '8px', fontSize: '12.5px', color: 'var(--text-muted)' }}>
                        <span style={{ width: '22px', height: '22px', borderRadius: '50%', background: 'rgba(139, 92, 246, 0.2)', color: '#a78bfa', display: 'flex', alignItems: 'center', justifyContent: 'center', fontSize: '11px', fontWeight: 700 }}>2</span>
                        <span>Platform Fee Deducted</span>
                    </div>
                    <MdArrowForward style={{ color: 'var(--text-muted)', fontSize: '16px' }} />
                    <div style={{ display: 'flex', alignItems: 'center', gap: '8px', fontSize: '12.5px', color: '#34d399', fontWeight: 600 }}>
                        <span style={{ width: '22px', height: '22px', borderRadius: '50%', background: 'rgba(16, 185, 129, 0.2)', color: '#34d399', display: 'flex', alignItems: 'center', justifyContent: 'center', fontSize: '11px', fontWeight: 700 }}>3</span>
                        <span>Instant Available Balance Credit</span>
                    </div>
                </div>
            </div>

            {/* Security Safeguards Section */}
            <div style={{ marginTop: '8px' }}>
                <div style={{ display: 'flex', alignItems: 'center', gap: '8px', marginBottom: '16px' }}>
                    <MdSecurity style={{ color: '#f59e0b', fontSize: '20px' }} />
                    <h3 style={{ margin: 0, fontSize: '17px', fontWeight: 700, color: 'var(--text-main)' }}>
                        Wallet Security Safeguards
                    </h3>
                </div>

                <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(320px, 1fr))', gap: '16px' }}>
                    {/* Primary Wallet Protection Card */}
                    <div style={{
                        background: 'rgba(255, 255, 255, 0.02)',
                        border: '1px solid var(--border)',
                        borderRadius: '16px',
                        padding: '20px',
                        display: 'flex',
                        flexDirection: 'column',
                        justifyContent: 'space-between',
                        gap: '16px',
                        boxShadow: '0 4px 20px rgba(0, 0, 0, 0.08)'
                    }}>
                        <div style={{ display: 'flex', gap: '14px', alignItems: 'flex-start' }}>
                            <div style={{
                                width: '40px',
                                height: '40px',
                                borderRadius: '12px',
                                background: walletsLocked ? 'rgba(16, 185, 129, 0.12)' : 'rgba(245, 158, 11, 0.12)',
                                border: walletsLocked ? '1px solid rgba(16, 185, 129, 0.25)' : '1px solid rgba(245, 158, 11, 0.25)',
                                color: walletsLocked ? '#34d399' : '#fbbf24',
                                display: 'flex',
                                alignItems: 'center',
                                justifyContent: 'center',
                                fontSize: '20px',
                                flexShrink: 0
                            }}>
                                {walletsLocked ? <MdShield /> : <MdWarning />}
                            </div>
                            <div>
                                <div style={{ display: 'flex', alignItems: 'center', gap: '8px', marginBottom: '4px' }}>
                                    <h4 style={{ margin: 0, fontSize: '15px', fontWeight: 700, color: 'var(--text-main)' }}>
                                        Primary Master Wallets
                                    </h4>
                                    <span style={{
                                        fontSize: '11px',
                                        fontWeight: 700,
                                        padding: '2px 8px',
                                        borderRadius: '10px',
                                        background: walletsLocked ? 'rgba(16, 185, 129, 0.15)' : 'rgba(245, 158, 11, 0.15)',
                                        color: walletsLocked ? '#34d399' : '#fbbf24',
                                        display: 'inline-flex',
                                        alignItems: 'center',
                                        gap: '4px'
                                    }}>
                                        {walletsLocked ? <><MdLock style={{ fontSize: '12px' }} /> LOCKED</> : <><MdLockOpen style={{ fontSize: '12px' }} /> UNLOCKED</>}
                                    </span>
                                </div>
                                <p style={{ margin: 0, fontSize: '13px', color: 'var(--text-muted)', lineHeight: '1.4' }}>
                                    {walletsLocked
                                        ? "Master wallet addresses are locked against changes."
                                        : "Master wallets are unlocked. Locking prevents unintended edits."
                                    }
                                </p>
                            </div>
                        </div>

                        <button
                            onClick={handleToggleWalletLock}
                            disabled={loading}
                            style={{
                                width: '100%',
                                padding: '10px 16px',
                                borderRadius: '10px',
                                border: 'none',
                                fontSize: '13.5px',
                                fontWeight: 700,
                                cursor: loading ? 'not-allowed' : 'pointer',
                                background: walletsLocked
                                    ? 'linear-gradient(135deg, #d97706 0%, #b45309 100%)'
                                    : 'linear-gradient(135deg, #10b981 0%, #059669 100%)',
                                color: 'white',
                                display: 'flex',
                                alignItems: 'center',
                                justifyContent: 'center',
                                gap: '8px',
                                boxShadow: walletsLocked
                                    ? '0 4px 12px rgba(217, 119, 6, 0.3)'
                                    : '0 4px 12px rgba(16, 185, 129, 0.3)',
                                transition: 'all 0.2s'
                            }}
                        >
                            {walletsLocked ? (
                                <>
                                    <MdLockOpen style={{ fontSize: '16px' }} /> Unlock Primary Wallets
                                </>
                            ) : (
                                <>
                                    <MdLock style={{ fontSize: '16px' }} /> Lock Primary Wallets
                                </>
                            )}
                        </button>
                    </div>

                    {/* Customer Wallet Protection Card */}
                    <div style={{
                        background: 'rgba(255, 255, 255, 0.02)',
                        border: '1px solid var(--border)',
                        borderRadius: '16px',
                        padding: '20px',
                        display: 'flex',
                        flexDirection: 'column',
                        justifyContent: 'space-between',
                        gap: '16px',
                        boxShadow: '0 4px 20px rgba(0, 0, 0, 0.08)'
                    }}>
                        <div style={{ display: 'flex', gap: '14px', alignItems: 'flex-start' }}>
                            <div style={{
                                width: '40px',
                                height: '40px',
                                borderRadius: '12px',
                                background: customerWalletsLocked ? 'rgba(16, 185, 129, 0.12)' : 'rgba(245, 158, 11, 0.12)',
                                border: customerWalletsLocked ? '1px solid rgba(16, 185, 129, 0.25)' : '1px solid rgba(245, 158, 11, 0.25)',
                                color: customerWalletsLocked ? '#34d399' : '#fbbf24',
                                display: 'flex',
                                alignItems: 'center',
                                justifyContent: 'center',
                                fontSize: '20px',
                                flexShrink: 0
                            }}>
                                {customerWalletsLocked ? <MdShield /> : <MdWarning />}
                            </div>
                            <div>
                                <div style={{ display: 'flex', alignItems: 'center', gap: '8px', marginBottom: '4px' }}>
                                    <h4 style={{ margin: 0, fontSize: '15px', fontWeight: 700, color: 'var(--text-main)' }}>
                                        Customer Deposit Wallets
                                    </h4>
                                    <span style={{
                                        fontSize: '11px',
                                        fontWeight: 700,
                                        padding: '2px 8px',
                                        borderRadius: '10px',
                                        background: customerWalletsLocked ? 'rgba(16, 185, 129, 0.15)' : 'rgba(245, 158, 11, 0.15)',
                                        color: customerWalletsLocked ? '#34d399' : '#fbbf24',
                                        display: 'inline-flex',
                                        alignItems: 'center',
                                        gap: '4px'
                                    }}>
                                        {customerWalletsLocked ? <><MdLock style={{ fontSize: '12px' }} /> LOCKED</> : <><MdLockOpen style={{ fontSize: '12px' }} /> UNLOCKED</>}
                                    </span>
                                </div>
                                <p style={{ margin: 0, fontSize: '13px', color: 'var(--text-muted)', lineHeight: '1.4' }}>
                                    {customerWalletsLocked
                                        ? "Customer deposit addresses are locked against re-provisioning."
                                        : "Customer wallets are unlocked. Lock for enhanced security."
                                    }
                                </p>
                            </div>
                        </div>

                        <button
                            onClick={handleToggleCustomerWalletLock}
                            disabled={loading}
                            style={{
                                width: '100%',
                                padding: '10px 16px',
                                borderRadius: '10px',
                                border: 'none',
                                fontSize: '13.5px',
                                fontWeight: 700,
                                cursor: loading ? 'not-allowed' : 'pointer',
                                background: customerWalletsLocked
                                    ? 'linear-gradient(135deg, #d97706 0%, #b45309 100%)'
                                    : 'linear-gradient(135deg, #10b981 0%, #059669 100%)',
                                color: 'white',
                                display: 'flex',
                                alignItems: 'center',
                                justifyContent: 'center',
                                gap: '8px',
                                boxShadow: customerWalletsLocked
                                    ? '0 4px 12px rgba(217, 119, 6, 0.3)'
                                    : '0 4px 12px rgba(16, 185, 129, 0.3)',
                                transition: 'all 0.2s'
                            }}
                        >
                            {customerWalletsLocked ? (
                                <>
                                    <MdLockOpen style={{ fontSize: '16px' }} /> Unlock Customer Wallets
                                </>
                            ) : (
                                <>
                                    <MdLock style={{ fontSize: '16px' }} /> Lock Customer Wallets
                                </>
                            )}
                        </button>
                    </div>
                </div>
            </div>

            {/* Password Confirmation Security Modal */}
            {passwordConfirm.show && (
                <div className={styles.modalOverlay}>
                    <div className={styles.modal}>
                        <div className={styles.modalHeader}>
                            <h2><MdLock /> Security Confirmation</h2>
                            <button
                                className={styles.closeBtn}
                                onClick={() => setPasswordConfirm({ ...passwordConfirm, show: false })}
                                disabled={loading}
                            >
                                <MdClose />
                            </button>
                        </div>
                        <div className={styles.modalBody}>
                            <p>
                                You are about to <strong>{passwordConfirm.newLockState ? 'lock' : 'unlock'}</strong> your 
                                {passwordConfirm.target === 'wallet' ? ' primary ' : ' customer '} 
                                wallets. Please enter your password to confirm this security action.
                            </p>
                            <div className={styles.inputGroup} style={{ marginTop: '20px' }}>
                                <label style={{ fontSize: '14px', fontWeight: 600, color: 'var(--text-main)' }}>
                                    Account Password
                                </label>
                                <input
                                    type="password"
                                    value={passwordConfirm.password}
                                    onChange={(e) => setPasswordConfirm({ ...passwordConfirm, password: e.target.value })}
                                    placeholder="Enter your account password"
                                    className={styles.urlInput}
                                    autoFocus
                                    onKeyDown={(e) => e.key === 'Enter' && confirmLockAction()}
                                />
                            </div>
                        </div>
                        <div className={styles.modalActions}>
                            <button
                                className={styles.cancelBtn}
                                onClick={() => setPasswordConfirm({ ...passwordConfirm, show: false })}
                                disabled={loading}
                            >
                                Cancel
                            </button>
                            <button
                                className={styles.confirmRotateBtn}
                                onClick={confirmLockAction}
                                disabled={loading || !passwordConfirm.password}
                                style={{ backgroundColor: passwordConfirm.newLockState ? '#10b981' : '#3b82f6' }}
                            >
                                {loading ? (
                                    <>
                                        <MdRefresh className="animate-spin" /> Verifying...
                                    </>
                                ) : (
                                    <>
                                        {passwordConfirm.newLockState ? <MdLock /> : <MdLockOpen />}
                                        Confirm {passwordConfirm.newLockState ? 'Lock' : 'Unlock'}
                                    </>
                                )}
                            </button>
                        </div>
                    </div>
                </div>
            )}
        </section>
    );
};

export default SettlementTab;
