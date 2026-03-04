import React, { useState, useEffect } from 'react'
import { withdrawalAPI, walletAPI } from '@/services/apiService'
import styles from '@/styles/pages/WithdrawalsPage.module.css'
import { useToast } from '@/contexts/ToastContext'
import { useAuthStore } from '@/stores/authStore'

interface WalletBalance {
    crypto_type: string
    network: string
    address: string
    is_active: boolean
    available_balance: string
    total_balance: string
    transaction_count: number
}

interface Withdrawal {
    withdrawal_id: string
    crypto_type: string
    amount: string
    destination_address: string
    status: string
    fee: string
    net_amount: string
    transaction_hash: string | null
    created_at: string
}

const WithdrawalsPage: React.FC = () => {
    const { showToast } = useToast()
    const { user } = useAuthStore()
    const settlementMode = user?.settlement_mode || 'managed'

    const [walletBalances, setWalletBalances] = useState<WalletBalance[]>([])
    const [withdrawals, setWithdrawals] = useState<Withdrawal[]>([])
    const [loading, setLoading] = useState(true)
    const [submitting, setSubmitting] = useState(false)

    // Form state
    const [selectedCrypto, setSelectedCrypto] = useState('')
    const [destinationAddress, setDestinationAddress] = useState('')
    const [amount, setAmount] = useState('')
    const [showConfirm, setShowConfirm] = useState(false)
    const [balanceError, setBalanceError] = useState<string | null>(null)

    useEffect(() => {
        fetchData()
    }, [user?.sandbox_mode])

    const fetchData = async () => {
        try {
            setLoading(true)
            setBalanceError(null)

            // Fetch balances — do NOT silently swallow errors
            let balances: WalletBalance[] = []
            try {
                const balRes = await walletAPI.getBalances()
                console.log('[WithdrawalsPage] balances API response:', balRes.data)
                balances = Array.isArray(balRes.data?.wallets) ? balRes.data.wallets : []
            } catch (balErr: any) {
                const errMsg = balErr.response?.data?.error || balErr.message || 'Unknown error'
                console.error('[WithdrawalsPage] Failed to fetch balances:', balErr.response?.status, errMsg)
                setBalanceError(`Failed to load wallets: ${errMsg}`)
            }

            setWalletBalances(balances)
            if (balances.length > 0 && !selectedCrypto) {
                setSelectedCrypto(balances[0].crypto_type)
            }

            // Fetch withdrawal history
            try {
                const histRes = await withdrawalAPI.getHistory()
                setWithdrawals(Array.isArray(histRes.data) ? histRes.data : [])
            } catch {
                setWithdrawals([])
            }
        } catch (error) {
            console.error('Failed to load data:', error)
        } finally {
            setLoading(false)
        }
    }

    const selectedWallet = walletBalances.find(w => w.crypto_type === selectedCrypto)
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
                destination_address: destinationAddress.trim()
            })
            showToast('Withdrawal submitted successfully!', 'success')
            setShowConfirm(false)
            setDestinationAddress('')
            setAmount('')
            await fetchData()
        } catch (error: any) {
            showToast(error.response?.data?.error || 'Failed to create withdrawal', 'error')
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
        return (
            <div className={styles.page}>
                <div className="flex items-center justify-center min-h-[400px]">
                    <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-primary"></div>
                </div>
            </div>
        )
    }

    return (
        <div className={styles.page}>
            <div className={styles.header}>
                <h1>Withdrawals</h1>
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
                                <label>Select Wallet</label>
                                <select
                                    value={selectedCrypto}
                                    onChange={e => setSelectedCrypto(e.target.value)}
                                    className={styles.select}
                                >
                                    {walletBalances.length === 0 && <option value="">No wallets available</option>}
                                    {walletBalances.map(w => (
                                        <option key={w.crypto_type} value={w.crypto_type}>
                                            {w.crypto_type} — Balance: {parseFloat(w.available_balance || '0').toFixed(6)} ({w.network})
                                        </option>
                                    ))}
                                </select>
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
                                        <strong>{selectedWallet.network}</strong>
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

                            <button type="submit" className={styles.submitBtn} disabled={submitting || walletBalances.length === 0}>
                                <i className="fas fa-paper-plane"></i>
                                {submitting ? 'Submitting...' : 'Submit Withdrawal'}
                            </button>
                        </form>
                    </div>
                )}

                {/* Withdrawal History */}
                <div className={styles.historyCard}>
                    <div className={styles.formCardHeader}>
                        <i className="fas fa-history"></i>
                        <h2>Withdrawal History</h2>
                    </div>

                    {withdrawals.length === 0 ? (
                        <div className={styles.emptyHistory}>
                            <i className="fas fa-inbox"></i>
                            <p>No withdrawals yet</p>
                        </div>
                    ) : (
                        <div className={styles.historyList}>
                            {withdrawals.map((w) => (
                                <div key={w.withdrawal_id} className={styles.historyItem}>
                                    <div className={styles.historyItemHeader}>
                                        <div>
                                            <strong>{w.crypto_type}</strong>
                                            <span className={styles.historyDate}>
                                                {new Date(w.created_at).toLocaleDateString()} {new Date(w.created_at).toLocaleTimeString()}
                                            </span>
                                        </div>
                                        {statusBadge(w.status)}
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
                                            <span className={styles.addressTruncate}>{w.destination_address}</span>
                                        </div>
                                        {w.transaction_hash && (
                                            <div className={styles.historyDetailRow}>
                                                <span>TX Hash</span>
                                                <span className={styles.addressTruncate}>{w.transaction_hash}</span>
                                            </div>
                                        )}
                                    </div>
                                </div>
                            ))}
                        </div>
                    )}
                </div>
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
                                <span className={styles.addressTruncate}>{destinationAddress}</span>
                            </div>
                        </div>
                        <div className={styles.confirmWarning}>
                            <i className="fas fa-exclamation-triangle"></i>
                            <p>Please verify the destination address carefully. Transactions cannot be reversed.</p>
                        </div>
                        <div className={styles.modalActions}>
                            <button className={styles.cancelBtn} onClick={() => setShowConfirm(false)}>Cancel</button>
                            <button className={styles.confirmBtn} onClick={confirmWithdrawal} disabled={submitting}>
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
