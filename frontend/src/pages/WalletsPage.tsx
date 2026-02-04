import React, { useState, useEffect } from 'react'
import { walletAPI, publicAPI } from '@/services/apiService'
import { Wallet, WalletConfig } from '../types'
import styles from '@/styles/pages/WalletsPage.module.css'
import { useToast } from '@/contexts/ToastContext'
import { useAuthStore } from '@/stores/authStore'
import { Link } from 'react-router-dom'

const WalletsPage: React.FC = () => {
  const [wallets, setWallets] = useState<Wallet[]>([])
  const [loading, setLoading] = useState(true)
  const [refreshing, setRefreshing] = useState(false)
  const [showConfigModal, setShowConfigModal] = useState(false)
  const { showToast } = useToast()
  const { user } = useAuthStore()

  const settlementMode = user?.settlement_mode || 'managed'

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

      setWallets(Array.isArray(walletsRes.data.wallets) ? walletsRes.data.wallets : [])

      const groups = currenciesRes.data.currency_groups
      const networksMap: { [key: string]: any } = {}

      Object.values(groups).flat().forEach((crypto: any) => {
        const networkName = crypto.network.split(' (')[0]
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

      setRefreshing(true)
      await walletAPI.configure(newWallet)
      await loadWallets()
      setShowConfigModal(false)
      setNewWallet({ crypto_type: 'SOL', address: '' })
      showToast('Wallet configured successfully!', 'success')
    } catch (error: any) {
      showToast(error.response?.data?.error || 'Failed to configure wallet', 'error')
    } finally {
      setRefreshing(false)
    }
  }

  const [activeMenu, setActiveMenu] = useState<string | null>(null)
  const [confirmModal, setConfirmModal] = useState<{ show: boolean; type: string | null; networkName: string | null; action: 'generate' | 'revoke' | null }>({
    show: false,
    type: null,
    networkName: null,
    action: null
  })

  const [generatedKey, setGeneratedKey] = useState<{ address: string; privateKey: string; network: string } | null>(null)

  const handleWalletAction = (cryptoType: string, networkName: string, action: 'generate' | 'revoke') => {
    setConfirmModal({ show: true, type: cryptoType, networkName, action })
    setActiveMenu(null)
  }

  const handleRevokeWallet = async () => {
    if (!confirmModal.type) return
    try {
      setRefreshing(true)
      await walletAPI.revoke(confirmModal.type)
      await loadWallets()
      showToast(`${confirmModal.networkName} wallet revoked successfully`, 'success')
      setConfirmModal({ show: false, type: null, networkName: null, action: null })
    } catch (error: any) {
      showToast(error.response?.data?.error || 'Failed to revoke wallet', 'error')
    } finally {
      setRefreshing(false)
    }
  }

  const confirmGeneration = async () => {
    if (!confirmModal.type) return
    try {
      setRefreshing(true)
      const response = await walletAPI.generate(confirmModal.type)
      const { config, private_key } = response.data.wallet
      setGeneratedKey({
        address: config.address,
        privateKey: private_key,
        network: confirmModal.networkName || ''
      })
      await loadWallets()
      showToast('New wallet generated successfully!', 'success')
      setConfirmModal({ show: false, type: null, networkName: null, action: null })
    } catch (error: any) {
      showToast(error.response?.data?.error || 'Failed to generate wallet', 'error')
    } finally {
      setRefreshing(false)
    }
  }

  const copyToClipboard = (text: string, label: string = 'Address') => {
    navigator.clipboard.writeText(text)
    showToast(`${label} copied to clipboard`, 'success')
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
          <p>Network-specific configuration for your deposit addresses</p>
        </div>
        <button className={styles.configureBtn} onClick={() => setShowConfigModal(true)}>
          <i className="fas fa-plus mr-2"></i>
          Add / Configure Wallet
        </button>
      </header>

      {/* Smart Mode Awareness Header */}
      <div className={styles.smartHeader}>
        <div className={styles.modeInfo}>
          <h3>
            <i className={`fas ${settlementMode === 'managed' ? 'fa-cloud' :
                settlementMode === 'imported' ? 'fa-key' : 'fa-share-square'
              }`}></i>
            Global Settlement Mode: <span className="text-blue-600">{settlementMode.toUpperCase()}</span>
          </h3>
          <p>
            {settlementMode === 'managed' && "FidduPay securely manages your keys. You can withdraw anytime."}
            {settlementMode === 'imported' && "Using your own private keys. You have full custody."}
            {settlementMode === 'forwarding' && "Funds are auto-forwards to your destination addresses."}
          </p>
        </div>
        <Link to="/settings" className={styles.changeModeLink}>
          Change in Settings <i className="fas fa-external-link-alt"></i>
        </Link>
      </div>

      <div className={styles.walletGrid}>
        {supportedCryptos.map((network) => {
          const wallet = wallets?.find(w => network.cryptos.some((c: any) => c.crypto_type === w.crypto_type && w.address !== ""))
          const baseCryptoType = network.cryptos[0].crypto_type

          return (
            <div key={network.name} className={styles.walletCard} style={{ position: 'relative' }}>
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

                {wallet && (
                  <div className={styles.walletActions}>
                    <button
                      className={styles.actionMenuToggle}
                      onClick={() => setActiveMenu(activeMenu === network.name ? null : network.name)}
                    >
                      <i className="fas fa-ellipsis-v"></i>
                    </button>
                    {activeMenu === network.name && (
                      <div className={styles.actionDropdown}>
                        <button
                          className={styles.actionItem}
                          onClick={() => handleWalletAction(baseCryptoType, network.name, 'generate')}
                        >
                          <i className="fas fa-sync-alt"></i> Generate New
                        </button>
                        <button
                          className={`${styles.actionItem} ${styles.danger}`}
                          onClick={() => handleWalletAction(baseCryptoType, network.name, 'revoke')}
                        >
                          <i className="fas fa-trash-alt"></i> Revoke / Remove
                        </button>
                      </div>
                    )}
                  </div>
                )}

                <div
                  className={wallet ? styles.statusActive : styles.statusInactive}
                  title={wallet ? 'Active' : 'Not Configured'}
                />
              </div>

              <div className={styles.walletContent}>
                {wallet ? (
                  <div className={styles.addressContainer}>
                    <label className={styles.addressLabel}>
                      {settlementMode === 'forwarding' ? 'Forwarding Payout Address' : 'Network Deposit Address'}
                    </label>
                    <div className={styles.addressRow}>
                      <span className={styles.addressText}>{wallet.address}</span>
                      <button className={styles.copyBtn} onClick={() => copyToClipboard(wallet.address)} title="Copy Address">
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
                    <p className="text-sm">
                      {settlementMode === 'managed' ? "No wallet generated yet" :
                        settlementMode === 'imported' ? "Provide private key to start" :
                          "Provide destination address"}
                    </p>
                  </div>
                )}
              </div>

              <div className="mt-4 pt-4 border-t border-gray-100">
                {wallet ? (
                  <div className="flex justify-between items-center text-xs text-gray-500">
                    <span className="flex items-center gap-1">
                      <i className="fas fa-check-circle text-green-500"></i> Active
                    </span>
                    <span>Last configured: {new Date(wallet.updated_at || wallet.configured_at || Date.now()).toLocaleDateString()}</span>
                  </div>
                ) : (
                  <div className="flex gap-2">
                    {settlementMode === 'managed' && (
                      <button className={styles.generateBtn} onClick={() => handleWalletAction(baseCryptoType, network.name, 'generate')}>
                        <i className="fas fa-magic"></i> Generate {network.name}
                      </button>
                    )}
                    {settlementMode !== 'managed' && (
                      <button className={styles.generateBtn} onClick={() => setShowConfigModal(true)}>
                        <i className="fas fa-edit"></i> Configure {network.name}
                      </button>
                    )}
                  </div>
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
              <select value={newWallet.crypto_type} onChange={(e) => setNewWallet({ ...newWallet, crypto_type: e.target.value })}>
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
              <input type="text" value={newWallet.address} onChange={(e) => setNewWallet({ ...newWallet, address: e.target.value })} placeholder="Enter 0x... or specific address" autoFocus />
              <p className="text-xs text-gray-500 mt-1">Payments sent to this address will be detected automatically.</p>
            </div>
            <div className={styles.modalActions}>
              <button className={styles.cancelBtn} onClick={() => setShowConfigModal(false)}>Cancel</button>
              <button className={styles.confirmBtn} onClick={handleConfigureWallet} disabled={refreshing}>{refreshing ? 'Saving...' : 'Save Configuration'}</button>
            </div>
          </div>
        </div>
      )}

      {/* Revoke/Generate Confirmation Modal */}
      {confirmModal.show && (
        <div className={styles.modalOverlay} onClick={() => setConfirmModal({ show: false, type: null, networkName: null, action: null })}>
          <div className={styles.modalContent} onClick={e => e.stopPropagation()} style={{ maxWidth: '400px' }}>
            <div className={styles.modalHeader}>
              <h2>{confirmModal.action === 'revoke' ? 'Revoke Wallet?' : 'Generate New?'}</h2>
              <button className={styles.closeButton} onClick={() => setConfirmModal({ show: false, type: null, networkName: null, action: null })}>
                <i className="fas fa-times text-xl"></i>
              </button>
            </div>
            <div className="py-4 text-gray-600">
              <p className="mb-4">
                {confirmModal.action === 'revoke'
                  ? `Are you sure you want to revoke the ${confirmModal.networkName} wallet configuration? You will need to re-configure it to receive payments.`
                  : `Are you sure you want to generate a new ${confirmModal.networkName} wallet address? This will replace your current one.`}
              </p>
              <div className={`bg-${confirmModal.action === 'revoke' ? 'red' : 'yellow'}-50 border border-${confirmModal.action === 'revoke' ? 'red' : 'yellow'}-200 rounded-lg p-3 flex gap-3 text-sm text-${confirmModal.action === 'revoke' ? 'red' : 'yellow'}-800`}>
                <i className="fas fa-exclamation-triangle mt-1"></i>
                <p>This action cannot be undone. Make sure you don't have pending funds.</p>
              </div>
            </div>
            <div className={styles.modalActions}>
              <button className={styles.cancelBtn} onClick={() => setConfirmModal({ show: false, type: null, networkName: null, action: null })}>Cancel</button>
              <button
                className={styles.confirmBtn}
                style={{ backgroundColor: confirmModal.action === 'revoke' ? '#ef4444' : '#2563eb' }}
                onClick={confirmModal.action === 'revoke' ? handleRevokeWallet : confirmGeneration}
                disabled={refreshing}
              >
                {refreshing ? 'Processing...' : `Yes, ${confirmModal.action === 'revoke' ? 'Revoke' : 'Generate'}`}
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Redesigned Private Key Reveal Modal */}
      {generatedKey && (
        <div className={styles.modalOverlay}>
          <div className={`${styles.modalContent} ${styles.premiumModal}`} onClick={e => e.stopPropagation()}>
            <div className={styles.modalContentRedesign}>
              <div className={styles.securityHeader}>
                <div className={styles.securityIcon}>
                  <i className="fas fa-shield-alt"></i>
                </div>
                <h2>Secure Your Wallet</h2>
                <p>Your new {generatedKey.network} wallet is ready.</p>
              </div>

              <div className={styles.keySection}>
                <div className={styles.keyField}>
                  <label className={styles.keyLabel}>
                    <i className="fas fa-link"></i> Public Address
                  </label>
                  <code className={styles.keyValue}>{generatedKey.address}</code>
                </div>
                <div className={styles.keyField}>
                  <label className={styles.keyLabel}>
                    <i className="fas fa-key"></i> Private Key (Secret)
                  </label>
                  <code className={`${styles.keyValue} ${styles.secret}`}>{generatedKey.privateKey}</code>
                </div>
              </div>

              <button className={styles.copyKeyBtn} onClick={() => copyToClipboard(generatedKey.privateKey, 'Private Key')}>
                <i className="fas fa-copy"></i> Copy Private Key
              </button>

              <div className="mt-6">
                <div className={styles.warningBox}>
                  <i className="fas fa-exclamation-circle"></i>
                  <p>
                    <strong>WARNING:</strong> This key is NEVER stored. If you close this window without saving it,
                    <strong> any funds sent to this address will be lost forever.</strong>
                  </p>
                </div>
              </div>
            </div>

            <div className={styles.modalFooterRedesign}>
              <button className={styles.finishBtn} onClick={() => setGeneratedKey(null)}>
                I Have Safely Stored My Key
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}

export default WalletsPage
