import React from 'react';
import { MdCheckCircle, MdForward, MdCloudDone, MdLock, MdWarning } from 'react-icons/md';

interface SettlementTabProps {
    user: any;
    selectedMode: 'forwarding' | 'managed';
    handleUpdateSettlementMode: (mode: 'forwarding' | 'managed') => Promise<void>;
    handleToggleWalletLock: () => Promise<void>;
    handleToggleCustomerWalletLock: () => Promise<void>;
    addressOnlyCustomerPaysFee: boolean;
    handleUpdateAddressOnlyFeeSetting: (customerPays: boolean) => Promise<void>;
    loading: boolean;
    styles: any;
}

const SettlementTab: React.FC<SettlementTabProps> = ({
    user,
    selectedMode,
    handleUpdateSettlementMode,
    handleToggleWalletLock,
    handleToggleCustomerWalletLock,
    addressOnlyCustomerPaysFee,
    handleUpdateAddressOnlyFeeSetting,
    loading,
    styles
}) => {
    return (
        <section className={styles.section}>
            <h2>Settlement Mode</h2>
            <p>Choose how you want to receive and manage your funds.</p>

            <div className={styles.modeGrid}>
                <div
                    className={`${styles.modeCard} ${selectedMode === 'forwarding' ? styles.activeCard : ''}`}
                    onClick={() => handleUpdateSettlementMode('forwarding')}
                >
                    {selectedMode === 'forwarding' && <MdCheckCircle className={styles.checkIcon} />}
                    <MdForward size={32} />
                    <h3>Forwarding Bridge (WIP)</h3>
                    <span>Auto-forwards funds to your external addresses. (Experimental)</span>
                </div>

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
                <div style={{ marginTop: '24px', padding: '20px', background: '#f8fafc', borderRadius: '12px', border: '1px solid #e2e8f0' }}>
                    <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                        <div>
                            <h4 style={{ margin: 0, color: '#1e293b' }}>Forwarding Fee Preference</h4>
                            <p style={{ margin: '4px 0 0', fontSize: '14px', color: '#64748b' }}>
                                Who pays the processing fee for address-only payments?
                            </p>
                        </div>
                        <div style={{ display: 'flex', background: '#f1f5f9', padding: '4px', borderRadius: '8px' }}>
                            <button
                                style={{
                                    padding: '6px 16px',
                                    borderRadius: '6px',
                                    border: 'none',
                                    fontSize: '14px',
                                    fontWeight: 600,
                                    cursor: 'pointer',
                                    background: !addressOnlyCustomerPaysFee ? '#fff' : 'transparent',
                                    boxShadow: !addressOnlyCustomerPaysFee ? '0 1px 3px rgba(0,0,0,0.1)' : 'none',
                                    color: !addressOnlyCustomerPaysFee ? '#2563eb' : '#64748b'
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
                                    background: addressOnlyCustomerPaysFee ? '#fff' : 'transparent',
                                    boxShadow: addressOnlyCustomerPaysFee ? '0 1px 3px rgba(0,0,0,0.1)' : 'none',
                                    color: addressOnlyCustomerPaysFee ? '#2563eb' : '#64748b'
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

            <div className={styles.safeguardBox} style={{ marginTop: '32px' }}>
                <div className={styles.safeguardInfo}>
                    <div className={styles.safeguardIcon}>
                        {user?.wallets_locked ? <MdLock color="#34d399" /> : <MdWarning color="#fbbf24" />}
                    </div>
                    <div className={styles.safeguardText}>
                        <h3>Primary Wallet Protection</h3>
                        <p>
                            {user?.wallets_locked 
                                ? "Your primary wallet addresses are locked. You must unlock them before making any changes."
                                : "Your primary wallets are currently unlocked. We recommend locking them to prevent accidental changes."
                            }
                        </p>
                    </div>
                </div>
                <button 
                    className={`${styles.lockBtn} ${user?.wallets_locked ? styles.unlocked : styles.locked}`}
                    onClick={handleToggleWalletLock}
                    disabled={loading}
                >
                    {user?.wallets_locked ? 'Unlock Wallets' : 'Lock Wallets'}
                </button>
            </div>

            <div className={styles.safeguardBox} style={{ marginTop: '20px' }}>
                <div className={styles.safeguardInfo}>
                    <div className={styles.safeguardIcon}>
                        {user?.customer_wallets_locked ? <MdLock color="#34d399" /> : <MdWarning color="#fbbf24" />}
                    </div>
                    <div className={styles.safeguardText}>
                        <h3>Customer Wallet Protection</h3>
                        <p>
                            {user?.customer_wallets_locked 
                                ? "Customer deposit addresses are locked. You must unlock them before re-provisioning wallets for your users."
                                : "Customer deposit addresses are currently unlocked. We recommend locking them for enhanced security."
                            }
                        </p>
                    </div>
                </div>
                <button 
                    className={`${styles.lockBtn} ${user?.customer_wallets_locked ? styles.unlocked : styles.locked}`}
                    onClick={handleToggleCustomerWalletLock}
                    disabled={loading}
                >
                    {user?.customer_wallets_locked ? 'Unlock Customer Wallets' : 'Lock Customer Wallets'}
                </button>
            </div>
        </section>
    );
};

export default SettlementTab;
