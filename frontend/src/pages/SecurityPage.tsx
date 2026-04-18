import React, { useState, useEffect } from 'react'
import { 
    MdNotificationsActive, 
    MdHistory, 
    MdSettings, 
    MdCheckCircle, 
    MdWarning, 
    MdError, 
    MdInfo
} from 'react-icons/md'
import { securityAPI, merchantAPI } from '@/services/apiService'
import { useAuthStore } from '@/stores/authStore'
import { useToast } from '@/contexts/ToastContext'
import { SecurityEvent, SecurityAlert } from '@/types'
import { SecurityHubSkeleton, TableSkeleton } from '@/components/layout/PageSkeletons'
import styles from '@/styles/pages/SecurityPage.module.css'

const formatDate = (dateString: string) => {
    try {
        return new Date(dateString).toLocaleString(undefined, { 
            month: 'short', 
            day: '2-digit', 
            hour: '2-digit', 
            minute: '2-digit', 
            second: '2-digit' 
        })
    } catch (e) {
        return dateString
    }
}

type TabType = 'alerts' | 'events' | 'config'

const SecurityPage: React.FC = () => {
    const { user, loadUser } = useAuthStore()
    const { showToast } = useToast()
    const [activeTab, setActiveTab] = useState<TabType>('alerts')
    const [loading, setLoading] = useState(false)
    const [alerts, setAlerts] = useState<SecurityAlert[]>([])
    const [events, setEvents] = useState<SecurityEvent[]>([])
    
    // Risk Config State
    const [lowBalanceEnabled, setLowBalanceEnabled] = useState(false)
    const [thresholdUsd, setThresholdUsd] = useState('50.00')

    useEffect(() => {
        fetchData()
    }, [activeTab])

    useEffect(() => {
        if (user) {
            setLowBalanceEnabled(user.low_balance_alerts_enabled || false)
            setThresholdUsd(user.low_balance_threshold_usd || '50.00')
        }
    }, [user])

    const fetchData = async () => {
        try {
            setLoading(true)
            if (activeTab === 'alerts') {
                const res = await securityAPI.getAlerts()
                setAlerts(res.data?.alerts || [])
            } else if (activeTab === 'events') {
                const res = await securityAPI.getEvents({ limit: 50 })
                setEvents(res.data?.events || [])
            }
        } catch (error) {
            console.error('Failed to fetch security data', error)
            showToast('Failed to load security data', 'error')
        } finally {
            setLoading(false)
        }
    }

    const handleAcknowledgeAlert = async (alertId: string) => {
        try {
            await securityAPI.acknowledgeAlert(alertId)
            setAlerts(prev => prev.filter(a => a.id !== alertId))
            showToast('Alert acknowledged', 'success')
        } catch (error) {
            showToast('Failed to acknowledge alert', 'error')
        }
    }

    const handleUpdateRiskConfig = async () => {
        try {
            setLoading(true)
            await merchantAPI.updateSettings({
                low_balance_alerts_enabled: lowBalanceEnabled,
                low_balance_threshold_usd: thresholdUsd
            })
            await loadUser(true)
            showToast('Risk configuration updated', 'success')
        } catch (error) {
            showToast('Failed to update configuration', 'error')
        } finally {
            setLoading(false)
        }
    }

    const getSeverityStyle = (severity: string) => {
        switch (severity.toLowerCase()) {
            case 'critical': return styles.severityCritical
            case 'high': return styles.severityHigh
            case 'medium': return styles.severityMedium
            case 'low': return styles.severityLow
            default: return ''
        }
    }

    const getAlertIcon = (severity: string) => {
        switch (severity.toLowerCase()) {
            case 'critical': return <MdError color="#b91c1c" />
            case 'high': return <MdWarning color="#ea580c" />
            case 'medium': return <MdInfo color="#ca8a04" />
            default: return <MdInfo color="#1e40af" />
        }
    }

    return (
        <div className={styles.securityPage}>
            <header className={styles.header}>
                <h1>Security Hub</h1>
                <p>Monitor real-time security events and manage system alerts.</p>
            </header>

            <div className={styles.tabs}>
                <button 
                    className={`${styles.tabBtn} ${activeTab === 'alerts' ? styles.activeTab : ''}`}
                    onClick={() => setActiveTab('alerts')}
                >
                    <MdNotificationsActive /> Active Alerts
                </button>
                <button 
                    className={`${styles.tabBtn} ${activeTab === 'events' ? styles.activeTab : ''}`}
                    onClick={() => setActiveTab('events')}
                >
                    <MdHistory /> Security Events
                </button>
                <button 
                    className={`${styles.tabBtn} ${activeTab === 'config' ? styles.activeTab : ''}`}
                    onClick={() => setActiveTab('config')}
                >
                    <MdSettings /> Risk Configuration
                </button>
            </div>

            <div className={styles.content}>
                {activeTab === 'alerts' && (
                    <div className={styles.alertsList}>
                        {loading ? (
                             <SecurityHubSkeleton />
                        ) : alerts.length > 0 ? (
                            alerts.map(alert => (
                                <div key={alert.id} className={styles.alertCard}>
                                    <div className={styles.alertInfo}>
                                        <div className={`${styles.alertIcon} ${getSeverityStyle(alert.severity)}`}>
                                            {getAlertIcon(alert.severity)}
                                        </div>
                                        <div className={styles.alertText}>
                                            <span className={`${styles.badge} ${getSeverityStyle(alert.severity)}`}>
                                                {alert.severity}
                                            </span>
                                            <h4>{alert.type.replace(/_/g, ' ')}</h4>
                                            <p>{alert.message}</p>
                                            <span className={styles.alertTime}>
                                                {formatDate(alert.created_at)}
                                            </span>
                                        </div>
                                    </div>
                                    <button 
                                        className={styles.actionBtn}
                                        onClick={() => handleAcknowledgeAlert(alert.id)}
                                    >
                                        Acknowledge
                                    </button>
                                </div>
                            ))
                        ) : (
                            <div className={styles.emptyState}>
                                <MdCheckCircle className={styles.emptyIcon} color="#10b981" />
                                <h3>No active alerts</h3>
                                <p>Everything looks secure. There are no pending security alerts.</p>
                            </div>
                        )}
                    </div>
                )}

                {activeTab === 'events' && (
                    <div className={styles.tableContainer}>
                        <table className={styles.table}>
                            <thead>
                                <tr>
                                    <th>Event</th>
                                    <th>Description</th>
                                    <th>IP Address</th>
                                    <th>Date</th>
                                </tr>
                            </thead>
                            <tbody>
                                {loading ? (
                                    <tr>
                                        <td colSpan={4}>
                                            <TableSkeleton rows={10} columns={4} />
                                        </td>
                                    </tr>
                                ) : (
                                    events.map(event => (
                                        <tr key={event.id}>
                                            <td>
                                                <span className="font-semibold">{event.action_type.replace(/_/g, ' ')}</span>
                                            </td>
                                            <td>{event.description}</td>
                                            <td><code className="bg-gray-100 px-1 rounded text-xs">{event.ip_address}</code></td>
                                            <td>{formatDate(event.created_at)}</td>
                                        </tr>
                                    ))
                                )}
                            </tbody>
                        </table>
                        {!loading && events.length === 0 && (
                            <div className={styles.emptyState}>No security events found.</div>
                        )}
                    </div>
                )}

                {activeTab === 'config' && (
                    <div className={styles.settingsSection}>
                        <div className={styles.formGroup}>
                            <label className="flex items-center justify-between pointer-cursor">
                                <div>
                                    <h4 className="m-0">Low Balance Notifications</h4>
                                    <p className="text-xs text-slate-500 m-0 font-normal">Receive an alert when your available balance drops below a threshold.</p>
                                </div>
                                <div className="ml-4">
                                    <input 
                                        type="checkbox" 
                                        className="w-5 h-5 accent-blue-600"
                                        checked={lowBalanceEnabled}
                                        onChange={(e) => setLowBalanceEnabled(e.target.checked)}
                                    />
                                </div>
                            </label>
                        </div>

                        <div className={styles.formGroup}>
                            <label>Notification Threshold (USD)</label>
                            <div className={styles.inputWrapper}>
                                <div className={styles.inputWithPrefix}>
                                    <span className={styles.prefix}>$</span>
                                    <input 
                                        type="number" 
                                        className={styles.input}
                                        value={thresholdUsd}
                                        onChange={(e) => setThresholdUsd(e.target.value)}
                                        placeholder="50.00"
                                    />
                                </div>
                                <button 
                                    className={styles.saveBtn}
                                    onClick={handleUpdateRiskConfig}
                                    disabled={loading}
                                >
                                    {loading ? 'Saving...' : 'Save Settings'}
                                </button>
                            </div>
                        </div>

                        <div className="mt-8 p-4 bg-blue-50 border border-blue-100 rounded-lg">
                            <h5 className="text-blue-800 font-semibold mb-2 flex items-center gap-2">
                                <MdInfo /> Proactive Security
                            </h5>
                            <ul className="text-sm text-blue-700 space-y-2 mb-0">
                                <li>Automatic IP blacklisting for repeated failed login attempts.</li>
                                <li>Session revocation when security credentials (API keys) are rotated.</li>
                                <li>Webhook signature verification for all inbound notifications.</li>
                            </ul>
                        </div>
                    </div>
                )}
            </div>
        </div>
    )
}

export default SecurityPage
