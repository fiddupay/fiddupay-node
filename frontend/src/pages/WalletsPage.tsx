import React, { useState, useEffect } from 'react'
import { walletAPI, publicAPI } from '@/services/apiService'
import { Wallet, WalletConfig } from '../types'
import styles from './WalletsPage.module.css'
import { Copy, Check, Plus, RefreshCw, X, ShieldCheck } from 'lucide-react'
import { useToast } from '@/contexts/ToastContext'

const WalletsPage: React.FC = () => {
  const [wallets, setWallets] = useState<Wallet[]>([])
  const [loading, setLoading] = useState(true)
  const [refreshing, setRefreshing] = useState(false)
  const [showConfigModal, setShowConfigModal] = useState(false)
  const { showToast } = useToast()

  const [newWallet, setNewWallet] = useState<WalletConfig>({
    crypto_type: 'SOL',
    address: ''
  })

  const [supportedCryptos, setSupportedCryptos] = useState<any[]>([])

  useEffect(() => {
    fetchInitialData()
  }, [])

  const fetchInitialData = async () => {
    try {
      setLoading(true)
      const [walletsRes, currenciesRes] = await Promise.all([
        walletAPI.getAll(),
        publicAPI.getSupportedCurrencies()
      ])

      setWallets(Array.isArray(walletsRes.data) ? walletsRes.data : [])

      const groups = currenciesRes.data.currency_groups
      const flattenedCurrencies = Object.values(groups).flat()
      setSupportedCryptos(flattenedCurrencies)

    } catch (error) {
      console.error('Failed to load data:', error)
      showToast('Failed to load wallet data', 'error')
    } finally {
      setLoading(false)
    }
  }

  const loadWallets = async () => {
    try {
      const walletsData = await walletAPI.getAll()
      setWallets(Array.isArray(walletsData.data) ? walletsData.data : [])
    } catch (error) {
      console.error('Failed to load wallets:', error)
    }
  }

  const handleConfigureWallet = async () => {
    try {
      const address = newWallet.address.trim()
      if (!address) {
        showToast('Please enter a wallet address', 'error')
        return
      }

      // Basic validation
      if (newWallet.crypto_type === 'SOL') {
        if (address.length < 32 || address.length > 44) {
          showToast('Invalid Solana address format', 'error')
          return
        }
      } else if (address.startsWith('0x') && address.length !== 42) {
        showToast('Invalid EVM address format', 'error')
        return
      }

      setRefreshing(true)
      await walletAPI.configure(newWallet)
      await loadWallets()
      setShowConfigModal(false)
      setNewWallet({ crypto_type: 'SOL', address: '' })
      showToast('Wallet configured successfully!', 'success')
    } catch (error: any) {
      console.error('Configuration failed:', error)
      showToast(error.response?.data?.error || 'Failed to configure wallet', 'error')
    } finally {
      setRefreshing(false)
    }
  }

  const [confirmModal, setConfirmModal] = useState<{ show: boolean; type: string | null }>({
    show: false,
    type: null
  })

  const handleGenerateWallet = (cryptoType: string) => {
    setConfirmModal({ show: true, type: cryptoType })
  }

  const confirmGeneration = async () => {
    if (!confirmModal.type) return

    try {
      setRefreshing(true)
      await walletAPI.generate(confirmModal.type)
      await loadWallets()
      showToast('New wallet generated successfully!', 'success')
      setConfirmModal({ show: false, type: null })
    } catch (error: any) {
      showToast(error.response?.data?.error || 'Failed to generate wallet', 'error')
    } finally {
      setRefreshing(false)
    }
  }

  const copyToClipboard = (text: string) => {
    navigator.clipboard.writeText(text)
    showToast('Address copied to clipboard', 'success')
  }

  if (loading && wallets.length === 0) {
    return (
      <div className={styles.walletsPage}>
        <div className="flex items-center justify-center min-h-[400px]">
          <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-primary"></div>
        </div>
      </div>
    )
  }

  return (
    <div className={styles.walletsPage}>
      <header className={styles.header}>
        <div>
          <h1>Wallet Management</h1>
          <p>Manage your deposit addresses for automatic payments</p>
        </div>
        <button
          className={styles.configureBtn}
          onClick={() => setShowConfigModal(true)}
        >
          <Plus size={20} />
          Add / Configure Wallet
        </button>
      </header>

      <div className={styles.walletGrid}>
        {supportedCryptos.map((crypto) => {
          const wallet = wallets?.find(w => w?.crypto_type === crypto.crypto_type)

          return (
            <div key={crypto.crypto_type} className={styles.walletCard}>
              <div className={styles.walletHeader}>
                <div className={styles.coinInfo}>
                  <div className={styles.coinIcon}>
                    <img src={crypto.iconUrl} alt={crypto.name} />
                  </div>
                  <div className={styles.coinDetails}>
                    <h3>{crypto.crypto_type.split('_')[0]}</h3>
                    <span className={styles.networkBadge}>{crypto.network}</span>
                  </div>
                </div>
                <div
                  className={wallet ? styles.statusActive : styles.statusInactive}
                  title={wallet ? 'Active' : 'Not Configured'}
                />
              </div>

              <div className={styles.walletContent}>
                {wallet ? (
                  <div className={styles.addressContainer}>
                    <label className={styles.addressLabel}>Deposit Address</label>
                    <div className={styles.addressRow}>
                      <span className={styles.addressText}>{wallet.address}</span>
                      <button
                        className={styles.copyBtn}
                        onClick={() => copyToClipboard(wallet.address)}
                        title="Copy Address"
                      >
                        <Copy size={16} />
                      </button>
                    </div>
                  </div>
                ) : (
                  <div className={styles.emptyState}>
                    <ShieldCheck className="text-gray-300 mx-auto mb-2" size={32} />
                    <p className="text-sm">No wallet configured</p>
                  </div>
                )}
              </div>

              <div className="mt-4 pt-4 border-t border-gray-100">
                {wallet ? (
                  <div className="flex justify-between items-center text-xs text-gray-500">
                    <span className="flex items-center gap-1">
                      <Check size={12} className="text-green-500" /> Verified
                    </span>
                    <span>{new Date(wallet.configured_at || Date.now()).toLocaleDateString()}</span>
                  </div>
                ) : (
                  <button
                    className={styles.generateBtn}
                    onClick={() => handleGenerateWallet(crypto.crypto_type)}
                  >
                    <RefreshCw size={16} />
                    Generate Address
                  </button>
                )}
              </div>
            </div>
          )
        })}
      </div>

      {/* Configuration Modal */}
      {showConfigModal && (
        <div className={styles.modalOverlay} onClick={() => setShowConfigModal(false)}>
          <div className={styles.modalContent} onClick={e => e.stopPropagation()}>
            <div className={styles.modalHeader}>
              <h2>Configure Wallet</h2>
              <button className={styles.closeButton} onClick={() => setShowConfigModal(false)}>
                <X size={24} />
              </button>
            </div>

            <div className={styles.formGroup}>
              <label>Select Cryptocurrency</label>
              <select
                value={newWallet.crypto_type}
                onChange={(e) => setNewWallet({ ...newWallet, crypto_type: e.target.value })}
              >
                {supportedCryptos.map(crypto => (
                  <option key={crypto.crypto_type} value={crypto.crypto_type}>
                    {crypto.crypto_type.split('_')[0]} on {crypto.network}
                  </option>
                ))}
              </select>
            </div>

            <div className={styles.formGroup}>
              <label>Wallet Address</label>
              <input
                type="text"
                value={newWallet.address}
                onChange={(e) => setNewWallet({ ...newWallet, address: e.target.value })}
                placeholder="Enter 0x... or specific address"
                autoFocus
              />
              <p className="text-xs text-gray-500 mt-1">
                Payments sent to this address will be detected automatically.
              </p>
            </div>

            <div className={styles.modalActions}>
              <button
                className={styles.cancelBtn}
                onClick={() => setShowConfigModal(false)}
              >
                Cancel
              </button>
              <button
                className={styles.confirmBtn}
                onClick={handleConfigureWallet}
                disabled={refreshing}
              >
                {refreshing ? 'Saving...' : 'Save Configuration'}
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Confirmation Modal */}
      {confirmModal.show && (
        <div className={styles.modalOverlay} onClick={() => setConfirmModal({ show: false, type: null })}>
          <div className={styles.modalContent} onClick={e => e.stopPropagation()} style={{ maxWidth: '400px' }}>
            <div className={styles.modalHeader}>
              <h2>Generate New Wallet?</h2>
              <button
                className={styles.closeButton}
                onClick={() => setConfirmModal({ show: false, type: null })}
              >
                <X size={24} />
              </button>
            </div>

            <div className="py-4 text-gray-600">
              <p className="mb-4">
                Are you sure you want to generate a new <strong>{confirmModal.type?.split('_')[0]}</strong> wallet address?
              </p>
              <div className="bg-yellow-50 border border-yellow-200 rounded-lg p-3 flex gap-3 text-sm text-yellow-800">
                <ShieldCheck size={20} className="shrink-0" />
                <p>This will create a dedicated deposit address for your merchant account. You can replace it later if needed.</p>
              </div>
            </div>

            <div className={styles.modalActions}>
              <button
                className={styles.cancelBtn}
                onClick={() => setConfirmModal({ show: false, type: null })}
              >
                Cancel
              </button>
              <button
                className={styles.confirmBtn}
                onClick={confirmGeneration}
                disabled={refreshing}
              >
                {refreshing ? 'Generating...' : 'Yes, Generate Wallet'}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}

export default WalletsPage
