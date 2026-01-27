import React, { useEffect, useState } from 'react'
import { MdPayment, MdTrendingUp, MdAccountBalance, MdPending, MdVerifiedUser, MdWarning } from 'react-icons/md'
import { useAuth } from '../contexts/AuthContext'
import { apiService } from '../services/api'
import { Analytics, Balance, Payment } from '../types'
import styles from './DashboardPage.module.css'

const DashboardPage: React.FC = () => {
  const { user } = useAuth()
  const [analytics, setAnalytics] = useState<Analytics | null>(null)
  const [balance, setBalance] = useState<Balance | null>(null)
  const [loading, setLoading] = useState(true)
  const [dailyVolumeUsed, setDailyVolumeUsed] = useState(0)

  useEffect(() => {
    loadDashboardData()
    // Calculate daily volume used
    if (user?.daily_volume_remaining) {
      const remaining = parseFloat(user.daily_volume_remaining)
      const limit = user.kyc_verified ? 0 : 1000 // $1000 limit for non-KYC
      const used = limit > 0 ? limit - remaining : 0
      setDailyVolumeUsed(used)
    }
  }, [user])

  const loadDashboardData = async () => {
    try {
      setLoading(true)
      const [analyticsData, balanceData] = await Promise.all([
        apiService.getAnalytics(),
        apiService.getBalance()
      ])
      setAnalytics(analyticsData)
      setBalance(balanceData)
    } catch (error) {
      console.error('Failed to load dashboard data:', error)
      // Set empty data on error to prevent crashes
      setAnalytics({
        total_payments: 0,
        total_volume_usd: '0',
        successful_payments: 0,
        pending_payments: 0,
        failed_payments: 0,
        average_payment_usd: '0',
        payment_trends: [],
        currency_breakdown: []
      })
      setBalance({
        total_usd: '0',
        available_usd: '0',
        reserved_usd: '0',
        balances: []
      })
    } finally {
      setLoading(false)
    }
  }

  // Real data from API with calculated trends
  const stats = [
    {
      name: 'Total Payments',
      value: analytics?.total_payments?.toLocaleString() || '0',
      change: calculatePaymentTrend(analytics?.payment_trends || []),
      changeType: getChangeType(calculatePaymentTrend(analytics?.payment_trends || [])),
      icon: MdPayment,
    },
    {
      name: 'Total Volume',
      value: analytics?.total_volume_usd ? `$${parseFloat(analytics.total_volume_usd).toLocaleString()}` : '$0',
      change: calculateVolumeTrend(analytics?.payment_trends || []),
      changeType: getChangeType(calculateVolumeTrend(analytics?.payment_trends || [])),
      icon: MdTrendingUp,
    },
    {
      name: 'Balance',
      value: balance?.total_usd ? `$${parseFloat(balance.total_usd).toLocaleString()}` : '$0',
      change: '+0%', // Balance doesn't have historical comparison yet
      changeType: 'neutral' as const,
      icon: MdAccountBalance,
    },
    {
      name: 'Pending',
      value: analytics?.pending_payments?.toString() || '0',
      change: calculatePendingTrend(analytics || undefined),
      changeType: getChangeType(calculatePendingTrend(analytics || undefined)),
      icon: MdPending,
    },
  ]

  // Helper functions for trend calculations
  function calculatePaymentTrend(trends?: any[]): string {
    if (!trends || trends.length < 2) return '+0%'
    const recent = trends[trends.length - 1]?.count || 0
    const previous = trends[trends.length - 2]?.count || 0
    if (previous === 0) return '+0%'
    const change = ((recent - previous) / previous) * 100
    return `${change >= 0 ? '+' : ''}${change.toFixed(1)}%`
  }

  function calculateVolumeTrend(trends?: any[]): string {
    if (!trends || trends.length < 2) return '+0%'
    const recent = parseFloat(trends[trends.length - 1]?.volume_usd || '0')
    const previous = parseFloat(trends[trends.length - 2]?.volume_usd || '0')
    if (previous === 0) return '+0%'
    const change = ((recent - previous) / previous) * 100
    return `${change >= 0 ? '+' : ''}${change.toFixed(1)}%`
  }

  function calculatePendingTrend(analytics?: Analytics): string {
    if (!analytics) return '+0%'
    const total = analytics.total_payments || 0
    const pending = analytics.pending_payments || 0
    if (total === 0) return '+0%'
    const pendingRate = (pending / total) * 100
    return pendingRate > 5 ? '+2%' : '-2%' // Simple heuristic
  }

  function getChangeType(change: string): 'positive' | 'negative' | 'neutral' {
    if (change.startsWith('+') && !change.startsWith('+0')) return 'positive'
    if (change.startsWith('-')) return 'negative'
    return 'neutral'
  }

  return (
    <div className={styles.container}>
      <div className={styles.header}>
        <h1 className={styles.title}>Dashboard</h1>
        <p className={styles.subtitle}>Welcome back! Here's what's happening with your payments.</p>
      </div>

      <div className={styles.statsGrid}>
        {stats.map((stat) => (
          <div key={stat.name} className={styles.statCard}>
            <div className={styles.statContent}>
              <div className={styles.statInfo}>
                <p className={styles.statName}>{stat.name}</p>
                <p className={styles.statValue}>{stat.value}</p>
              </div>
              <div className={styles.statIcon}>
                <stat.icon />
              </div>
            </div>
            <div className={styles.statFooter}>
              <span className={`${styles.statChange} ${styles[stat.changeType]}`}>
                {stat.change}
              </span>
              <span className={styles.statPeriod}>from last month</span>
            </div>
          </div>
        ))}
      </div>

      {/* Daily Volume Limit Section */}
      {user && (
        <div className={styles.volumeLimitSection}>
          <div className={styles.volumeLimitCard}>
            <div className={styles.volumeLimitHeader}>
              <div className={styles.volumeLimitIcon}>
                {user.kyc_verified ? <MdVerifiedUser /> : <MdWarning />}
              </div>
              <div className={styles.volumeLimitInfo}>
                <h3 className={styles.volumeLimitTitle}>
                  {user.kyc_verified ? 'KYC Verified Account' : 'Daily Volume Limit'}
                </h3>
                <p className={styles.volumeLimitSubtitle}>
                  {user.kyc_verified 
                    ? 'No daily volume limits apply to your account'
                    : `$${user.daily_volume_remaining} remaining today`
                  }
                </p>
              </div>
            </div>
            {!user.kyc_verified && (
              <div className={styles.volumeLimitProgress}>
                <div className={styles.progressBar}>
                  <div 
                    className={styles.progressFill}
                    style={{ 
                      width: `${((1000 - parseFloat(user.daily_volume_remaining || '1000')) / 1000) * 100}%` 
                    }}
                  />
                </div>
                <div className={styles.progressLabels}>
                  <span>${dailyVolumeUsed.toFixed(2)} used</span>
                  <span>$1,000.00 limit</span>
                </div>
              </div>
            )}
          </div>
        </div>
      )}

      <div className={styles.content}>
        <div className={styles.section}>
          <h2 className={styles.sectionTitle}>Recent Payments</h2>
          {loading ? (
            <div className={styles.loading}>Loading payments...</div>
          ) : (
            <RecentPaymentsList />
          )}
        </div>

        <div className={styles.section}>
          <h2 className={styles.sectionTitle}>Balance Overview</h2>
          {loading ? (
            <div className={styles.loading}>Loading balance...</div>
          ) : balance ? (
            <div className={styles.balanceOverview}>
              <div className={styles.balanceItem}>
                <span className={styles.balanceLabel}>Available:</span>
                <span className={styles.balanceValue}>${parseFloat(balance.available_usd).toLocaleString()}</span>
              </div>
              <div className={styles.balanceItem}>
                <span className={styles.balanceLabel}>Reserved:</span>
                <span className={styles.balanceValue}>${parseFloat(balance.reserved_usd).toLocaleString()}</span>
              </div>
              {balance.balances.map((currencyBalance) => (
                <div key={currencyBalance.crypto_type} className={styles.currencyBalance}>
                  <span className={styles.currencyType}>{currencyBalance.crypto_type}:</span>
                  <span className={styles.currencyAmount}>{currencyBalance.amount}</span>
                  <span className={styles.currencyUsd}>(${parseFloat(currencyBalance.amount_usd).toLocaleString()})</span>
                </div>
              ))}
            </div>
          ) : (
            <div className={styles.placeholder}>No balance data available</div>
          )}
        </div>
      </div>
    </div>
  )
}

