import React from 'react'
import styles from '@/styles/pages/BalancePage.module.css'

const BalancePage: React.FC = () => {
    return (
        <div className={styles.page}>
            <div className={styles.header}>
                <h1>Balance</h1>
                <p>View and manage your account balances</p>
            </div>
            <div className={styles.content}>
                <div className={styles.emptyState}>
                    <i className="fas fa-wallet"></i>
                    <h3>Balance details coming soon</h3>
                    <p>We are putting the finishing touches on the detailed balance view.</p>
                </div>
            </div>
        </div>
    )
}

export default BalancePage
