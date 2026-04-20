import React from 'react';
import { MdCheckCircle, MdError } from 'react-icons/md';

interface SecurityTabProps {
    user: any;
    pin: string;
    setPin: (pin: string) => void;
    confirmPin: string;
    setConfirmPin: (pin: string) => void;
    handleSetPin: (e: React.FormEvent) => Promise<void>;
    settingPin: boolean;
    lowBalanceThreshold: string;
    setLowBalanceThreshold: (value: string) => void;
    lowBalanceAlertsEnabled: boolean;
    setLowBalanceAlertsEnabled: (value: boolean) => void;
    handleUpdateSettings: (updates: any) => Promise<void>;
    styles: any;
}

const SecurityTab: React.FC<SecurityTabProps> = ({
    user,
    pin,
    setPin,
    confirmPin,
    setConfirmPin,
    handleSetPin,
    settingPin,
    lowBalanceThreshold,
    setLowBalanceThreshold,
    lowBalanceAlertsEnabled,
    setLowBalanceAlertsEnabled,
    handleUpdateSettings,
    styles
}) => {
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
                        {settingPin ? <i className="fas fa-spinner fa-spin"></i> : (user?.has_transaction_pin ? 'Update Merchant PIN' : 'Set Merchant PIN')}
                    </button>
                    {pin !== confirmPin && confirmPin.length === 4 && (
                        <p style={{ color: '#ef4444', fontSize: '12px', marginTop: '8px', textAlign: 'center' }}>PINs do not match.</p>
                    )}
                </form>
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
                                />
                            </div>
                            <button 
                                className={styles.saveBtn} 
                                style={{ background: 'var(--primary)', whiteSpace: 'nowrap', height: '48px', padding: '0 24px', borderRadius: '12px' }}
                                onClick={() => handleUpdateSettings({ low_balance_threshold_usd: lowBalanceThreshold })}
                            >
                                Update Threshold
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
