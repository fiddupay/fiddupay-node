import React from 'react'
import styles from '@/styles/pages/WithdrawalsPage.module.css'

const WithdrawalsPage: React.FC = () => {
    return (
        <div className={styles.page}>
            <div className={styles.header}>
                <h1>Withdrawals</h1>
                <p>Manage your crypto withdrawals</p>
            </div>
            <div className={styles.content}>
                <div className={styles.emptyState}>
                    <i className="fas fa-sign-out-alt"></i>
                    <h3>Withdrawals coming soon</h3>
                    <p>You'll be able to manage withdrawals here.</p>
                </div>
            </div>
        </div>
    )
}

export default WithdrawalsPage
