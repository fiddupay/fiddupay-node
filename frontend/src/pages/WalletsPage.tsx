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

  const [applyToAllEvm, setApplyToAllEvm] = useState(false)

  const [supportedCryptos, setSupportedCryptos] = useState<any[]>([])

  // Helper: check if a network is EVM (not Solana)
  const isEvmNetwork = (networkName: string) => {
    return !networkName.toLowerCase().includes('solana')
  }

  useEffect(() => {
    fetchInitialData()
  }, [user?.sandbox_mode])

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
        showToast(`Please enter a ${settlementMode === 'imported' ? 'private key' : 'wallet address'}`, 'error')
        return
      }

      setRefreshing(true)

      const mode = settlementMode === 'imported' ? 'import' : 'address';

      const selectedNetwork = supportedCryptos.find(n => n.cryptos.some((c: any) => c.crypto_type === newWallet.crypto_type));
      const selectedIsEvm = selectedNetwork ? isEvmNetwork(selectedNetwork.name) : false;

      if (settlementMode === 'forwarding') {
        // Forwarding mode: apply per-network or all-EVM based on checkbox
        let cryptosToUpdate: string[];

        if (applyToAllEvm && selectedIsEvm) {
          // Apply to ALL EVM networks (but never Solana)
          const evmCryptos = supportedCryptos
            .filter(n => isEvmNetwork(n.name))
            .flatMap(n => n.cryptos.map((c: any) => c.crypto_type));
          cryptosToUpdate = evmCryptos;
        } else {
          // Apply only to the selected network's tokens
          cryptosToUpdate = selectedNetwork
            ? selectedNetwork.cryptos.map((c: any) => c.crypto_type)
            : [newWallet.crypto_type];
        }

        await Promise.all(cryptosToUpdate.map((ct: string) =>
          walletAPI.setup({
            crypto_type: ct,
            mode: 'address',
            address: address,
            is_active: true
          })
        ));

        await loadWallets()
        setShowConfigModal(false)
        setNewWallet({ crypto_type: 'SOL', address: '' })
        setApplyToAllEvm(false)
        showToast(
          applyToAllEvm && selectedIsEvm
            ? 'Forwarding address applied to all EVM networks!'
            : `Forwarding address configured for ${selectedNetwork?.name || 'network'}!`,
          'success'
        )
      } else {
        // Managed / Imported mode: existing behavior (apply to whole network)
        if (settlementMode === 'imported' && applyToAllEvm && selectedIsEvm) {
          await walletAPI.setup({
            crypto_type: newWallet.crypto_type,
            mode: 'import',
            private_key: address,
            is_active: true,
            enable_all_evm: true
          });
        } else {
          // Manual looping for other modes/scenarios where we don't have a single cross-network backend handler yet
          const cryptosToUpdate = selectedNetwork ? selectedNetwork.cryptos.map((c: any) => c.crypto_type) : [newWallet.crypto_type];

          await Promise.all(cryptosToUpdate.map((ct: string) =>
            walletAPI.setup({
              crypto_type: ct,
              mode: mode,
              address: settlementMode === 'imported' ? undefined : address,
              private_key: settlementMode === 'imported' ? address : undefined,
              is_active: true
            })
          ));
        }

        await loadWallets()
        setShowConfigModal(false)
        setNewWallet({ crypto_type: 'SOL', address: '' })
        setApplyToAllEvm(false)
        showToast('Wallet configured successfully for all assets on this network!', 'success')
      }
    } catch (error: any) {
      showToast(error.response?.data?.error?.message || error.response?.data?.error || 'Failed to configure wallet', 'error')
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
    if (!confirmModal.type || !confirmModal.networkName) return
    try {
      setRefreshing(true)

      // Find all crypto types on this network and revoke them all
      const network = supportedCryptos.find(n => n.name === confirmModal.networkName)
      const cryptoTypesToRevoke = network
        ? network.cryptos.map((c: any) => c.crypto_type)
        : [confirmModal.type]

      await Promise.all(cryptoTypesToRevoke.map((ct: string) => walletAPI.revoke(ct).catch(() => { })))

      await loadWallets()
      showToast(`${confirmModal.networkName} wallet revoked successfully`, 'success')
      setConfirmModal({ show: false, type: null, networkName: null, action: null })
    } catch (error: any) {
      showToast(error.response?.data?.error?.message || error.response?.data?.error || 'Failed to revoke wallet', 'error')
    } finally {
      setRefreshing(false)
    }
  }

  const handleToggleNetwork = async (networkName: string, isActive: boolean) => {
    const network = supportedCryptos.find(n => n.name === networkName);
    if (!network) return;

    // Optimistically update all wallets on this network in the local state
    const cryptosOnNetwork = network.cryptos.map((c: any) => c.crypto_type);
    const updatedWallets = wallets.map(w =>
      cryptosOnNetwork.includes(w.crypto_type) ? { ...w, is_active: isActive } : w
    );
    setWallets(updatedWallets);

    try {
      setRefreshing(true)

      // Update all crypto types on this network
      await Promise.all(cryptosOnNetwork.map(async (ct: string) => {
        const existingWallet = wallets.find(w => w.crypto_type === ct);
        if (existingWallet || settlementMode !== 'managed') {
          return walletAPI.setup({
            crypto_type: ct,
            mode: 'address',
            address: existingWallet?.address || '',
            is_active: isActive
          });
        }
      }));

      await loadWallets()
      showToast(`${isActive ? 'Enabled' : 'Disabled'} ${networkName} network successfully`, 'success')
    } catch (error: any) {
      // Revert on error
      await loadWallets()
      showToast(error.response?.data?.error?.message || error.response?.data?.error || 'Failed to toggle network', 'error')
    } finally {
      setRefreshing(false)
    }
  }

  const confirmGeneration = async () => {
    if (!confirmModal.type) return
    try {
      setRefreshing(true)
      // Generate wallet for the base crypto type
      const response = await walletAPI.setup({
        crypto_type: confirmModal.type,
        mode: 'generate',
        is_active: true,
        enable_all_evm: applyToAllEvm,
      })
      const { wallet, managed } = response.data

      // Replicate the generated address to all sibling tokens on this SINGLE network manually
      // (The backend also handles the cross-network `enable_all_evm` if the flag is passed)
      const generatedAddress = wallet?.config?.address || wallet?.address
      if (generatedAddress && confirmModal.networkName && !applyToAllEvm) {
        const network = supportedCryptos.find((n: any) => n.name === confirmModal.networkName)
        if (network) {
          const siblingCryptos = network.cryptos
            .map((c: any) => c.crypto_type)
            .filter((ct: string) => ct !== confirmModal.type)

          if (siblingCryptos.length > 0) {
            await Promise.all(siblingCryptos.map((ct: string) =>
              walletAPI.setup({
                crypto_type: ct,
                mode: 'address',
                address: generatedAddress,
                is_active: true
              }).catch(() => { }) // Don't fail if a sibling fails
            ))
          }
        }
      }

      // In managed mode, the backend does NOT return private_key — show only a success toast
      if (managed || settlementMode === 'managed' || !wallet.private_key) {
        await loadWallets()
        showToast(`${confirmModal.networkName} wallet generated successfully! Keys are securely managed by FidduPay.`, 'success')
        setConfirmModal({ show: false, type: null, networkName: null, action: null })
        return
      }

      // Non-managed mode: show private key reveal modal
      setGeneratedKey({
        address: wallet.config.address,
        privateKey: wallet.private_key,
        network: confirmModal.networkName || ''
      })
      await loadWallets()
      showToast('New wallet generated successfully!', 'success')
      setConfirmModal({ show: false, type: null, networkName: null, action: null })
    } catch (error: any) {
      showToast(error.response?.data?.error?.message || error.response?.data?.error || 'Failed to generate wallet', 'error')
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
        {settlementMode !== 'managed' && (
          <button className={styles.configureBtn} onClick={() => setShowConfigModal(true)}>
            <i className={`fas ${settlementMode === 'imported' ? 'fa-key' : 'fa-plus'} mr-2`}></i>
            {settlementMode === 'imported' ? 'Import Wallet' : 'Configure Networks'}
          </button>
        )}
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

                <div className="flex items-center gap-5">
                  <label className={styles.networkToggle} title={`${wallet?.is_active ? 'Disable' : 'Enable'} Network`}>
                    <input
                      type="checkbox"
                      checked={wallet?.is_active ?? false}
                      onChange={(e) => handleToggleNetwork(network.name, e.target.checked)}
                      disabled={!wallet}
                    />
                    <span className={styles.slider}></span>
                  </label>

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
                          {settlementMode === 'managed' && (
                            <button
                              className={styles.actionItem}
                              onClick={() => handleWalletAction(baseCryptoType, network.name, 'generate')}
                            >
                              <i className="fas fa-sync-alt"></i> Generate New
                            </button>
                          )}
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
                </div>
              </div>

              <div className={styles.walletContent}>
                {wallet ? (
                  <>
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
                    </div>

                    <div className={styles.statusInfo}>
                      <i className={`fas fa-check-circle ${styles.statusIcon} ${wallet.is_active ? styles.active : styles.inactive}`}></i>
                      <span>ActiveUpdated: {new Date(wallet.updated_at || wallet.configured_at || Date.now()).toLocaleDateString()}</span>
                    </div>
                  </>
                ) : (
                  <div className={styles.emptyState}>
                    <p className="text-sm">Not configured yet</p>
                  </div>
                )}
              </div>

              <div className="mt-6">
                {!wallet && (
                  <div className="flex gap-2">
                    {settlementMode === 'managed' ? (
                      <button className={styles.generateBtn} onClick={() => handleWalletAction(baseCryptoType, network.name, 'generate')}>
                        <i className="fas fa-magic"></i> Generate {network.name}
                      </button>
                    ) : (
                      <button className={styles.generateBtn} onClick={() => {
                        setNewWallet({ crypto_type: baseCryptoType, address: '' })
                        setShowConfigModal(true)
                      }}>
                        <i className={`fas ${settlementMode === 'imported' ? 'fa-key' : 'fa-edit'}`}></i>
                        Setup {network.name}
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
      {
        showConfigModal && (
          <div className={styles.modalOverlay} onClick={() => setShowConfigModal(false)}>
            <div className={styles.modalContent} onClick={e => e.stopPropagation()}>
              <div className={styles.modalHeader}>
                <h2>{settlementMode === 'imported' ? 'Import Wallet' : 'Configure Wallet'}</h2>
                <button className={styles.closeButton} onClick={() => setShowConfigModal(false)}>
                  <i className="fas fa-times text-xl"></i>
                </button>
              </div>
              <div className={styles.formGroup}>
                <label>Select Network</label>
                <select value={newWallet.crypto_type} onChange={(e) => setNewWallet({ ...newWallet, crypto_type: e.target.value })}>
                  {supportedCryptos.map(network => (
                    <option key={network.name} value={network.cryptos[0].crypto_type}>
                      {network.name}
                    </option>
                  ))}
                </select>
              </div>
              {settlementMode === 'managed' ? (
                <div className="py-6 text-center bg-gray-50 rounded-lg border border-gray-200">
                  <i className="fas fa-magic text-blue-500 text-3xl mb-3"></i>
                  <p className="text-gray-700 font-medium">Automatic Wallet Generation</p>
                  <p className="text-sm text-gray-500 px-6 mt-2">
                    In Managed mode, FidduPay creates and secures your wallets.
                    Simply click the <strong>Generate</strong> button on any network card in the main view.
                  </p>
                </div>
              ) : (
                <div className={styles.formGroup}>
                  <label>{settlementMode === 'imported' ? 'Private Key' : (settlementMode === 'forwarding' ? 'Forwarding Destination Address' : 'Wallet Address')}</label>
                  <input
                    type={settlementMode === 'imported' ? 'password' : 'text'}
                    value={newWallet.address}
                    onChange={(e) => setNewWallet({ ...newWallet, address: e.target.value })}
                    placeholder={settlementMode === 'imported' ? 'Enter private key' : (settlementMode === 'forwarding' ? 'Enter your payout address' : 'Enter 0x... or specific address')}
                    className={settlementMode === 'imported' ? styles.privateKeyInput : ''}
                    autoFocus
                  />
                  <p className={styles.inputHelper}>
                    {settlementMode === 'imported'
                      ? "Your private key will be encrypted and used to derive your wallet address."
                      : settlementMode === 'forwarding'
                        ? "Payments received will be automatically forwarded to this address."
                        : "Payments sent to this address will be detected automatically."}
                  </p>
                </div>
              )}
              {/* Apply to All EVM checkbox */}
              {(() => {
                const selectedNetwork = supportedCryptos.find(n => n.cryptos.some((c: any) => c.crypto_type === newWallet.crypto_type));
                if (selectedNetwork && isEvmNetwork(selectedNetwork.name)) {
                  return (
                    <div className={styles.formGroup} style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
                      <input
                        type="checkbox"
                        id="applyAllEvm"
                        checked={applyToAllEvm}
                        onChange={(e) => setApplyToAllEvm(e.target.checked)}
                        style={{ width: '18px', height: '18px' }}
                      />
                      <label htmlFor="applyAllEvm" style={{ cursor: 'pointer', fontWeight: 500, fontSize: '0.85rem' }}>
                        Apply to all EVM networks (Ethereum, BSC, Polygon, Arbitrum)
                      </label>
                    </div>
                  )
                }
                return null;
              })()}
              <div className={styles.modalActions}>
                <button className={styles.cancelBtn} onClick={() => setShowConfigModal(false)}>Cancel</button>
                <button className={styles.confirmBtn} onClick={handleConfigureWallet} disabled={refreshing}>
                  {refreshing ? 'Processing...' : (settlementMode === 'imported' ? 'Import & Encrypt Key' : 'Save Configuration')}
                </button>
              </div>
            </div>
          </div>
        )
      }

      {/* Revoke/Generate Confirmation Modal */}
      {
        confirmModal.show && (
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

                {confirmModal.action === 'generate' && confirmModal.networkName && isEvmNetwork(confirmModal.networkName) && (
                  <div className={styles.formGroup} style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', marginBottom: '1rem', background: '#f8fafc', padding: '10px', borderRadius: '8px', border: '1px solid #e2e8f0' }}>
                    <input
                      type="checkbox"
                      id="applyAllEvmGenerate"
                      checked={applyToAllEvm}
                      onChange={(e) => setApplyToAllEvm(e.target.checked)}
                      style={{ width: '18px', height: '18px' }}
                    />
                    <label htmlFor="applyAllEvmGenerate" style={{ cursor: 'pointer', fontWeight: 500, fontSize: '0.85rem' }}>
                      Generate identical key for all EVM networks
                    </label>
                  </div>
                )}

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
        )
      }

      {/* Redesigned Private Key Reveal Modal */}
      {
        generatedKey && (
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
        )
      }
    </div >
  )
}

export default WalletsPage
