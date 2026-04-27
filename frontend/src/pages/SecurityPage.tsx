import React, { useEffect } from 'react'
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
import { useDataStore } from '@/stores/dataStore'
import { useToast } from '@/contexts/ToastContext'
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

type TabType = 'alerts' | 'events' | 'overview'

const SecurityPage: React.FC = () => {
    const { showToast } = useToast()
    const [searchParams, setSearchParams] = useSearchParams()
    
    // Sync active tab with URL search parameter
    const activeTab = (searchParams.get('tab') as TabType) || 'alerts'
    const setActiveTab = (tab: TabType) => {
        setSearchParams({ tab })
    }

    // Use global dataStore for alerts and events
    const {
        securityAlerts: alertsCache,
        securityEvents: eventsCache,
        fetchSecurityAlerts,
        fetchSecurityEvents,
        setSecurityAlerts,
    } = useDataStore()
    const alerts = alertsCache.data || []
    const events = eventsCache.data || []
    const loading = (alertsCache.loading && alerts.length === 0) || (eventsCache.loading && events.length === 0)

    useEffect(() => {
        fetchSecurityAlerts()
        fetchSecurityEvents()
    }, [])

    const handleAcknowledgeAlert = async (alertId: string) => {
        try {
            await securityAPI.acknowledgeAlert(alertId)
            setSecurityAlerts(alerts.filter(a => a.id !== alertId))
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
                    className={`${styles.tabBtn} ${activeTab === 'overview' ? styles.activeTab : ''}`}
                    onClick={() => setActiveTab('overview')}
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

                {activeTab === 'overview' && (
                    <div className={styles.settingsSection} style={{ maxWidth: '100%', padding: '2rem' }}>
                        <div style={{
                            padding: '2rem',
                            background: 'linear-gradient(to bottom right, rgba(15, 23, 42, 1), rgba(30, 41, 59, 1))',
                            border: '1px solid rgba(51, 65, 85, 0.5)',
                            borderRadius: '1rem',
                            boxShadow: '0 20px 25px -5px rgba(0, 0, 0, 0.1), 0 10px 10px -5px rgba(0, 0, 0, 0.04)'
                        }}>
                            <div style={{ display: 'flex', alignItems: 'center', gap: '1rem', marginBottom: '2rem' }}>
                                <div style={{
                                    width: '3rem', height: '3rem', borderRadius: '0.75rem',
                                    background: 'rgba(59, 130, 246, 0.2)',
                                    display: 'flex', alignItems: 'center', justifyContent: 'center',
                                    border: '1px solid rgba(59, 130, 246, 0.3)'
                                }}>
                                    <MdSecurity style={{ color: '#60a5fa', fontSize: '1.5rem' }} />
                                </div>
                                <div>
                                    <h3 style={{ fontSize: '1.25rem', fontWeight: 700, color: '#f1f5f9', margin: 0 }}>Proactive System Security</h3>
                                    <p style={{ color: '#94a3b8', fontSize: '0.875rem', marginTop: '0.25rem', marginBottom: 0 }}>
                                        FidduPay employs automated risk mitigation protocols to protect your institutional account.
                                    </p>
                                </div>
                            </div>
                            
                            <div style={{
                                display: 'grid',
                                gridTemplateColumns: 'repeat(auto-fit, minmax(300px, 1fr))',
                                gap: '1.5rem'
                            }}>
                                {/* Brute-Force Card */}
                                <div style={{
                                    position: 'relative', overflow: 'hidden', background: 'rgba(15, 23, 42, 0.5)',
                                    borderRadius: '0.75rem', border: '1px solid #334155', padding: '1.5rem',
                                    transition: 'all 0.3s'
                                }}>
                                    <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '1rem' }}>
                                        <div style={{
                                            width: '2.5rem', height: '2.5rem', borderRadius: '50%', background: '#1e293b',
                                            display: 'flex', alignItems: 'center', justifyContent: 'center', color: '#34d399',
                                            boxShadow: '0 0 15px rgba(52, 211, 153, 0.1)', border: '1px solid rgba(52, 211, 153, 0.2)'
                                        }}>
                                            <MdCheckCircle size={20} />
                                        </div>
                                        <span style={{
                                            fontSize: '0.625rem', textTransform: 'uppercase', letterSpacing: '0.05em', fontWeight: 700,
                                            color: '#34d399', background: 'rgba(52, 211, 153, 0.1)', padding: '0.25rem 0.5rem', borderRadius: '0.375rem'
                                        }}>Active</span>
                                    </div>
                                    <h4 style={{ fontSize: '1rem', fontWeight: 700, color: '#e2e8f0', margin: '0 0 0.5rem 0' }}>Brute-Force Shield</h4>
                                    <p style={{ fontSize: '0.875rem', color: '#94a3b8', margin: 0 }}>Automatic IP blacklisting after 5 failed login attempts within 10 minutes.</p>
                                </div>

                                {/* Session Integrity Card */}
                                <div style={{
                                    position: 'relative', overflow: 'hidden', background: 'rgba(15, 23, 42, 0.5)',
                                    borderRadius: '0.75rem', border: '1px solid #334155', padding: '1.5rem',
                                    transition: 'all 0.3s'
                                }}>
                                    <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '1rem' }}>
                                        <div style={{
                                            width: '2.5rem', height: '2.5rem', borderRadius: '50%', background: '#1e293b',
                                            display: 'flex', alignItems: 'center', justifyContent: 'center', color: '#34d399',
                                            boxShadow: '0 0 15px rgba(52, 211, 153, 0.1)', border: '1px solid rgba(52, 211, 153, 0.2)'
                                        }}>
                                            <MdCheckCircle size={20} />
                                        </div>
                                        <span style={{
                                            fontSize: '0.625rem', textTransform: 'uppercase', letterSpacing: '0.05em', fontWeight: 700,
                                            color: '#34d399', background: 'rgba(52, 211, 153, 0.1)', padding: '0.25rem 0.5rem', borderRadius: '0.375rem'
                                        }}>Active</span>
                                    </div>
                                    <h4 style={{ fontSize: '1rem', fontWeight: 700, color: '#e2e8f0', margin: '0 0 0.5rem 0' }}>Session Integrity</h4>
                                    <p style={{ fontSize: '0.875rem', color: '#94a3b8', margin: 0 }}>Immediate revocation of all active sessions when API keys or passwords are rotated.</p>
                                </div>

                                {/* Webhook Verification Card */}
                                <div style={{
                                    position: 'relative', overflow: 'hidden', background: 'rgba(15, 23, 42, 0.5)',
                                    borderRadius: '0.75rem', border: '1px solid #334155', padding: '1.5rem',
                                    transition: 'all 0.3s'
                                }}>
                                    <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '1rem' }}>
                                        <div style={{
                                            width: '2.5rem', height: '2.5rem', borderRadius: '50%', background: '#1e293b',
                                            display: 'flex', alignItems: 'center', justifyContent: 'center', color: '#34d399',
                                            boxShadow: '0 0 15px rgba(52, 211, 153, 0.1)', border: '1px solid rgba(52, 211, 153, 0.2)'
                                        }}>
                                            <MdCheckCircle size={20} />
                                        </div>
                                        <span style={{
                                            fontSize: '0.625rem', textTransform: 'uppercase', letterSpacing: '0.05em', fontWeight: 700,
                                            color: '#34d399', background: 'rgba(52, 211, 153, 0.1)', padding: '0.25rem 0.5rem', borderRadius: '0.375rem'
                                        }}>Active</span>
                                    </div>
                                    <h4 style={{ fontSize: '1rem', fontWeight: 700, color: '#e2e8f0', margin: '0 0 0.5rem 0' }}>Payload Verification</h4>
                                    <p style={{ fontSize: '0.875rem', color: '#94a3b8', margin: 0 }}>Strict HMAC-SHA256 signature validation required for all inbound webhook signals.</p>
                                </div>
                            </div>

                            <div style={{
                                marginTop: '2rem', display: 'flex', alignItems: 'center', justifyContent: 'space-between',
                                padding: '1rem', background: 'rgba(30, 41, 59, 0.5)', border: '1px solid #334155', borderRadius: '0.75rem', flexWrap: 'wrap', gap: '1rem'
                            }}>
                                <div style={{ display: 'flex', alignItems: 'center', gap: '0.75rem', fontSize: '0.875rem', color: '#cbd5e1' }}>
                                    <div style={{
                                        width: '2rem', height: '2rem', borderRadius: '50%', background: 'rgba(59, 130, 246, 0.1)',
                                        display: 'flex', alignItems: 'center', justifyContent: 'center', color: '#60a5fa'
                                    }}>
                                        <MdInfo size={16} />
                                    </div>
                                    <span>To configure threshold alerts or update security credentials, visit your Account Settings.</span>
                                </div>
                                <a href="/app/settings?tab=security" style={{
                                    padding: '0.5rem 1rem', background: '#2563eb', color: 'white', fontSize: '0.875rem', fontWeight: 700,
                                    borderRadius: '0.5rem', textDecoration: 'none', boxShadow: '0 0 15px rgba(37, 99, 235, 0.3)', transition: 'all 0.3s'
                                }}>
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
