import React, { useState, useEffect } from 'react'
import { 
    MdNotificationsActive, 
    MdHistory, 
    MdSettings, 
    MdCheckCircle, 
    MdWarning, 
    MdError, 
    MdInfo,
    MdSecurity
} from 'react-icons/md'
import { securityAPI } from '@/services/apiService'
import { useToast } from '@/contexts/ToastContext'
import { SecurityEvent, SecurityAlert } from '@/types'
import { SecurityHubSkeleton, TableSkeleton } from '@/components/layout/PageSkeletons'
import styles from '@/styles/pages/SecurityPage.module.css'
import SEO from '@/components/ui/SEO'

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
    const { showToast } = useToast()
    const [activeTab, setActiveTab] = useState<TabType>('alerts')
    const [loading, setLoading] = useState(false)
    const [alerts, setAlerts] = useState<SecurityAlert[]>([])
    const [events, setEvents] = useState<SecurityEvent[]>([])

    useEffect(() => {
        fetchData()
    }, [])

    const fetchData = async () => {
        try {
            setLoading(true)
            // Fetch everything in parallel for better responsiveness
            const [alertsRes, eventsRes] = await Promise.all([
                securityAPI.getAlerts().catch(() => ({ data: { alerts: [] } })),
                securityAPI.getEvents({ limit: 50 }).catch(() => ({ data: { events: [] } }))
            ])

            setAlerts(alertsRes.data?.alerts || [])
            setEvents(eventsRes.data?.events || [])
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
            <SEO 
                title="Security Hub" 
                description="Monitor real-time security events, manage risk configurations, and view system alerts."
            />
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
                    <MdSettings /> Security Overview
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
                        <div className="p-6 bg-slate-900/50 border border-slate-800 rounded-xl">
                            <h3 className="text-lg font-bold mb-4 flex items-center gap-2">
                                <MdSecurity className="text-blue-400" /> Proactive System Security
                            </h3>
                            <p className="text-slate-400 text-sm mb-6">
                                FidduPay employs automated risk mitigation protocols to protect your institutional account.
                            </p>
                            
                            <div className="space-y-4">
                                <div className="flex items-start gap-4 p-4 bg-slate-800/30 rounded-lg border border-slate-800">
                                    <div className="w-8 h-8 rounded-full bg-blue-500/10 flex items-center justify-center text-blue-400 shrink-0">
                                        <MdCheckCircle size={18} />
                                    </div>
                                    <div>
                                        <h4 className="text-sm font-bold m-0">Brute-Force Protection</h4>
                                        <p className="text-xs text-slate-500 m-0 mt-1">Automatic IP blacklisting after 5 failed login attempts within 10 minutes.</p>
                                    </div>
                                </div>

                                <div className="flex items-start gap-4 p-4 bg-slate-800/30 rounded-lg border border-slate-800">
                                    <div className="w-8 h-8 rounded-full bg-blue-500/10 flex items-center justify-center text-blue-400 shrink-0">
                                        <MdCheckCircle size={18} />
                                    </div>
                                    <div>
                                        <h4 className="text-sm font-bold m-0">Session Integrity</h4>
                                        <p className="text-xs text-slate-500 m-0 mt-1">Immediate revocation of all active sessions when API keys are rotated.</p>
                                    </div>
                                </div>

                                <div className="flex items-start gap-4 p-4 bg-slate-800/30 rounded-lg border border-slate-800">
                                    <div className="w-8 h-8 rounded-full bg-blue-500/10 flex items-center justify-center text-blue-400 shrink-0">
                                        <MdCheckCircle size={18} />
                                    </div>
                                    <div>
                                        <h4 className="text-sm font-bold m-0">Webhook Verification</h4>
                                        <p className="text-xs text-slate-500 m-0 mt-1">Strict HMAC-SHA256 signature validation required for all inbound signals.</p>
                                    </div>
                                </div>
                            </div>

                            <div className="mt-8 p-4 bg-blue-500/5 border border-blue-500/20 rounded-lg">
                                <p className="text-xs text-blue-400 m-0 flex items-center gap-2">
                                    <MdInfo size={16} /> To configure threshold alerts or update security credentials, please visit the <strong>Account Settings</strong>.
                                </p>
                            </div>
                        </div>
                    </div>
                )}
            </div>
        </div>
    )
}

export default SecurityPage
