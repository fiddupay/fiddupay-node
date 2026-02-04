import React from 'react'
import styles from '@/styles/pages/SettingsPage.module.css'

const SettingsPage: React.FC = () => {
    return (
        <div className={styles.page}>
            <div className={styles.header}>
                <h1>Settings</h1>
                <p>Manage your account settings and preferences</p>
            </div>
            <div className={styles.content}>
                <div className={styles.emptyState}>
                    <i className="fas fa-cog"></i>
                    <h3>Settings coming soon</h3>
                    <p>Account configuration options will be here.</p>
                </div>
            </div>
        </div>
    )
}

export default SettingsPage
