import React, { useState, useEffect } from 'react'
import styles from './StatusPage.module.css'

interface ServiceStatus {
  name: string
  description: string
  status: string
  response_time?: number
  last_check: string
}

interface UptimeStats {
  thirty_days: number
  ninety_days: number
  one_year: number
}

interface SystemStatus {
  overall_status: string
  services: ServiceStatus[]
  uptime_stats: UptimeStats
  last_updated: string
}

const StatusPage: React.FC = () => {
  const [status, setStatus] = useState<SystemStatus | null>(null)
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    const fetchStatus = async () => {
      try {
        const response = await fetch('/api/v1/status')
        const data = await response.json()
        setStatus(data)
      } catch (error) {
        console.error('Failed to fetch status:', error)
      } finally {
        setLoading(false)
      }
    }

    fetchStatus()
    // Refresh every 30 seconds
    const interval = setInterval(fetchStatus, 30000)
    return () => clearInterval(interval)
  }, [])

  if (loading) {
    return (
      <div className={styles.statusPage}>
        <div className={styles.container}>
          <div className={styles.loading}>Loading system status...</div>
        </div>
      </div>
    )
  }

  if (!status) {
    return (
      <div className={styles.statusPage}>
        <div className={styles.container}>
          <div className={styles.error}>Failed to load system status</div>
        </div>
      </div>
    )
  }
  return (
    <div className={styles.statusPage}>
      <div className={styles.container}>
        <div className={styles.header}>
          <h1 className={styles.title}>System Status</h1>
          <p className={styles.subtitle}>Real-time status of FidduPay services</p>
        </div>

        <div className={styles.overallStatus}>
          <div className={`${styles.statusIndicator} ${status.overall_status !== 'operational' ? styles.degraded : ''}`}>
            <i className={status.overall_status === 'operational' ? 'fas fa-check-circle' : 'fas fa-exclamation-triangle'}></i>
            <span>{status.overall_status === 'operational' ? 'All Systems Operational' : 'System Issues Detected'}</span>
          </div>
        </div>

        <div className={styles.services}>
          <div className={styles.servicesHeader}>
            <h2 className={styles.servicesTitle}>Service Status</h2>
          </div>
          {status.services.map((service) => (
            <div key={service.name} className={styles.service}>
              <div className={styles.serviceInfo}>
                <h3 className={styles.serviceName}>{service.name}</h3>
                <p className={styles.serviceDescription}>{service.description}</p>
              </div>
              <div className={styles.serviceMetrics}>
                {service.response_time && (
                  <span className={styles.responseTime}>{service.response_time}ms</span>
                )}
                <span className={`${styles.serviceStatus} ${service.status === 'operational' ? styles.operational : styles.degraded}`}>
                  {service.status}
                </span>
              </div>
            </div>
          ))}
        </div>

        <div className={styles.uptime}>
          <h2 className={styles.uptimeTitle}>Uptime Statistics</h2>
          <div className={styles.uptimeStats}>
            <div className={styles.stat}>
              <span className={styles.statValue}>{status.uptime_stats.thirty_days.toFixed(2)}%</span>
              <span className={styles.statLabel}>30 Days</span>
            </div>
            <div className={styles.stat}>
              <span className={styles.statValue}>{status.uptime_stats.ninety_days.toFixed(2)}%</span>
              <span className={styles.statLabel}>90 Days</span>
            </div>
            <div className={styles.stat}>
              <span className={styles.statValue}>{status.uptime_stats.one_year.toFixed(2)}%</span>
              <span className={styles.statLabel}>1 Year</span>
            </div>
          </div>
          <p className={styles.lastUpdated}>
            Last updated: {new Date(status.last_updated).toLocaleString()}
          </p>
        </div>
      </div>
    </div>
  )
}

export default StatusPage
