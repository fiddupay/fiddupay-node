import React, { useState, useEffect } from 'react'
import { useSearchParams } from 'react-router-dom'
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
    const [searchParams, setSearchParams] = useSearchParams()
    
    // Sync active tab with URL search parameter
    const activeTab = (searchParams.get('tab') as TabType) || 'alerts'
    const setActiveTab = (tab: TabType) => {
        setSearchParams({ tab })
    }

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
                        <div className="p-8 bg-gradient-to-br from-[#0f172a] to-[#1e293b] border border-slate-700/50 rounded-2xl shadow-xl">
                            <div className="flex items-center gap-4 mb-8">
                                <div className="w-12 h-12 rounded-xl bg-blue-500/20 flex items-center justify-center border border-blue-500/30">
                                    <MdSecurity className="text-blue-400 text-2xl" />
                                </div>
                                <div>
                                    <h3 className="text-xl font-bold text-slate-100 m-0">Proactive System Security</h3>
                                    <p className="text-slate-400 text-sm mt-1 mb-0">
                                        FidduPay employs automated risk mitigation protocols to protect your institutional account.
                                    </p>
                                </div>
                            </div>
                            
                            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                                {/* Brute-Force Card */}
                                <div className="group relative overflow-hidden bg-slate-900/50 rounded-xl border border-slate-700 hover:border-blue-500/50 transition-all duration-300">
                                    <div className="absolute inset-0 bg-gradient-to-br from-blue-500/5 to-transparent opacity-0 group-hover:opacity-100 transition-opacity"></div>
                                    <div className="p-6 relative z-10">
                                        <div className="flex items-center justify-between mb-4">
                                            <div className="w-10 h-10 rounded-full bg-slate-800 flex items-center justify-center text-emerald-400 ring-1 ring-emerald-400/20 shadow-[0_0_15px_rgba(52,211,153,0.1)]">
                                                <MdCheckCircle size={20} />
                                            </div>
                                            <span className="text-[10px] uppercase tracking-wider font-bold text-emerald-400 bg-emerald-400/10 px-2 py-1 rounded-md">Active</span>
                                        </div>
                                        <h4 className="text-base font-bold text-slate-200 m-0 mb-2">Brute-Force Shield</h4>
                                        <p className="text-sm text-slate-400 m-0">Automatic IP blacklisting after 5 failed login attempts within 10 minutes.</p>
                                    </div>
                                </div>

                                {/* Session Integrity Card */}
                                <div className="group relative overflow-hidden bg-slate-900/50 rounded-xl border border-slate-700 hover:border-purple-500/50 transition-all duration-300">
                                    <div className="absolute inset-0 bg-gradient-to-br from-purple-500/5 to-transparent opacity-0 group-hover:opacity-100 transition-opacity"></div>
                                    <div className="p-6 relative z-10">
                                        <div className="flex items-center justify-between mb-4">
                                            <div className="w-10 h-10 rounded-full bg-slate-800 flex items-center justify-center text-emerald-400 ring-1 ring-emerald-400/20 shadow-[0_0_15px_rgba(52,211,153,0.1)]">
                                                <MdCheckCircle size={20} />
                                            </div>
                                            <span className="text-[10px] uppercase tracking-wider font-bold text-emerald-400 bg-emerald-400/10 px-2 py-1 rounded-md">Active</span>
                                        </div>
                                        <h4 className="text-base font-bold text-slate-200 m-0 mb-2">Session Integrity</h4>
                                        <p className="text-sm text-slate-400 m-0">Immediate revocation of all active sessions when API keys or passwords are rotated.</p>
                                    </div>
                                </div>

                                {/* Webhook Verification Card */}
                                <div className="group relative overflow-hidden bg-slate-900/50 rounded-xl border border-slate-700 hover:border-cyan-500/50 transition-all duration-300">
                                    <div className="absolute inset-0 bg-gradient-to-br from-cyan-500/5 to-transparent opacity-0 group-hover:opacity-100 transition-opacity"></div>
                                    <div className="p-6 relative z-10">
                                        <div className="flex items-center justify-between mb-4">
                                            <div className="w-10 h-10 rounded-full bg-slate-800 flex items-center justify-center text-emerald-400 ring-1 ring-emerald-400/20 shadow-[0_0_15px_rgba(52,211,153,0.1)]">
                                                <MdCheckCircle size={20} />
                                            </div>
                                            <span className="text-[10px] uppercase tracking-wider font-bold text-emerald-400 bg-emerald-400/10 px-2 py-1 rounded-md">Active</span>
                                        </div>
                                        <h4 className="text-base font-bold text-slate-200 m-0 mb-2">Payload Verification</h4>
                                        <p className="text-sm text-slate-400 m-0">Strict HMAC-SHA256 signature validation required for all inbound webhook signals.</p>
                                    </div>
                                </div>
                            </div>

                            <div className="mt-8 flex items-center justify-between p-4 bg-slate-800/50 border border-slate-700 rounded-xl">
                                <div className="flex items-center gap-3 text-sm text-slate-300">
                                    <div className="w-8 h-8 rounded-full bg-blue-500/10 flex items-center justify-center text-blue-400">
                                        <MdInfo size={16} />
                                    </div>
                                    <span>To configure threshold alerts or update security credentials, visit your Account Settings.</span>
                                </div>
                                <a href="/app/settings?tab=security" className="px-4 py-2 bg-blue-600 hover:bg-blue-500 text-white text-sm font-bold rounded-lg transition-colors shadow-[0_0_15px_rgba(37,99,235,0.3)] hover:shadow-[0_0_20px_rgba(37,99,235,0.5)]">
                                    Go to Settings
                                </a>
                            </div>
                        </div>
                    </div>
                )}
            </div>
        </div>
    )
}

export default SecurityPage
