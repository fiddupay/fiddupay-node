import React, { useState } from 'react'
import styles from '@/styles/pages/ReportsPage.module.css'
import { merchantAPI } from '@/services/apiService'
import { useNotification } from '@/contexts/NotificationContext'

const ReportsPage: React.FC = () => {
    const { showNotification } = useNotification()
    const [loading, setLoading] = useState<'csv' | 'pdf' | null>(null)
    const [filters, setFilters] = useState({
        from_date: new Date(Date.now() - 30 * 24 * 60 * 60 * 1000).toISOString().split('T')[0],
        to_date: new Date().toISOString().split('T')[0],
        blockchain: 'ALL',
        status: 'ALL'
    })

    const handleExport = async (format: 'csv' | 'pdf') => {
        try {
            setLoading(format)
            const params = {
                from_date: filters.from_date ? new Date(filters.from_date).toISOString() : undefined,
                to_date: filters.to_date ? new Date(filters.to_date).toISOString() : undefined,
                blockchain: filters.blockchain !== 'ALL' ? filters.blockchain : undefined,
                status: filters.status !== 'ALL' ? filters.status : undefined,
                format
            }

            const response = await merchantAPI.exportAnalytics(params)
            
            // Handle Blob download
            const url = window.URL.createObjectURL(new Blob([response.data]))
            const link = document.createElement('a')
            link.href = url
            link.setAttribute('download', `fiddupay_report_${filters.from_date}_to_${filters.to_date}.${format}`)
            document.body.appendChild(link)
            link.click()
            link.remove()
            
            showNotification(`Successfully exported ${format.toUpperCase()} report`, 'success')
        } catch (error) {
            console.error('Export failed:', error)
            showNotification('Failed to export report. Please try again.', 'error')
        } finally {
            setLoading(null)
        }
    }

    return (
        <div className={styles.page}>
            <div className={styles.header}>
                <div className={styles.titleArea}>
                    <h1>Reporting Hub</h1>
                    <p>Generate transaction statements and tax-ready reports</p>
                </div>
            </div>

            <div className={styles.filtersCard}>
                <div className={styles.filterGrid}>
                    <div className={styles.filterGroup}>
                        <label>From Date</label>
                        <input 
                            type="date" 
                            value={filters.from_date}
                            onChange={(e) => setFilters({...filters, from_date: e.target.value})}
                        />
                    </div>
                    <div className={styles.filterGroup}>
                        <label>To Date</label>
                        <input 
                            type="date" 
                            value={filters.to_date}
                            onChange={(e) => setFilters({...filters, to_date: e.target.value})}
                        />
                    </div>
                    <div className={styles.filterGroup}>
                        <label>Network</label>
                        <select 
                            value={filters.blockchain}
                            onChange={(e) => setFilters({...filters, blockchain: e.target.value})}
                        >
                            <option value="ALL">All Networks</option>
                            <option value="SOLANA">Solana</option>
                            <option value="ETHEREUM">Ethereum</option>
                            <option value="BINANCE">Binance Smart Chain</option>
                            <option value="POLYGON">Polygon</option>
                            <option value="ARBITRUM">Arbitrum</option>
                            <option value="BITCOIN">Bitcoin</option>
                        </select>
                    </div>
                    <div className={styles.filterGroup}>
                        <label>Status</label>
                        <select 
                            value={filters.status}
                            onChange={(e) => setFilters({...filters, status: e.target.value})}
                        >
                            <option value="ALL">All Statuses</option>
                            <option value="CONFIRMED">Confirmed</option>
                            <option value="PENDING">Pending</option>
                            <option value="FAILED">Failed</option>
                            <option value="EXPIRED">Expired</option>
                            <option value="REFUNDED">Refunded</option>
                        </select>
                    </div>
                </div>
            </div>

            <div className={styles.actionsGrid}>
                <div className={styles.exportCard}>
                    <div className={styles.exportIcon}>
                        <i className="fas fa-file-csv"></i>
                    </div>
                    <div className={styles.exportInfo}>
                        <h3>Spreadsheet Export (CSV)</h3>
                        <p>Detailed transaction data for Excel, Google Sheets, or manual accounting.</p>
                    </div>
                    <button 
                        className={styles.exportBtn}
                        onClick={() => handleExport('csv')}
                        disabled={loading !== null}
                    >
                        {loading === 'csv' ? <i className="fas fa-spinner fa-spin"></i> : 'Download CSV'}
                    </button>
                </div>

                <div className={styles.exportCard}>
                    <div className={styles.exportIcon}>
                        <i className="fas fa-file-pdf"></i>
                    </div>
                    <div className={styles.exportInfo}>
                        <h3>Financial Statement (PDF)</h3>
                        <p>Professional, branded statement with summary statistics and transaction list.</p>
                    </div>
                    <button 
                        className={`${styles.exportBtn} ${styles.pdfBtn}`}
                        onClick={() => handleExport('pdf')}
                        disabled={loading !== null}
                    >
                        {loading === 'pdf' ? <i className="fas fa-spinner fa-spin"></i> : 'Download PDF'}
                    </button>
                </div>
            </div>

            <div className={styles.recentReports}>
                <h2>Report Generation Tips</h2>
                <ul>
                    <li><i className="fas fa-check-circle"></i> Filter by <strong>Confirmed</strong> status for accurate revenue calculations.</li>
                    <li><i className="fas fa-check-circle"></i> Use the <strong>Date Range</strong> to match your fiscal periods.</li>
                    <li><i className="fas fa-check-circle"></i> PDF reports are optimized for printing and official record keeping.</li>
                </ul>
            </div>
        </div>
    )
}

export default ReportsPage
