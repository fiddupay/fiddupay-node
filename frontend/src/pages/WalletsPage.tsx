import React from 'react'
import styles from './WalletsPage.module.css'

const WalletsPage: React.FC = () => {
  return (
    <div className={styles.walletsPage}>
      <div className={styles.header}>
        <h1>Wallets</h1>
        <p>Configure your cryptocurrency wallet addresses for automatic forwarding</p>
      </div>

      <div className={styles.walletGrid}>
        <div className={styles.walletCard}>
          <div className={styles.walletHeader}>
            <h3>Solana (SOL)</h3>
            <span className={styles.statusActive}>Active</span>
          </div>
          <div className={styles.walletAddress}>
            <label>Wallet Address</label>
            <div className={styles.addressInput}>
              <input 
                type="text" 
                value="9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM" 
                readOnly 
              />
              <button className={styles.copyBtn}>Copy</button>
            </div>
          </div>
          <div className={styles.walletStats}>
            <div className={styles.stat}>
              <span>Total Received</span>
              <strong>1,234.56 SOL</strong>
            </div>
            <div className={styles.stat}>
              <span>Last Payment</span>
              <strong>2 hours ago</strong>
            </div>
          </div>
        </div>

        <div className={styles.walletCard}>
          <div className={styles.walletHeader}>
            <h3>USDT (Ethereum)</h3>
            <span className={styles.statusActive}>Active</span>
          </div>
          <div className={styles.walletAddress}>
            <label>Wallet Address</label>
            <div className={styles.addressInput}>
              <input 
                type="text" 
                value="0x742d35Cc6634C0532925a3b8D4C0532925a3b8D4" 
                readOnly 
              />
              <button className={styles.copyBtn}>Copy</button>
            </div>
          </div>
          <div className={styles.walletStats}>
            <div className={styles.stat}>
              <span>Total Received</span>
              <strong>$45,678.90</strong>
            </div>
            <div className={styles.stat}>
              <span>Last Payment</span>
              <strong>1 day ago</strong>
            </div>
          </div>
        </div>

        <div className={styles.walletCard}>
          <div className={styles.walletHeader}>
            <h3>USDT (BSC)</h3>
            <span className={styles.statusInactive}>Inactive</span>
          </div>
          <div className={styles.walletAddress}>
            <label>Wallet Address</label>
            <div className={styles.addressInput}>
              <input 
                type="text" 
                placeholder="Enter BSC wallet address" 
              />
              <button className={styles.saveBtn}>Save</button>
            </div>
          </div>
          <div className={styles.walletStats}>
            <div className={styles.stat}>
              <span>Total Received</span>
              <strong>$0.00</strong>
            </div>
            <div className={styles.stat}>
              <span>Last Payment</span>
              <strong>Never</strong>
            </div>
          </div>
        </div>
      </div>

      <div className={styles.infoBox}>
        <h3>Important Notes</h3>
        <ul>
          <li>All payments will be automatically forwarded to these wallet addresses</li>
          <li>Make sure you control these wallet addresses before saving</li>
          <li>Test with small amounts first to verify forwarding works correctly</li>
          <li>You can update wallet addresses at any time</li>
        </ul>
      </div>
    </div>
  )
}

export default WalletsPage