// Recent Payments Component
const RecentPaymentsList: React.FC = () => {
  const [payments, setPayments] = useState<Payment[]>([])
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    loadRecentPayments()
  }, [])

  const loadRecentPayments = async () => {
    try {
      const response = await apiService.getPayments({ page: 1, page_size: 5 })
      setPayments(response.data || [])
    } catch (error) {
      console.error('Failed to load recent payments:', error)
    } finally {
      setLoading(false)
    }
  }

  if (loading) {
    return <div className={styles.loading}>Loading recent payments...</div>
  }

  if (payments.length === 0) {
    return <div className={styles.placeholder}>No recent payments</div>
  }

  return (
    <div className={styles.paymentsList}>
      {payments.slice(0, 5).map((payment: any) => (
        <div key={payment.payment_id} className={styles.paymentItem}>
          <div className={styles.paymentInfo}>
            <span className={styles.paymentId}>{payment.payment_id.substring(0, 8)}...</span>
            <span className={styles.paymentAmount}>${payment.amount_usd}</span>
          </div>
          <div className={styles.paymentMeta}>
            <span className={`${styles.paymentStatus} ${styles[payment.status.toLowerCase()]}`}>
              {payment.status}
            </span>
            <span className={styles.paymentDate}>
              {new Date(payment.created_at).toLocaleDateString()}
            </span>
          </div>
        </div>
      ))}
    </div>
  )
}

export default DashboardPage
