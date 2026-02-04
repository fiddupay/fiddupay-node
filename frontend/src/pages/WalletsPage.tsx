import React, { useState, useEffect } from 'react'
import { walletAPI, publicAPI } from '@/services/apiService'
import { Wallet, WalletConfig } from '../types'
import styles from '@/styles/pages/WalletsPage.module.css'
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

      // Fix: Access wallets array from response object
      setWallets(Array.isArray(walletsRes.data.wallets) ? walletsRes.data.wallets : [])

      const groups = currenciesRes.data.currency_groups

      // Group by network instead of showing all currencies individually
      const networksMap: { [key: string]: any } = {}

      Object.values(groups).flat().forEach((crypto: any) => {
        const networkName = crypto.network.split(' (')[0] // Group by base network name
        if (!networksMap[networkName]) {
          networksMap[networkName] = {
            name: networkName,
            fullName: crypto.network,
            icon_url: crypto.icon_url,
            cryptos: []
          }
        }
        networksMap[networkName].cryptos.push(crypto)
      })

      setSupportedCryptos(Object.values(networksMap))

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
      // Fix: Access wallets array from response object
      setWallets(Array.isArray(walletsData.data.wallets) ? walletsData.data.wallets : [])
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
      if (newWallet.crypto_type === 'SOL' || newWallet.crypto_type === 'USDT_SOL') {
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

  const [confirmModal, setConfirmModal] = useState<{ show: boolean; type: string | null; networkName: string | null }>({
    show: false,
    type: null,
    networkName: null
  })

  const [generatedKey, setGeneratedKey] = useState<{ address: string; privateKey: string; network: string } | null>(null)

  const handleGenerateWallet = (cryptoType: string, networkName: string) => {
    setConfirmModal({ show: true, type: cryptoType, networkName })
  }

  const confirmGeneration = async () => {
    if (!confirmModal.type) return

    try {
      setRefreshing(true)
      const response = await walletAPI.generate(confirmModal.type)

      // Capture the generated data
      const { config, private_key } = response.data.wallet
      setGeneratedKey({
        address: config.address,
        privateKey: private_key,
        network: confirmModal.networkName || ''
      })

      await loadWallets()
      showToast('New wallet generated successfully!', 'success')
      setConfirmModal({ show: false, type: null, networkName: null })
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
          <p>Manage your deposit addresses per blockchain network</p>
        </div>
        <button
          className={styles.configureBtn}
          onClick={() => setShowConfigModal(true)}
        >
          <i className="fas fa-plus mr-2"></i>
          Add / Configure Wallet
        </button>
      </header>

      <div className={styles.walletGrid}>
        {supportedCryptos.map((network) => {
          // A network wallet is active if any of its currencies have an address configured
          // Since our backend now syncs them, checking one is enough, but we'll be robust.
          const wallet = wallets?.find(w => network.cryptos.some((c: any) => c.crypto_type === w.crypto_type))

          return (
            <div key={network.name} className={styles.walletCard}>
              <div className={styles.walletHeader}>
                <div className={styles.coinInfo}>
                  <div className={styles.coinIcon}>
                    <img src={network.icon_url} alt={network.name} />
                  </div>
                  <div className={styles.coinDetails}>
                    <h3>{network.name}</h3>
                    <div className="flex gap-1 mt-1">
                      {network.cryptos.map((c: any) => (
                        <span key={c.crypto_type} className={styles.networkBadge} title={c.network}>
                          {c.crypto_type.split('_')[0]}
                        </span>
                      ))}
                    </div>
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
                    <label className={styles.addressLabel}>Network Deposit Address</label>
                    <div className={styles.addressRow}>
                      <span className={styles.addressText}>{wallet.address}</span>
                      <button
                        className={styles.copyBtn}
                        onClick={() => copyToClipboard(wallet.address)}
                        title="Copy Address"
                      >
                        <i className="fas fa-copy"></i>
                      </button>
                    </div>
                    <p className="text-[10px] text-gray-400 mt-2">
                      Supports: {network.cryptos.map((c: any) => c.crypto_type.split('_')[0]).join(', ')}
                    </p>
                  </div>
                ) : (
                  <div className={styles.emptyState}>
                    <i className="fas fa-shield-alt text-gray-300 text-3xl mb-2 mx-auto block"></i>
                    <p className="text-sm">No wallet configured for {network.name}</p>
                  </div>
                )}
              </div>

              <div className="mt-4 pt-4 border-t border-gray-100">
                {wallet ? (
                  <div className="flex justify-between items-center text-xs text-gray-500">
                    <span className="flex items-center gap-1">
                      <i className="fas fa-check text-green-500 text-xs"></i> Verified
                    </span>
                    <span>{new Date(wallet.configured_at || Date.now()).toLocaleDateString()}</span>
                  </div>
                ) : (
                  <button
                    className={styles.generateBtn}
                    onClick={() => handleGenerateWallet(network.cryptos[0].crypto_type, network.name)}
                  >
                    <i className="fas fa-sync-alt"></i>
                    Generate {network.name} Address
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
                <i className="fas fa-times text-xl"></i>
              </button>
            </div>

            <div className={styles.formGroup}>
              <label>Select Cryptocurrency</label>
              <select
                value={newWallet.crypto_type}
                onChange={(e) => setNewWallet({ ...newWallet, crypto_type: e.target.value })}
              >
                {supportedCryptos.map(network => (
                  <optgroup key={network.name} label={network.name}>
                    {network.cryptos.map((crypto: any) => (
                      <option key={crypto.crypto_type} value={crypto.crypto_type}>
                        {crypto.crypto_type.split('_')[0]} on {crypto.network}
                      </option>
                    ))}
                  </optgroup>
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
        <div className={styles.modalOverlay} onClick={() => setConfirmModal({ show: false, type: null, networkName: null })}>
          <div className={styles.modalContent} onClick={e => e.stopPropagation()} style={{ maxWidth: '400px' }}>
            <div className={styles.modalHeader}>
              <h2>Generate New Wallet?</h2>
              <button
                className={styles.closeButton}
                onClick={() => setConfirmModal({ show: false, type: null, networkName: null })}
              >
                <i className="fas fa-times text-xl"></i>
              </button>
            </div>

            <div className="py-4 text-gray-600">
              <p className="mb-4">
                Are you sure you want to generate a new <strong>{confirmModal.type?.split('_')[0]}</strong> wallet address?
              </p>
              <div className="bg-yellow-50 border border-yellow-200 rounded-lg p-3 flex gap-3 text-sm text-yellow-800">
                <i className="fas fa-shield-alt text-xl shrink-0"></i>
                <p>This will create a dedicated deposit address for your merchant account. You can replace it later if needed.</p>
              </div>
            </div>

            <div className={styles.modalActions}>
              <button
                className={styles.cancelBtn}
                onClick={() => setConfirmModal({ show: false, type: null, networkName: null })}
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
      {/* Private Key Reveal Modal */}
      {generatedKey && (
        <div className={styles.modalOverlay} onClick={() => setGeneratedKey(null)}>
          <div className={styles.modalContent} onClick={e => e.stopPropagation()} style={{ maxWidth: '500px' }}>
            <div className={styles.modalHeader}>
              <h2 className="text-red-600">⚠️ Secure Your Private Key</h2>
              <button
                className={styles.closeButton}
                onClick={() => setGeneratedKey(null)}
              >
                <i className="fas fa-times text-xl"></i>
              </button>
            </div>

            <div className="py-4">
              <p className="text-sm text-gray-600 mb-4">
                Your new <strong>{generatedKey.network}</strong> wallet has been created.
                <span className="text-red-500 font-bold"> This private key will NEVER be shown again.</span>
              </p>

              <div className="bg-gray-50 p-4 rounded-lg border border-gray-200 mb-4">
                <div className="mb-3">
                  <label className="text-[10px] uppercase font-bold text-gray-400 block mb-1">Generated Address</label>
                  <code className="text-xs break-all block text-gray-800">{generatedKey.address}</code>
                </div>
                <div>
                  <label className="text-[10px] uppercase font-bold text-red-400 block mb-1">Private Key (Secret)</label>
                  <code className="text-xs break-all block text-red-600 font-bold bg-red-50 p-2 rounded border border-red-100">{generatedKey.privateKey}</code>
                </div>
              </div>

              <div className="flex gap-2 mb-4">
                <button
                  className="flex-1 py-2 bg-gray-100 hover:bg-gray-200 text-gray-700 rounded text-sm font-medium transition-colors flex items-center justify-center gap-2"
                  onClick={() => copyToClipboard(generatedKey.privateKey)}
                >
                  <i className="fas fa-copy"></i> Copy Key
                </button>
              </div>

              <div className="bg-red-50 border border-red-200 rounded-lg p-3 text-xs text-red-800">
                <strong>CRITICAL:</strong> If you lose this key, you will lose access to all funds sent to this address.
                We do not store this key and cannot recover it for you.
              </div>
            </div>

            <div className={styles.modalActions}>
              <button
                className={styles.confirmBtn}
                style={{ backgroundColor: '#10b981' }}
                onClick={() => setGeneratedKey(null)}
              >
                I have saved my private key
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}

export default WalletsPage
