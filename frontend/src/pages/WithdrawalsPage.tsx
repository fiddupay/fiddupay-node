import { useToast } from '@/contexts/ToastContext'
import { publicAPI, withdrawalAPI, walletAPI } from '@/services/apiService'
import { WithdrawalFormSkeleton } from '@/components/layout/PageSkeletons'
import { useAuthStore } from '@/stores/authStore'
import styles from '@/styles/pages/WithdrawalsPage.module.css'
import CustomSelect from '@/components/ui/CustomSelect'
import { Withdrawal } from '@/types'
import { extractErrorMessage } from '@/utils/errorUtils'
import React, { useEffect, useState } from 'react'


import { useBalanceStore } from '@/stores/balanceStore'

const WithdrawalsPage: React.FC = () => {
    const { showToast } = useToast()
    const { user } = useAuthStore()
    const { balance, fetchBalance } = useBalanceStore()
    const settlementMode = user?.settlement_mode || 'managed'

    const walletBalances = balance?.balances || []
    const [withdrawals, setWithdrawals] = useState<Withdrawal[]>([])
    const [loading, setLoading] = useState(true)
    const [submitting, setSubmitting] = useState(false)
    const [refreshingId, setRefreshingId] = useState<string | null>(null)
    const [dateFrom, setDateFrom] = useState('')
    const [dateTo, setDateTo] = useState('')

    // Form state
    const [selectedCrypto, setSelectedCrypto] = useState('')
    const [destinationAddress, setDestinationAddress] = useState('')
    const [amount, setAmount] = useState('')
    const [showConfirm, setShowConfirm] = useState(false)
    const [balanceError, setBalanceError] = useState<string | null>(null)
    const [transactionPin, setTransactionPin] = useState('')
    const [supportedCurrencies, setSupportedCurrencies] = useState<any[]>([])
    const [configuredWallets, setConfiguredWallets] = useState<any[]>([])

    useEffect(() => {
        fetchData()
        fetchPrices()

        const interval = setInterval(() => {
            fetchWithdrawalsBackground()
            fetchPrices()
        }, 15000)
        return () => clearInterval(interval)
    }, [user?.sandbox_mode])

    const filteredWithdrawals = withdrawals.filter(w => {
        if (!dateFrom && !dateTo) return true
        const wDate = new Date(w.created_at).getTime()
        if (dateFrom && wDate < new Date(dateFrom).getTime()) return false
        if (dateTo) {
            const toDate = new Date(dateTo)
            toDate.setHours(23, 59, 59, 999)
            if (wDate > toDate.getTime()) return false
        }
        return true
    })

    const fetchPrices = async () => {
        try {
            const res = await publicAPI.getSupportedCurrencies()
            if (res.data?.currency_groups) {
                const flattened = Object.values(res.data.currency_groups).flat() as any[]
                setSupportedCurrencies(flattened)
            }
        } catch (err) {
            console.error('Failed to fetch prices', err)
        }
    }

    const fetchWithdrawalsBackground = async () => {
        try {
            const histRes = await withdrawalAPI.getHistory()
            setWithdrawals(Array.isArray(histRes.data) ? histRes.data : [])
        } catch {
            // silent fail on auto-refresh
        }
    }

    const handleManualRefresh = async (id: string, e: React.MouseEvent) => {
        e.stopPropagation()
        setRefreshingId(id)
        await fetchWithdrawalsBackground()
        setRefreshingId(null)
    }

    const handleCopy = (text: string) => {
        navigator.clipboard.writeText(text)
        showToast('Copied to clipboard!', 'success')
    }

    const getExplorerUrl = (cryptoType: string, hash: string, isSandbox: boolean) => {
        if (!hash) return null
        const up = cryptoType.toUpperCase()
        if (up.includes('SOL')) return `https://solscan.io/tx/${hash}${isSandbox ? '?cluster=devnet' : ''}`
        if (up.includes('ETH')) return `https://${isSandbox ? 'sepolia.' : ''}etherscan.io/tx/${hash}`
        if (up.includes('BINANCE') || up.includes('BNB') || up.includes('BSC')) return `https://${isSandbox ? 'testnet.' : ''}bscscan.com/tx/${hash}`
        if (up.includes('MATIC') || up.includes('POLYGON')) return `https://${isSandbox ? 'mumbai.' : ''}polygonscan.com/tx/${hash}`
        if (up.includes('ARB')) return `https://${isSandbox ? 'sepolia.' : ''}arbiscan.io/tx/${hash}`
        if (up.includes('BTC')) return `https://www.blockchain.com/btc${isSandbox ? '-testnet' : ''}/tx/${hash}`
        return null
    }

    const fetchData = async () => {
        try {
            setLoading(true)
            setBalanceError(null)

            // Fetch balances and history in parallel for better performance
            const [histRes] = await Promise.all([
                withdrawalAPI.getHistory().catch(() => ({ data: [] }))
            ])

            // Load balance through store
            await fetchBalance()

            // Fetch configured wallets
            const walletsRes = await walletAPI.getAll()
            const activeWallets = Array.isArray(walletsRes.data.wallets) 
                ? walletsRes.data.wallets.filter((w: any) => w.is_active) 
                : []
            setConfiguredWallets(activeWallets)

            if (activeWallets.length > 0 && !selectedCrypto) {
                // Prioritize wallets with balance, then just the first active one
                const withBalance = walletBalances.find(wb => parseFloat(wb.available_balance || '0') > 0)
                setSelectedCrypto(withBalance?.crypto_type || activeWallets[0].crypto_type)
            }

            setWithdrawals(Array.isArray(histRes.data) ? histRes.data : [])
        } catch (error) {
            console.error('Failed to load data:', error)
        } finally {
            setLoading(false)
        }
    }

    const combinedWallets = configuredWallets.map(cw => {
        const balanceEntry = walletBalances.find(wb => wb.crypto_type === cw.crypto_type)
        return {
            crypto_type: cw.crypto_type,
            available_balance: balanceEntry?.available_balance || '0',
            network: cw.network || cw.crypto_type.split('_')[1] || 'Mainnet'
        }
    })

    const selectedWallet = combinedWallets.find(w => w.crypto_type === selectedCrypto)
    const maxAmount = selectedWallet ? parseFloat(selectedWallet.available_balance || '0') : 0

    const handleMaxAmount = () => {
        setAmount(maxAmount.toString())
    }

    const handleSubmit = (e: React.FormEvent) => {
        e.preventDefault()
        if (!selectedCrypto) {
            showToast('Please select a wallet', 'error')
            return
        }
        if (!destinationAddress.trim()) {
            showToast('Please enter a destination address', 'error')
            return
        }
        if (!amount || parseFloat(amount) <= 0) {
            showToast('Please enter a valid amount', 'error')
            return
        }
        if (parseFloat(amount) > maxAmount) {
            showToast('Amount exceeds available balance', 'error')
            return
        }
        setShowConfirm(true)
    }

    const confirmWithdrawal = async () => {
        try {
            setSubmitting(true)
            await withdrawalAPI.create({
                crypto_type: selectedCrypto,
                amount: amount,
                destination_address: destinationAddress.trim(),
                pin: transactionPin
            })
            showToast('Withdrawal submitted successfully!', 'success')
            setShowConfirm(false)
            setDestinationAddress('')
            setAmount('')
            setTransactionPin('')
            await fetchData()
        } catch (error: any) {
            showToast(extractErrorMessage(error, 'Failed to create withdrawal'), 'error')
        } finally {
            setSubmitting(false)
        }
    }

    const statusBadge = (status: string) => {
        const map: Record<string, { color: string; bg: string; icon: string }> = {
            'PENDING': { color: '#d97706', bg: '#fef3c7', icon: 'fa-clock' },
            'APPROVED': { color: '#2563eb', bg: '#dbeafe', icon: 'fa-check' },
            'PROCESSING': { color: '#7c3aed', bg: '#ede9fe', icon: 'fa-spinner' },
            'COMPLETED': { color: '#059669', bg: '#d1fae5', icon: 'fa-check-circle' },
            'REJECTED': { color: '#dc2626', bg: '#fee2e2', icon: 'fa-times-circle' },
            'CANCELLED': { color: '#6b7280', bg: '#f3f4f6', icon: 'fa-ban' },
        }
        const s = map[status] || map['PENDING']
        return (
            <span className={styles.statusBadge} style={{ color: s.color, background: s.bg }}>
                <i className={`fas ${s.icon}`}></i> {status}
            </span>
        )
    }

    if (loading) {
        return <WithdrawalFormSkeleton />
    }

    return (
        <div className={styles.page}>
            <div className={styles.header}>
                <h1>Withdrawals</h1>
                <div className={styles.historyHeader}>
                    <h2>Withdrawal History</h2>
                    <div className={styles.historyFilters}>
                        <div className={styles.dateFilterGroup}>
                            <label>From</label>
                            <input 
                                type="date" 
                                value={dateFrom} 
                                onChange={e => setDateFrom(e.target.value)}
                                className={styles.dateInput}
                            />
                        </div>
                        <div className={styles.dateFilterGroup}>
                            <label>To</label>
                            <input 
                                type="date" 
                                value={dateTo} 
                                onChange={e => setDateTo(e.target.value)}
                                className={styles.dateInput}
                            />
                        </div>
                        {(dateFrom || dateTo) && (
                            <button className={styles.resetBtn} onClick={() => { setDateFrom(''); setDateTo(''); }}>
                                Reset
                            </button>
                        )}
                    </div>
                </div>
                <p>Withdraw funds from your managed wallets to any external address</p>
            </div>

            {settlementMode === 'forwarding' && (
                <div style={{
                    marginBottom: '1.5rem',
                    background: 'linear-gradient(135deg, #eff6ff, #f0f9ff)',
                    border: '1px solid #bfdbfe',
                    borderRadius: '0.75rem',
                    padding: '2rem',
                    textAlign: 'center',
                }}>
                    <i className="fas fa-exchange-alt" style={{ fontSize: '2rem', color: '#3b82f6', marginBottom: '0.75rem', display: 'block' }}></i>
                    <h3 style={{ margin: '0 0 0.5rem', color: '#1e40af', fontSize: '1.1rem' }}>Forwarding Mode Active</h3>
                    <p style={{ margin: 0, color: '#1e40af', fontSize: '0.9rem', maxWidth: '500px', marginInline: 'auto' }}>
                        Your payments are instantly forwarded to your external wallet (minus platform fees).
                        There is no balance to withdraw. Switch to <strong>Managed</strong> mode in Settings to hold funds and use withdrawals.
                    </p>
                </div>
            )}

            {balanceError && (
                <div style={{ marginBottom: '1.5rem', background: '#fef2f2', border: '1px solid #fecaca', borderRadius: '0.5rem', padding: '1rem', display: 'flex', gap: '0.75rem', color: '#991b1b' }}>
                    <i className="fas fa-exclamation-triangle" style={{ marginTop: '2px' }}></i>
                    <div>
                        <p style={{ fontWeight: 700 }}>Error Loading Wallets</p>
                        <p style={{ fontSize: '0.875rem' }}>{balanceError}</p>
                        <button onClick={fetchData} style={{ marginTop: '0.5rem', fontSize: '0.8rem', padding: '4px 12px', background: '#dc2626', color: '#fff', border: 'none', borderRadius: '6px', cursor: 'pointer' }}>
                            <i className="fas fa-redo" style={{ marginRight: '4px' }}></i> Retry
                        </button>
                    </div>
                </div>
            )}

            <div className={styles.layout}>
                {/* Withdrawal Form — only for managed/imported modes */}
                {settlementMode !== 'forwarding' && (
                    <div className={styles.formCard}>
                        <div className={styles.formCardHeader}>
                            <i className="fas fa-paper-plane"></i>
                            <h2>New Withdrawal</h2>
                        </div>

                        <form onSubmit={handleSubmit}>
                            {/* Wallet Selector */}
                            <div className={styles.formGroup}>
                                <CustomSelect
                                    label="Select Wallet"
                                    options={combinedWallets.map(w => {
                                        const currencyInfo = supportedCurrencies.find(c => c.crypto_type === w.crypto_type)
                                        const networkDisplay = currencyInfo?.network || w.network
                                        return {
                                            value: w.crypto_type,
                                            label: `${w.crypto_type.split('_')[0]} — Balance: ${parseFloat(w.available_balance).toFixed(6)} (${networkDisplay})`
                                        }
                                    })}
                                    value={selectedCrypto}
                                    onChange={(v) => setSelectedCrypto(v)}
                                    placeholder={combinedWallets.length === 0 ? "No wallets configured" : "Select a wallet"}
                                />
                            </div>

                            {/* Selected wallet info */}
                            {selectedWallet && (
                                <div className={styles.walletInfo}>
                                    <div className={styles.walletInfoRow}>
                                        <span>Available Balance</span>
                                        <strong style={{ color: '#059669' }}>{parseFloat(selectedWallet.available_balance || '0').toFixed(6)} {selectedCrypto.split('_')[0]}</strong>
                                    </div>
                                    <div className={styles.walletInfoRow}>
                                        <span>Network</span>
                                        <strong>{supportedCurrencies.find(c => c.crypto_type === selectedCrypto)?.network || selectedCrypto.split('_')[1] || 'Mainnet'}</strong>
                                    </div>
                                </div>
                            )}

                            {/* Destination Address */}
                            <div className={styles.formGroup}>
                                <label>Destination Address</label>
                                <input
                                    type="text"
                                    value={destinationAddress}
                                    onChange={e => setDestinationAddress(e.target.value)}
                                    placeholder="Enter external wallet address"
                                    className={styles.input}
                                />
                            </div>

                            {/* Amount */}
                            <div className={styles.formGroup}>
                                <label>Amount</label>
                                <div className={styles.amountRow}>
                                    <input
                                        type="number"
                                        step="any"
                                        value={amount}
                                        onChange={e => setAmount(e.target.value)}
                                        placeholder="0.00"
                                        className={styles.input}
                                    />
                                    <button type="button" className={styles.maxBtn} onClick={handleMaxAmount}>MAX</button>
                                </div>
                                {amount && !isNaN(parseFloat(amount)) && (
                                    <p style={{ marginTop: '0.5rem', fontSize: '0.85rem', color: '#059669', fontWeight: 600 }}>
                                        ≈ ${ (parseFloat(amount) * (supportedCurrencies.find(c => c.crypto_type === selectedCrypto)?.price_usd || 0)).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 }) } USD
                                    </p>
                                )}
                            </div>

                            {/* Fee Display */}
                            <div className={styles.feeDisplay}>
                                <div className={styles.feeRow}>
                                    <span>Withdrawal Fee</span>
                                    <span className={styles.freeLabel}><i className="fas fa-check-circle"></i> FREE</span>
                                </div>
                                <div className={styles.feeRow}>
                                    <span>You Receive</span>
                                    <strong>{amount ? parseFloat(amount).toFixed(6) : '0.000000'} {selectedCrypto.split('_')[0]}</strong>
                                </div>
                            </div>

                            <button type="submit" className={styles.submitBtn} disabled={submitting || combinedWallets.length === 0}>
                                <i className="fas fa-paper-plane"></i>
                                {submitting ? 'Submitting...' : 'Submit Withdrawal'}
                            </button>
                        </form>
                    </div>
                )}

                {/* Withdrawal History */}
                {settlementMode !== 'forwarding' && (
                    <div className={styles.historyCard}>
                        <div className={styles.formCardHeader} style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                            <div style={{ display: 'flex', alignItems: 'center', gap: '0.75rem' }}>
                                <i className="fas fa-history"></i>
                                <h2>Withdrawal History</h2>
                            </div>
                            <button
                                className={`${styles.refreshBtn} ${refreshingId === 'global' ? styles.spinIcon : ''}`}
                                onClick={(e) => handleManualRefresh('global', e)}
                                title="Refresh history"
                            >
                                <i className="fas fa-sync-alt"></i>
                            </button>
                        </div>

                        {filteredWithdrawals.length === 0 ? (
                            <div className={styles.emptyHistory}>
                                <i className="fas fa-inbox"></i>
                                <p>{withdrawals.length === 0 ? "No withdrawals yet" : "No withdrawals match your date filter."}</p>
                            </div>
                        ) : (
                            <div className={styles.historyList}>
                                {filteredWithdrawals.map((w) => (
                                    <div key={w.withdrawal_id} className={styles.historyItem}>
                                        <div className={styles.historyItemHeader}>
                                            <div>
                                            <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                                                {(w.crypto_type?.includes('SOL') || w.crypto_type?.includes('BUSD')) ? (
                                                <img 
                                                    src={w.crypto_type.includes('SOL') ? '/solana-sol-logo.png' : '/binance-usd-busd-logo.png'} 
                                                    alt={w.crypto_type}
                                                    style={{ width: '18px', height: '18px', borderRadius: '50%', objectFit: 'contain', display: 'block' }}
                                                />
                                                ) : null}
                                                <strong>{w.crypto_type}</strong>
                                            </div>
                                                <span className={styles.historyDate}>
                                                    {new Date(w.created_at).toLocaleDateString()} {new Date(w.created_at).toLocaleTimeString()}
                                                </span>
                                            </div>
                                            <div style={{ display: 'flex', alignItems: 'center', gap: '0.75rem' }}>
                                                <button
                                                    className={`${styles.refreshBtn} ${refreshingId === w.withdrawal_id ? styles.spinIcon : ''}`}
                                                    onClick={(e) => handleManualRefresh(w.withdrawal_id, e)}
                                                    title="Refresh status"
                                                >
                                                    <i className="fas fa-sync-alt"></i>
                                                </button>
                                                {statusBadge(w.status)}
                                            </div>
                                        </div>
                                        <div className={styles.historyDetails}>
                                            <div className={styles.historyDetailRow}>
                                                <span>Amount</span>
                                                <strong>{parseFloat(w.amount || '0').toFixed(6)}</strong>
                                            </div>
                                            <div className={styles.historyDetailRow}>
                                                <span>Fee</span>
                                                <span style={{ color: '#059669' }}>0 (Free)</span>
                                            </div>
                                            <div className={styles.historyDetailRow}>
                                                <span>To</span>
                                                <div className={styles.copyableText}>
                                                    <span className={styles.addressFull}>{w.destination_address}</span>
                                                    <button className={styles.iconBtn} onClick={() => handleCopy(w.destination_address)} title="Copy address">
                                                        <i className="far fa-copy"></i>
                                                    </button>
                                                </div>
                                            </div>
                                            {w.rejection_reason && (
                                                <div className={styles.historyDetailRow} style={{ color: '#dc2626', background: '#fef2f2', padding: '8px', borderRadius: '4px', marginTop: '8px' }}>
                                                    <span>Reason</span>
                                                    <strong>{w.rejection_reason}</strong>
                                                </div>
                                            )}
                                            {w.transaction_hash && (
                                                <div className={styles.historyDetailRow}>
                                                    <span>TX Hash</span>
                                                    <div className={styles.copyableText}>
                                                        <span className={styles.addressFull}>{w.transaction_hash}</span>
                                                        <div className={styles.actionIcons}>
                                                            <button className={styles.iconBtn} onClick={() => handleCopy(w.transaction_hash!)} title="Copy hash">
                                                                <i className="far fa-copy"></i>
                                                            </button>
                                                            {getExplorerUrl(w.crypto_type, w.transaction_hash, user?.sandbox_mode || false) && (
                                                                <a href={getExplorerUrl(w.crypto_type, w.transaction_hash, user?.sandbox_mode || false)!} target="_blank" rel="noopener noreferrer" className={styles.iconBtn} title="View on explorer">
                                                                    <i className="fas fa-external-link-alt"></i>
                                                                </a>
                                                            )}
                                                        </div>
                                                    </div>
                                                </div>
                                            )}
                                        </div>
                                    </div>
                                ))}
                            </div>
                        )}
                    </div>
                )}
            </div>

            {/* Confirmation Modal */}
            {showConfirm && (
                <div className={styles.modalOverlay} onClick={() => setShowConfirm(false)}>
                    <div className={styles.modalContent} onClick={e => e.stopPropagation()}>
                        <div className={styles.modalHeader}>
                            <h2>Confirm Withdrawal</h2>
                            <button className={styles.closeButton} onClick={() => setShowConfirm(false)}>
                                <i className="fas fa-times"></i>
                            </button>
                        </div>
                        <div className={styles.confirmDetails}>
                            <div className={styles.confirmRow}>
                                <span>Asset</span>
                                <strong>{selectedCrypto}</strong>
                            </div>
                            <div className={styles.confirmRow}>
                                <span>Amount</span>
                                <strong>{parseFloat(amount).toFixed(6)} {selectedCrypto.split('_')[0]}</strong>
                            </div>
                            <div className={styles.confirmRow}>
                                <span>Withdrawal Fee</span>
                                <strong style={{ color: '#059669' }}>Free</strong>
                            </div>
                            <div className={styles.confirmRow}>
                                <span>Destination</span>
                                <span style={{
                                    wordBreak: 'break-all',
                                    textAlign: 'right',
                                    maxWidth: '60%',
                                    fontSize: '0.9rem',
                                    fontFamily: 'monospace',
                                    fontWeight: 500,
                                    color: '#374151'
                                }}>{destinationAddress}</span>
                            </div>

                            <div style={{ marginTop: '1.5rem', borderTop: '1px solid #e5e7eb', paddingTop: '1.5rem' }}>
                                <label style={{ display: 'block', fontSize: '0.875rem', fontWeight: 600, color: '#374151', marginBottom: '0.5rem' }}>
                                    Merchant Transaction PIN
                                </label>
                                <input
                                    type="password"
                                    maxLength={4}
                                    value={transactionPin}
                                    onChange={e => setTransactionPin(e.target.value.replace(/\D/g, ''))}
                                    placeholder="Merchant Transaction PIN"
                                    style={{
                                        width: '100%',
                                        padding: '0.75rem',
                                        borderRadius: '0.5rem',
                                        border: '1px solid #d1d5db',
                                        textAlign: 'center',
                                        fontSize: '1.25rem',
                                        letterSpacing: '0.5rem',
                                        fontWeight: 700
                                    }}
                                />
                            </div>
                        </div>
                        <div className={styles.confirmWarning}>
                            <i className="fas fa-exclamation-triangle"></i>
                            <p>Please verify the destination address carefully. Transactions cannot be reversed.</p>
                        </div>
                        <div className={styles.modalActions}>
                            <button className={styles.cancelBtn} onClick={() => { setShowConfirm(false); setTransactionPin(''); }}>Cancel</button>
                            <button 
                                className={styles.confirmBtn} 
                                onClick={confirmWithdrawal} 
                                disabled={submitting || transactionPin.length !== 4}
                            >
                                {submitting ? 'Processing...' : 'Confirm & Send'}
                            </button>
                        </div>
                    </div>
                </div>
            )}
        </div>
    )
}

export default WithdrawalsPage
