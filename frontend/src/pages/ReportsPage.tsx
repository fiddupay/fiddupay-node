import React from 'react'
import styles from './ReportsPage.module.css'

const ReportsPage: React.FC = () => {
    return (
        <div className={styles.page}>
            <div className={styles.header}>
                <h1>Reports</h1>
                <p>Download transaction and tax reports</p>
            </div>
            <div className={styles.content}>
                <div className={styles.emptyState}>
                    <i className="fas fa-file-invoice"></i>
                    <h3>Reports coming soon</h3>
                    <p>Exportable reports will be available here.</p>
                </div>
            </div>
        </div>
    )
}

export default ReportsPage
