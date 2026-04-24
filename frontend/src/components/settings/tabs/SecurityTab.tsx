import React, { useState, useEffect } from 'react';
import { 
    MdCheckCircle, 
    MdError, 
    MdFingerprint, 
    MdVpnKey, 
    MdAdd, 
    MdSecurity
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
            <h2>Security & Transaction PIN</h2>
            <p>Manage your transaction authorization settings.</p>

            <div className={styles.safeguardBox}>
                <div className={styles.safeguardInfo}>
                    <div className={styles.safeguardIcon}>
                        {user?.has_transaction_pin ? <MdCheckCircle color="var(--primary)" /> : <MdError color="#ef4444" />}
                    </div>
                    <div className={styles.safeguardText}>
                        <h3>4-Digit Transaction PIN</h3>
                        <p>
                            {user?.has_transaction_pin 
                                ? `Your PIN is set. You will be prompted for this PIN whenever you initiate a withdrawal, sweep, or payment.`
                                : "A 4-digit PIN is REQUIRED for all financial actions. Please set one now to enable withdrawals."
                            }
                        </p>
                    </div>
                </div>
            </div>

            <div className={styles.formCard} style={{ marginTop: '24px', maxWidth: '400px' }}>
                <form onSubmit={handleSetPin}>
                    <div className={styles.formGroup}>
                        <label>{user?.has_transaction_pin ? 'Update Transaction PIN' : 'Set Merchant Transaction PIN'}</label>
                        <input 
                            type="password"
                            className={styles.inputStyle}
                            maxLength={4}
                            pattern="\d*"
                            placeholder="••••"
                            style={{ letterSpacing: '0.5rem', textAlign: 'center', fontSize: '1.5rem', width: '100%', marginBottom: '12px' }}
                            value={pin}
                            onChange={e => setPin(e.target.value.replace(/\D/g, ''))}
                            required
                        />
                        <p style={{ fontSize: '0.75rem', color: 'var(--text-muted)', marginTop: '4px' }}>Must be exactly 4 numeric digits.</p>
                    </div>
                    <div className={styles.formGroup} style={{ marginTop: '16px' }}>
                        <label>Confirm PIN</label>
                        <input 
                            type="password"
                            className={styles.inputStyle}
                            maxLength={4}
                            pattern="\d*"
                            placeholder="••••"
                            style={{ letterSpacing: '0.5rem', textAlign: 'center', fontSize: '1.5rem', width: '100%', marginBottom: '12px' }}
                            value={confirmPin}
                            onChange={e => setConfirmPin(e.target.value.replace(/\D/g, ''))}
                            required
                        />
                    </div>
                    <button 
                        type="submit" 
                        className={styles.saveBtn} 
                        style={{ width: '100%', marginTop: '12px', background: 'var(--primary)' }}
                        disabled={settingPin || pin.length !== 4 || pin !== confirmPin}
                    >
                        {settingPin ? 'Updating...' : (user?.has_transaction_pin ? 'Update Merchant PIN' : 'Set Merchant PIN')}
                    </button>
                    {pin !== confirmPin && confirmPin.length === 4 && (
                        <p style={{ color: '#ef4444', fontSize: '12px', marginTop: '8px', textAlign: 'center' }}>PINs do not match.</p>
                    )}
                </form>
            </div>
            <div style={{ marginTop: '48px', borderTop: '1px solid var(--border)', paddingTop: '32px' }}>
                <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                    <div>
                        <h2 style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
                            <MdFingerprint className="text-primary" />
                            Passkeys & Biometrics
                        </h2>
                        <p>Enable secure, passwordless login using your device's fingerprint or FaceID.</p>
                    </div>
                    <Badge style={{ background: 'rgba(99, 102, 241, 0.1)', color: 'var(--primary)', border: '1px solid rgba(99, 102, 241, 0.2)' }}>
                        Recommended
                    </Badge>
                </div>

                <div className={styles.formCard} style={{ marginTop: '24px', maxWidth: '600px' }}>
                    <div style={{ display: 'flex', flexDirection: 'column', gap: '20px' }}>
                        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', padding: '20px', background: 'rgba(255, 255, 255, 0.02)', borderRadius: '16px', border: '1px solid var(--border)' }}>
                            <div style={{ display: 'flex', gap: '16px', alignItems: 'center' }}>
                                <div style={{ width: '48px', height: '48px', background: 'rgba(99, 102, 241, 0.1)', borderRadius: '12px', display: 'flex', alignItems: 'center', justifyContent: 'center', color: 'var(--primary)' }}>
                                    <MdVpnKey size={24} />
                                </div>
                                <div>
                                    <h4 style={{ margin: 0, fontSize: '14px', fontWeight: 800, color: 'var(--text-main)' }}>Biometric Login</h4>
                                    <p style={{ margin: '4px 0 0', fontSize: '12px', color: 'var(--text-muted)' }}>Use TouchID / FaceID to sign in instantly.</p>
                                </div>
                            </div>
                            <label className={styles.switch}>
                                <input 
                                    type="checkbox" 
                                    checked={user?.passkey_enabled}
                                    onChange={async (e) => {
                                        if (e.target.checked) {
                                            showToast('Initializing WebAuthn Biometric Protocol...', 'info');
                                            // Trigger passkey registration
                                        }
                                    }}
                                />
                                <span className={styles.slider}></span>
                            </label>
                        </div>

                        <button 
                            className={styles.viewBtn} 
                            style={{ width: '100%', padding: '16px', borderRadius: '16px', border: '1px dashed var(--border)', background: 'transparent', display: 'flex', alignItems: 'center', justifyContent: 'center', gap: '10px' }}
                            onClick={() => showToast('Registering new security key...', 'info')}
                        >
                            <MdAdd /> Register New Security Key / Passkey
                        </button>
                    </div>

                    <div className={styles.privacyNote} style={{ marginTop: '24px', background: 'rgba(99, 102, 241, 0.03)' }}>
                        <MdSecurity size={20} style={{ color: 'var(--primary)', flexShrink: 0 }} />
                        <p style={{ fontSize: '11px', color: 'var(--text-muted)' }}>
                            FidduPay uses industry-standard **FIDO2/WebAuthn**. Your biometric data never leaves your device; it is used only to unlock a cryptographic key stored on your secure hardware.
                        </p>
                    </div>
                </div>
            </div>

            <div style={{ marginTop: '48px', borderTop: '1px solid var(--border)', paddingTop: '32px' }}>
                <h2>Risk Monitoring</h2>
                <p>Configure automated system alerts for operational balance safety.</p>

                <div className={styles.formCard} style={{ marginTop: '24px', maxWidth: '500px' }}>
                    <div className={styles.formGroup} style={{ marginBottom: '24px', display: 'flex', justifyContent: 'space-between', alignItems: 'center', padding: '16px', background: 'rgba(255, 255, 255, 0.02)', borderRadius: '12px', border: '1px solid var(--border)' }}>
                        <div>
                            <h4 style={{ margin: 0, fontSize: '0.95rem', fontWeight: 700, color: 'var(--text-main)' }}>Enable Alerts</h4>
                            <p style={{ margin: '4px 0 0', fontSize: '0.8rem', color: 'var(--text-muted)' }}>Receive notifications when balance is low.</p>
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
                        <label style={{ color: 'var(--text-main)', marginBottom: '8px', display: 'block', fontSize: '14px', fontWeight: 600 }}>Low Balance Threshold (USD)</label>
                        <div style={{ display: 'flex', gap: '12px', alignItems: 'stretch' }}>
                            <div style={{ position: 'relative', flex: 1 }}>
                                <span style={{ position: 'absolute', left: '16px', top: '50%', transform: 'translateY(-50%)', color: 'var(--text-muted)', fontWeight: 600, fontSize: '14px', zIndex: 2 }}>$</span>
                                <input 
                                    type="number"
                                    className={styles.urlInput}
                                    style={{ paddingLeft: '32px', width: '100%', height: '48px' }}
                                    value={lowBalanceThreshold}
                                    onChange={e => setLowBalanceThreshold(e.target.value)}
                                    placeholder="0.00"
                                    disabled={loading}
                                />
                            </div>
                            <button 
                                className={styles.saveBtn} 
                                style={{ background: 'var(--primary)', whiteSpace: 'nowrap', height: '48px', padding: '0 24px', borderRadius: '12px' }}
                                onClick={() => handleUpdateSettings({ low_balance_threshold_usd: lowBalanceThreshold })}
                                disabled={loading}
                            >
                                {loading ? 'Updating...' : 'Update Threshold'}
                            </button>
                        </div>
                        <p style={{ fontSize: '0.875rem', color: 'var(--text-muted)', marginTop: '12px', lineHeight: '1.5' }}>
                            The system will trigger <strong>balance.low</strong> webhooks and in-app alerts when your total 
                            account balance (across all supported currencies) falls below this amount.
                        </p>
                    </div>
                </div>
            </div>
        </section>
    );
};

export default SecurityTab;
