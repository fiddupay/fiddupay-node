import React, { useState, useEffect } from 'react';
import { 
    MdCheckCircle, 
    MdError, 
    MdFingerprint, 
    MdAdd, 
    MdSecurity,
    MdAutoGraph
} from 'react-icons/md';
import { Badge } from '@/components/ui/badge';
import { merchantAPI } from '@/services/apiService';
import { useToast } from '@/contexts/ToastContext';
import { useAuthStore } from '@/stores/authStore';

interface SecurityTabProps {
    user: any;
    styles: any;
}

const SecurityTab: React.FC<SecurityTabProps> = ({
    user,
    styles
}) => {
    const { showToast } = useToast();
    const { loadUser } = useAuthStore();
    const [loading, setLoading] = useState(false);
    
    const [pin, setPin] = useState('');
    const [confirmPin, setConfirmPin] = useState('');
    const [settingPin, setSettingPin] = useState(false);
    const [lowBalanceThreshold, setLowBalanceThreshold] = useState(user?.low_balance_threshold_usd || '0');
    const [lowBalanceAlertsEnabled, setLowBalanceAlertsEnabled] = useState(user?.low_balance_alerts_enabled !== false);

    useEffect(() => {
        if (user) {
            setLowBalanceThreshold(user.low_balance_threshold_usd || '0');
            setLowBalanceAlertsEnabled(user.low_balance_alerts_enabled !== false);
        }
    }, [user]);

    const handleSetPin = async (e: React.FormEvent) => {
        e.preventDefault();
        if (pin.length !== 4 || !/^\d+$/.test(pin)) {
            showToast('PIN must be exactly 4 digits', 'warning');
            return;
        }
        if (pin !== confirmPin) {
            showToast('PINs do not match', 'error');
            return;
        }

        try {
            setSettingPin(true);
            await merchantAPI.setTransactionPin(pin);
            showToast('Transaction PIN set successfully', 'success');
            setPin('');
            setConfirmPin('');
            await loadUser(true);
        } catch (error: any) {
            showToast(error.response?.data?.error || 'Failed to set PIN', 'error');
        } finally {
            setSettingPin(false);
        }
    };

    const handleUpdateSettings = async (updates: any) => {
        try {
            setLoading(true);
            await merchantAPI.updateSettings(updates);
            await loadUser(true);
            showToast('Security settings updated', 'success');
        } catch (error: any) {
            showToast('Failed to update security settings', 'error');
        } finally {
            setLoading(false);
        }
    };

    return (
        <section className={styles.section}>
            <div className={styles.formHeader} style={{ marginBottom: '32px' }}>
                <h2 style={{ fontSize: '24px', fontWeight: '800', color: 'var(--text-main)', marginBottom: '8px' }}>Security & Transaction Logic</h2>
                <p style={{ color: 'var(--text-muted)' }}>Manage your institutional-grade security protocols and risk thresholds.</p>
            </div>

            <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(min(100%, 400px), 1fr))', gap: '24px' }}>
                {/* Column 1: PIN & Authentication */}
                <div style={{ display: 'flex', flexDirection: 'column', gap: '24px' }}>
                    <div className={styles.formCard} style={{ height: '100%' }}>
                        <div style={{ display: 'flex', alignItems: 'center', gap: '12px', marginBottom: '20px' }}>
                            <div style={{ width: '40px', height: '40px', borderRadius: '10px', background: 'rgba(99, 102, 241, 0.1)', display: 'flex', alignItems: 'center', justifyContent: 'center', color: 'var(--primary)' }}>
                                <MdSecurity size={24} />
                            </div>
                            <div>
                                <h3 style={{ margin: 0, fontSize: '17px', fontWeight: '800' }}>Transaction PIN</h3>
                                <p style={{ margin: 0, fontSize: '12px', color: 'var(--text-muted)' }}>Required for all financial actions.</p>
                            </div>
                        </div>

                        <div className={styles.safeguardBox} style={{ marginBottom: '20px', padding: '12px', borderRadius: '12px', background: 'rgba(255,255,255,0.02)', border: '1px solid var(--border)' }}>
                            <div className={styles.safeguardInfo}>
                                <div className={styles.safeguardIcon}>
                                    {user?.has_transaction_pin ? <MdCheckCircle color="#22c55e" /> : <MdError color="#ef4444" />}
                                </div>
                                <div className={styles.safeguardText}>
                                    <p style={{ fontSize: '12px', margin: 0, fontWeight: '600', color: user?.has_transaction_pin ? '#22c55e' : '#ef4444' }}>
                                        {user?.has_transaction_pin ? "Protocol Active" : "Action Required"}
                                    </p>
                                </div>
                            </div>
                        </div>

                        <form onSubmit={handleSetPin}>
                            <div className={styles.formGroup}>
                                <label style={{ fontSize: '13px', fontWeight: '700', marginBottom: '10px', display: 'block' }}>
                                    {user?.has_transaction_pin ? 'Update PIN' : 'Set 4-Digit PIN'}
                                </label>
                                <input 
                                    type="password"
                                    className={styles.inputStyle}
                                    maxLength={4}
                                    pattern="\d*"
                                    placeholder="••••"
                                    style={{ letterSpacing: '0.5rem', textAlign: 'center', fontSize: '1.5rem', width: '100%', padding: '12px', background: 'rgba(0,0,0,0.2)', border: '1px solid var(--border)', borderRadius: '12px', color: 'white' }}
                                    value={pin}
                                    onChange={e => setPin(e.target.value.replace(/\D/g, ''))}
                                    required
                                />
                                <input 
                                    type="password"
                                    className={styles.inputStyle}
                                    maxLength={4}
                                    pattern="\d*"
                                    placeholder="Confirm PIN"
                                    style={{ letterSpacing: '0.5rem', textAlign: 'center', fontSize: '1.2rem', width: '100%', padding: '12px', background: 'rgba(0,0,0,0.2)', border: '1px solid var(--border)', borderRadius: '12px', color: 'white', marginTop: '12px' }}
                                    value={confirmPin}
                                    onChange={e => setConfirmPin(e.target.value.replace(/\D/g, ''))}
                                    required
                                />
                            </div>
                            <button 
                                type="submit" 
                                className={styles.saveBtn} 
                                style={{ width: '100%', marginTop: '20px', height: '48px', borderRadius: '12px' }}
                                disabled={settingPin || pin.length !== 4 || pin !== confirmPin}
                            >
                                {settingPin ? 'Syncing...' : (user?.has_transaction_pin ? 'Update Secure PIN' : 'Activate PIN Protection')}
                            </button>
                        </form>
                    </div>

                    <div className={styles.formCard}>
                        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '20px' }}>
                            <div style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
                                <div style={{ width: '40px', height: '40px', borderRadius: '10px', background: 'rgba(99, 102, 241, 0.1)', display: 'flex', alignItems: 'center', justifyContent: 'center', color: 'var(--primary)' }}>
                                    <MdFingerprint size={24} />
                                </div>
                                <div>
                                    <h3 style={{ margin: 0, fontSize: '17px', fontWeight: '800' }}>Biometrics</h3>
                                    <p style={{ margin: 0, fontSize: '12px', color: 'var(--text-muted)' }}>Passwordless FIDO2 login.</p>
                                </div>
                            </div>
                            <Badge className="bg-indigo-500/10 text-indigo-400 border-indigo-500/20">Elite</Badge>
                        </div>

                        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', padding: '16px', background: 'rgba(255, 255, 255, 0.02)', borderRadius: '12px', border: '1px solid var(--border)', marginBottom: '16px' }}>
                            <div>
                                <h4 style={{ margin: 0, fontSize: '14px', fontWeight: '700' }}>Passkey Status</h4>
                                <p style={{ margin: 0, fontSize: '11px', color: 'var(--text-muted)' }}>{user?.passkey_enabled ? 'Active on this device' : 'Not configured'}</p>
                            </div>
                            <label className={styles.switch}>
                                <input 
                                    type="checkbox" 
                                    checked={user?.passkey_enabled}
                                    onChange={async (e) => {
                                        if (e.target.checked) showToast('Initializing Biometric Registry...', 'info');
                                    }}
                                />
                                <span className={styles.slider}></span>
                            </label>
                        </div>

                        <button 
                            className={styles.viewBtn} 
                            style={{ width: '100%', height: '48px', borderRadius: '12px', border: '1px dashed var(--border)', background: 'transparent', display: 'flex', alignItems: 'center', justifyContent: 'center', gap: '10px' }}
                            onClick={() => showToast('Starting WebAuthn ceremony...', 'info')}
                        >
                            <MdAdd /> Add New Security Key
                        </button>
                    </div>
                </div>

                {/* Column 2: Risk Monitoring */}
                <div style={{ display: 'flex', flexDirection: 'column', gap: '24px' }}>
                    <div className={styles.formCard} style={{ height: 'fit-content' }}>
                        <div style={{ display: 'flex', alignItems: 'center', gap: '12px', marginBottom: '20px' }}>
                            <div style={{ width: '40px', height: '40px', borderRadius: '10px', background: 'rgba(245, 158, 11, 0.1)', display: 'flex', alignItems: 'center', justifyContent: 'center', color: '#f59e0b' }}>
                                <MdAutoGraph size={24} />
                            </div>
                            <div>
                                <h3 style={{ margin: 0, fontSize: '17px', fontWeight: '800' }}>Risk Monitoring</h3>
                                <p style={{ margin: 0, fontSize: '12px', color: 'var(--text-muted)' }}>Balance safeguards & alerts.</p>
                            </div>
                        </div>

                        <div style={{ marginBottom: '24px', display: 'flex', justifyContent: 'space-between', alignItems: 'center', padding: '16px', background: 'rgba(255, 255, 255, 0.02)', borderRadius: '12px', border: '1px solid var(--border)' }}>
                            <div>
                                <h4 style={{ margin: 0, fontSize: '14px', fontWeight: '700' }}>Balance Threshold Alerts</h4>
                                <p style={{ margin: '4px 0 0', fontSize: '11px', color: 'var(--text-muted)' }}>Webhook signal on low liquidity.</p>
                            </div>
                            <label className={styles.switch}>
                                <input 
                                    type="checkbox" 
                                    checked={lowBalanceAlertsEnabled}
                                    disabled={loading}
                                    onChange={e => {
                                        const newValue = e.target.checked;
                                        setLowBalanceAlertsEnabled(newValue);
                                        handleUpdateSettings({ low_balance_alerts_enabled: newValue });
                                    }}
                                />
                                <span className={styles.slider}></span>
                            </label>
                        </div>

                        <div className={styles.formGroup}>
                            <label style={{ color: 'var(--text-main)', marginBottom: '12px', display: 'block', fontSize: '13px', fontWeight: '700' }}>Critical Threshold (USD)</label>
                            <div className={styles.thresholdGroup}>
                                <div style={{ position: 'relative', flex: 1 }}>
                                    <span style={{ position: 'absolute', left: '16px', top: '50%', transform: 'translateY(-50%)', color: 'var(--text-muted)', fontWeight: '800', fontSize: '14px', zIndex: 2 }}>$</span>
                                    <input 
                                        type="number"
                                        className={styles.urlInput}
                                        style={{ paddingLeft: '32px', width: '100%', height: '48px', borderRadius: '12px' }}
                                        value={lowBalanceThreshold}
                                        onChange={e => setLowBalanceThreshold(e.target.value)}
                                        placeholder="0.00"
                                        disabled={loading}
                                    />
                                </div>
                                <button 
                                    className={styles.saveBtn} 
                                    style={{ background: 'var(--primary)', height: '48px', padding: '0 20px', borderRadius: '12px', fontSize: '13px' }}
                                    onClick={() => handleUpdateSettings({ low_balance_threshold_usd: lowBalanceThreshold })}
                                    disabled={loading}
                                >
                                    {loading ? '...' : 'Save'}
                                </button>
                            </div>
                            <div style={{ marginTop: '20px', padding: '16px', borderRadius: '12px', background: 'rgba(255,255,255,0.02)', border: '1px solid var(--border)', fontSize: '11px', color: 'var(--text-muted)', lineHeight: '1.5' }}>
                                <MdSecurity size={14} style={{ marginRight: '6px', color: 'var(--primary)' }} />
                                The Swarm will trigger <strong>balance.low</strong> protocols when total liquidity falls below this marker.
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </section>
    );
};

export default SecurityTab;
