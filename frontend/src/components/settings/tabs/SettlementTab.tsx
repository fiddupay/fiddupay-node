import React, { useState, useEffect } from 'react';
import { MdCheckCircle, MdForward, MdCloudDone, MdLock, MdWarning, MdClose, MdRefresh } from 'react-icons/md';
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
            
            // Clean up modal immediately
            setPasswordConfirm({ show: false, target: null, newLockState: false, password: '' });

            // Try to sync, but don't crash if backend is having profile issues
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
        <section className={styles.section}>
            <h2>Settlement Mode</h2>
            <p>Choose how you want to receive and manage your funds.</p>

            <div className={styles.modeGrid}>
                {!user?.managed_mode_only && (
                    <div
                        className={`${styles.modeCard} ${selectedMode === 'forwarding' ? styles.activeCard : ''}`}
                        onClick={() => handleUpdateSettlementMode('forwarding')}
                    >
                        {selectedMode === 'forwarding' && <MdCheckCircle className={styles.checkIcon} />}
                        <MdForward size={32} />
                        <h3>Forwarding Bridge (WIP)</h3>
                        <span>Auto-forwards funds to your external addresses. (Experimental)</span>
                    </div>
                )}

                <div
                    className={`${styles.modeCard} ${selectedMode === 'managed' ? styles.activeCard : ''}`}
                    onClick={() => handleUpdateSettlementMode('managed')}
                >
                    {selectedMode === 'managed' && <MdCheckCircle className={styles.checkIcon} />}
                    <MdCloudDone size={32} />
                    <h3>Managed Wallet</h3>
                    <span>Funds are held in FidduPay generated wallets.</span>
                </div>
            </div>

            {selectedMode === 'forwarding' && (
                <div style={{ marginTop: '24px', padding: '20px', background: 'var(--surface)', borderRadius: '12px', border: '1px solid var(--border)' }}>
                    <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                        <div>
                            <h4 style={{ margin: 0, color: 'var(--text-main)' }}>Forwarding Fee Preference</h4>
                            <p style={{ margin: '4px 0 0', fontSize: '14px', color: 'var(--text-muted)' }}>
                                Who pays the processing fee for address-only payments?
                            </p>
                        </div>
                        <div style={{ display: 'flex', background: 'var(--bg-main)', padding: '4px', borderRadius: '8px', border: '1px solid var(--border)' }}>
                            <button
                                style={{
                                    padding: '6px 16px',
                                    borderRadius: '6px',
                                    border: 'none',
                                    fontSize: '14px',
                                    fontWeight: 600,
                                    cursor: 'pointer',
                                    background: !addressOnlyCustomerPaysFee ? 'var(--surface-hover)' : 'transparent',
                                    boxShadow: !addressOnlyCustomerPaysFee ? '0 1px 3px rgba(0,0,0,0.3)' : 'none',
                                    color: !addressOnlyCustomerPaysFee ? 'var(--primary)' : 'var(--text-muted)'
                                }}
                                onClick={() => handleUpdateAddressOnlyFeeSetting(false)}
                                disabled={loading}
                            >
                                Merchant
                            </button>
                            <button
                                style={{
                                    padding: '6px 16px',
                                    borderRadius: '6px',
                                    border: 'none',
                                    fontSize: '14px',
                                    fontWeight: 600,
                                    cursor: 'pointer',
                                    background: addressOnlyCustomerPaysFee ? 'var(--surface-hover)' : 'transparent',
                                    boxShadow: addressOnlyCustomerPaysFee ? '0 1px 3px rgba(0,0,0,0.3)' : 'none',
                                    color: addressOnlyCustomerPaysFee ? 'var(--primary)' : 'var(--text-muted)'
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

            <div className={styles.safeguardBox}>
                <div className={styles.safeguardInfo}>
                    <div className={styles.safeguardIcon}>
                        {walletsLocked ? <MdLock color="var(--primary)" /> : <MdWarning color="#f59e0b" />}
                    </div>
                    <div className={styles.safeguardText}>
                        <h3>Primary Wallet Protection</h3>
                        <p>
                            {walletsLocked 
                                ? "Your primary wallet addresses are locked. You must unlock them before making any changes."
                                : "Your primary wallets are currently unlocked. We recommend locking them to prevent accidental changes."
                            }
                        </p>
                    </div>
                </div>
                <button 
                    className={`${styles.lockBtn} ${walletsLocked ? styles.unlocked : styles.locked}`}
                    onClick={handleToggleWalletLock}
                    disabled={loading}
                >
                    {walletsLocked ? 'Unlock Wallets' : 'Lock Wallets'}
                </button>
            </div>

            <div className={styles.safeguardBox} style={{ marginTop: '20px' }}>
                <div className={styles.safeguardInfo}>
                    <div className={styles.safeguardIcon}>
                        {customerWalletsLocked ? <MdLock color="var(--primary)" /> : <MdWarning color="#f59e0b" />}
                    </div>
                    <div className={styles.safeguardText}>
                        <h3>Customer Wallet Protection</h3>
                        <p>
                            {customerWalletsLocked 
                                ? "Customer deposit addresses are locked. You must unlock them before re-provisioning wallets for your users."
                                : "Customer deposit addresses are currently unlocked. We recommend locking them for enhanced security."
                            }
                        </p>
                    </div>
                </div>
                <button 
                    className={`${styles.lockBtn} ${customerWalletsLocked ? styles.unlocked : styles.locked}`}
                    onClick={handleToggleCustomerWalletLock}
                    disabled={loading}
                >
                    {customerWalletsLocked ? 'Unlock Customer Wallets' : 'Lock Customer Wallets'}
                </button>
            </div>

            {/* Password Confirmation Modal */}
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
                                wallets. This is a sensitive security action.
                            </p>
                            <div className={styles.inputGroup} style={{ marginTop: '20px' }}>
                                <label style={{ fontSize: '14px', fontWeight: 600, color: '#374151' }}>
                                    Enter Account Password
                                </label>
                                <input
                                    type="password"
                                    value={passwordConfirm.password}
                                    onChange={(e) => setPasswordConfirm({ ...passwordConfirm, password: e.target.value })}
                                    placeholder="Your account password"
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
