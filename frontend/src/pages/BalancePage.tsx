import React, { useEffect, useState } from 'react'
import { merchantAPI } from '@/services/apiService'
import { Balance } from '@/types'
import { useToast } from '@/contexts/ToastContext'
import styles from '@/styles/pages/BalancePage.module.css'

const BalancePage: React.FC = () => {
    const [balance, setBalance] = useState<Balance | null>(null)
    const [loading, setLoading] = useState(true)
    const { showToast } = useToast()

    useEffect(() => {
        loadBalance()
    }, [])

    const loadBalance = async () => {
        try {
            setLoading(true)
            const response = await merchantAPI.getBalance()
            if (response.data) {
                setBalance(response.data)
            }
        } catch (error) {
            console.error('Failed to load balance:', error)
            showToast('Failed to load balance data', 'error')
        } finally {
            setLoading(false)
        }
    }

    return (
        <div className={styles.page}>
            <div className={styles.header}>
                <div>
                    <h1><i className="fas fa-wallet"></i> Balance</h1>
                    <p>View and manage your cryptocurrency balances across all networks</p>
                </div>
                <button
                    className={styles.refreshBtn}
                    onClick={loadBalance}
                    disabled={loading}
                >
                    <i className={`fas fa-sync-alt ${loading ? 'fa-spin' : ''}`}></i>
                    Refresh
                </button>
            </div>

            {loading && !balance ? (
                <div className={styles.loadingState}>
                    <i className="fas fa-spinner fa-spin"></i>
                    <p>Loading your balances...</p>
                </div>
            ) : (
                <div className={styles.content}>
                    <div className={styles.statsGrid}>
                        <div className={styles.statCard}>
                            <div className={styles.statIcon}>
                                <i className="fas fa-dollar-sign"></i>
                            </div>
                            <div className={styles.statInfo}>
                                <p className={styles.statLabel}>Total Balance</p>
                                <p className={styles.statValue}>${parseFloat(balance?.total_usd || '0').toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 })}</p>
                            </div>
                        </div>

                        <div className={styles.statCard}>
                            <div className={styles.statIcon} style={{ background: 'rgba(34, 197, 94, 0.1)', color: '#16a34a' }}>
                                <i className="fas fa-check-circle"></i>
                            </div>
                            <div className={styles.statInfo}>
                                <p className={styles.statLabel}>Available to Withdraw</p>
                                <p className={styles.statValue}>${parseFloat(balance?.available_usd || '0').toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 })}</p>
                            </div>
                        </div>

                        <div className={styles.statCard}>
                            <div className={styles.statIcon} style={{ background: 'rgba(249, 115, 22, 0.1)', color: '#ea580c' }}>
                                <i className="fas fa-clock"></i>
                            </div>
                            <div className={styles.statInfo}>
                                <p className={styles.statLabel}>Reserved / Processing</p>
                                <p className={styles.statValue}>${parseFloat(balance?.reserved_usd || '0').toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 })}</p>
                            </div>
                        </div>
                    </div>

                    <div className={styles.balanceListSection}>
                        <div className={styles.sectionHeader}>
                            <h2>Assets breakdown</h2>
                        </div>

                        <div className={styles.assetsGrid}>
                            {balance?.balances && balance.balances.length > 0 ? (
                                balance.balances.map((asset) => (
                                    <div key={asset.crypto_type} className={styles.assetCard}>
                                        <div className={styles.assetHeader}>
                                            <div className={styles.assetIcon}>
                                                <i className={`fab fa-${getIconForCrypto(asset.crypto_type)}`}></i>
                                            </div>
                                            <div className={styles.assetName}>
                                                <h3>{asset.crypto_type.split('_')[0]}</h3>
                                                <span>{asset.crypto_type.includes('_') ? asset.crypto_type.split('_')[1] : 'Native'}</span>
                                            </div>
                                        </div>
                                        <div className={styles.assetValues}>
                                            <div className={styles.cryptoAmount}>
                                                {parseFloat(asset.amount).toFixed(6)} {asset.crypto_type.split('_')[0]}
                                            </div>
                                            <div className={styles.usdAmount}>
                                                ${parseFloat(asset.amount_usd).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 })}
                                            </div>
                                        </div>
                                    </div>
                                ))
                            ) : (
                                <div className={styles.emptyAssets}>
                                    <i className="fas fa-coins"></i>
                                    <p>No crypto assets found in your account yet.</p>
                                </div>
                            )}
                        </div>
                    </div>
                </div>
            )}
        </div>
    )
}

// Helper to get font-awesome icons for crypto
function getIconForCrypto(type: string): string {
    const t = type.toLowerCase()
    if (t.includes('eth')) return 'ethereum'
    if (t.includes('btc')) return 'bitcoin'
    if (t.includes('usdt') || t.includes('usdc')) return 'dollar-sign' // FA doesn't have USDT
    if (t.includes('sol')) return 'bolt' // Proxy for Solana
    if (t.includes('bnb')) return 'binance' // Might needing fab
    return 'coins'
}

export default BalancePage
