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
    styles
}) => {
    return (
        <section className={styles.section}>
            <h2>Security & Transaction PIN</h2>
            <p>Manage your transaction authorization settings.</p>

            <div className={styles.safeguardBox} style={{ borderLeft: '4px solid #2563eb' }}>
                <div className={styles.safeguardInfo}>
                    <div className={styles.safeguardIcon}>
                        {user?.has_transaction_pin ? <MdCheckCircle color="#10b981" /> : <MdError color="#ef4444" />}
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
                        <p style={{ fontSize: '0.75rem', color: '#64748b', marginTop: '4px' }}>Must be exactly 4 numeric digits.</p>
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
                        style={{ width: '100%', marginTop: '12px', background: '#2563eb' }}
                        disabled={settingPin || pin.length !== 4 || pin !== confirmPin}
                    >
                        {settingPin ? <i className="fas fa-spinner fa-spin"></i> : (user?.has_transaction_pin ? 'Update Merchant PIN' : 'Set Merchant PIN')}
                    </button>
                    {pin !== confirmPin && confirmPin.length === 4 && (
                        <p style={{ color: '#ef4444', fontSize: '12px', marginTop: '8px', textAlign: 'center' }}>PINs do not match.</p>
                    )}
                </form>
            </div>
        </section>
    );
};

export default SecurityTab;
