import React, { useState, useEffect } from 'react'
import { apiService } from '../services/api'
import { Wallet, WalletConfig } from '../types'
import styles from './WalletsPage.module.css'

const WalletsPage: React.FC = () => {
  const [wallets, setWallets] = useState<Wallet[]>([])
  const [loading, setLoading] = useState(true)
  const [showConfigModal, setShowConfigModal] = useState(false)
  const [newWallet, setNewWallet] = useState<WalletConfig>({
    crypto_type: 'SOL',
    address: ''
  })

  const supportedCryptos = [
    { type: 'SOL', name: 'Solana', network: 'Solana' },
    { type: 'USDT_ETH', name: 'USDT', network: 'Ethereum' },
    { type: 'USDT_BSC', name: 'USDT', network: 'BSC' },
    { type: 'USDT_POLYGON', name: 'USDT', network: 'Polygon' },
    { type: 'USDT_ARBITRUM', name: 'USDT', network: 'Arbitrum' },
    { type: 'ETH', name: 'Ethereum', network: 'Ethereum' },
    { type: 'BNB', name: 'BNB', network: 'BSC' },
    { type: 'MATIC', name: 'MATIC', network: 'Polygon' },
    { type: 'ARB', name: 'Arbitrum', network: 'Arbitrum' }
  ]

  useEffect(() => {
    loadWallets()
  }, [])

  const loadWallets = async () => {
    try {
      setLoading(true)
      const walletsData = await apiService.getWallets()
      setWallets(walletsData)
    } catch (error) {
      console.error('Failed to load wallets:', error)
    } finally {
      setLoading(false)
    }
  }

  const handleConfigureWallet = async () => {
    try {
      // Validate wallet address format
      const address = newWallet.address.trim()
      if (!address) {
        alert('Please enter a wallet address')
        return
      }

      // Basic address format validation
      if (newWallet.crypto_type === 'SOL') {
        if (address.length < 32 || address.length > 44) {
          alert('Invalid Solana address format. Should be 32-44 characters.')
          return
        }
      } else if (newWallet.crypto_type.includes('ETH') || newWallet.crypto_type.includes('BSC') || 
                 newWallet.crypto_type.includes('POLYGON') || newWallet.crypto_type.includes('ARBITRUM') ||
                 newWallet.crypto_type === 'BNB' || newWallet.crypto_type === 'MATIC' || newWallet.crypto_type === 'ARB') {
        if (!address.startsWith('0x') || address.length !== 42) {
          alert('Invalid EVM address format. Should start with 0x and be 42 characters long.')
          return
        }
      }

      await apiService.configureWallet(newWallet)
      setShowConfigModal(false)
      setNewWallet({ crypto_type: 'SOL', address: '' })
      loadWallets()
      alert('Wallet configured successfully!')
    } catch (error) {
      console.error('Failed to configure wallet:', error)
      alert('Failed to configure wallet. Please try again.')
    }
  }

  const handleGenerateWallet = async (cryptoType: string) => {
    try {
      await apiService.generateWallet(cryptoType)
      loadWallets()
    } catch (error) {
      console.error('Failed to generate wallet:', error)
    }
  }

  const copyToClipboard = (text: string) => {
    navigator.clipboard.writeText(text)
  }

  if (loading) {
    return (
      <div className={styles.walletsPage}>
        <div className={styles.loading}>Loading wallets...</div>
      </div>
    )
  }

  return (
    <div className={styles.walletsPage}>
      <div className={styles.header}>
        <h1>Wallets</h1>
        <p>Configure your cryptocurrency wallet addresses for automatic forwarding</p>
        <button 
          className={styles.configureBtn}
          onClick={() => setShowConfigModal(true)}
        >
          Configure New Wallet
        </button>
      </div>

      <div className={styles.walletGrid}>
        {supportedCryptos.map((crypto) => {
          const wallet = wallets.find(w => w.crypto_type === crypto.type)
          
          return (
            <div key={crypto.type} className={styles.walletCard}>
              <div className={styles.walletHeader}>
                <h3>{crypto.name} ({crypto.network})</h3>
                <span className={wallet ? styles.statusActive : styles.statusInactive}>
                  {wallet ? 'Configured' : 'Not Configured'}
                </span>
              </div>
              
              {wallet ? (
                <div className={styles.walletAddress}>
                  <label>Wallet Address</label>
                  <div className={styles.addressInput}>
                    <input 
                      type="text" 
                      value={wallet.address} 
                      readOnly 
                    />
                    <button 
                      className={styles.copyBtn}
                      onClick={() => copyToClipboard(wallet.address)}
                    >
                      Copy
                    </button>
                  </div>
                  <div className={styles.configuredDate}>
                    Configured: {new Date(wallet.configured_at).toLocaleDateString()}
                  </div>
                </div>
              ) : (
                <div className={styles.walletActions}>
                  <button 
                    className={styles.generateBtn}
                    onClick={() => handleGenerateWallet(crypto.type)}
                  >
                    Generate Wallet
                  </button>
                  <p className={styles.actionNote}>
                    Generate a new wallet or configure your existing address
                  </p>
                </div>
              )}
            </div>
          )
        })}
      </div>

      {/* Configure Wallet Modal */}
      {showConfigModal && (
        <div className={styles.modal}>
          <div className={styles.modalContent}>
            <h2>Configure Wallet</h2>
            <div className={styles.formGroup}>
              <label>Cryptocurrency</label>
              <select 
                value={newWallet.crypto_type}
                onChange={(e) => setNewWallet({...newWallet, crypto_type: e.target.value})}
              >
                {supportedCryptos.map(crypto => (
                  <option key={crypto.type} value={crypto.type}>
                    {crypto.name} ({crypto.network})
                  </option>
                ))}
              </select>
            </div>
            <div className={styles.formGroup}>
              <label>Wallet Address</label>
              <input 
                type="text"
                value={newWallet.address}
                onChange={(e) => setNewWallet({...newWallet, address: e.target.value})}
                placeholder="Enter your wallet address"
              />
            </div>
            <div className={styles.modalActions}>
              <button onClick={() => setShowConfigModal(false)}>Cancel</button>
              <button onClick={handleConfigureWallet}>Configure</button>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}

export default WalletsPage
