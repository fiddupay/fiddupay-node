import React from 'react'
import styles from './AnalyticsPage.module.css'

const AnalyticsPage: React.FC = () => {
    return (
        <div className={styles.page}>
            <div className={styles.header}>
                <h1>Analytics</h1>
                <p>Insights into your payments and revenue</p>
            </div>
            <div className={styles.content}>
                <div className={styles.emptyState}>
                    <i className="fas fa-chart-bar"></i>
                    <h3>Analytics coming soon</h3>
                    <p>Detailed charts and insights are on the way.</p>
                </div>
            </div>
        </div>
    )
}

export default AnalyticsPage
