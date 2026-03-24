import React, { useEffect, useState } from 'react'
import {
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
  AreaChart,
  Area
} from 'recharts'
import { useAuthStore } from '@/stores/authStore'
import { merchantAPI, paymentAPI, securityAPI } from '@/services/apiService'
import { Balance, SecurityAlert } from '../types'
import { MdWarning, MdArrowForward } from 'react-icons/md'
import styles from '@/styles/pages/DashboardPage.module.css'

interface AnalyticsData {
  total_volume_usd: string
  successful_payments: number
  failed_payments: number
  pending_payments?: number
  total_fees_paid: string
  average_transaction_value: string
  average_payment_usd?: string
  by_blockchain: Record<string, {
    volume_usd: string
    payment_count: number
    average_value: string
  }>
  payment_trends: { date: string; volume_usd: string; count: number }[]
}

const DashboardPage: React.FC = () => {
  const { user } = useAuthStore()
  const [analytics, setAnalytics] = useState<AnalyticsData | null>(null)
  const [balance, setBalance] = useState<Balance | null>(null)
  const [alerts, setAlerts] = useState<SecurityAlert[]>([])
  const [loading, setLoading] = useState(true)
  const [dailyVolumeUsed, setDailyVolumeUsed] = useState(0)
  const [dateRange, setDateRange] = useState(() => {
    const now = new Date();
    const dayOfWeek = now.getDay(); // 0 (Sun) to 6 (Sat)
    const diff = now.getDate() - dayOfWeek + (dayOfWeek === 0 ? -6 : 1); // Adjust to Monday
    const monday = new Date(now.setDate(diff));

    return {
      from_date: monday.toISOString().split('T')[0],
      to_date: new Date().toISOString().split('T')[0]
    };
  })

  useEffect(() => {
    loadDashboardData()
  }, [dateRange])

  useEffect(() => {
    if (user?.daily_volume_remaining) {
      const remaining = parseFloat(user.daily_volume_remaining)
      const systemDefaultLimit = user.kyc_verified ? 0 : 1000
      const limit = user.daily_limit_usd ? parseFloat(user.daily_limit_usd) : systemDefaultLimit
      const used = limit > 0 ? limit - remaining : 0
      setDailyVolumeUsed(used)
    }
  }, [user])

  const loadDashboardData = async () => {
    try {
      setLoading(true)
      const [analyticsData, balanceData, alertsData] = await Promise.all([
        merchantAPI.getAnalytics({
          from_date: new Date(dateRange.from_date).toISOString(),
          to_date: new Date(dateRange.to_date + 'T23:59:59Z').toISOString()
        }),
        merchantAPI.getBalance(),
        securityAPI.getAlerts()
      ])
      setAnalytics(analyticsData.data)
      setBalance(balanceData.data)
      setAlerts(alertsData.data || [])
    } catch (error) {
      console.error('Failed to load dashboard data:', error)
    } finally {
      setLoading(false)
    }
  }

  const totalPayments = (analytics?.successful_payments || 0) + (analytics?.failed_payments || 0) + (analytics?.pending_payments || 0)
  const successRate = totalPayments > 0
    ? ((analytics?.successful_payments || 0) / totalPayments * 100).toFixed(1)
    : '0'

  const chartData = analytics?.payment_trends?.map(point => ({
    ...point,
    volume: parseFloat(point.volume_usd),
    displayDate: new Date(point.date).toLocaleDateString(undefined, { month: 'short', day: 'numeric' })
  })) || []

  return (
    <div className={styles.page}>
      {/* Header */}
      <div className={styles.header}>
        <div>
          <h1 className={styles.title}>Dashboard</h1>
          <p className={styles.subtitle}>Welcome back! Here's your business at a glance.</p>
        </div>
        <div className={styles.filters}>
          <div className={styles.dateInput}>
            <label>From</label>
            <input
              type="date"
              value={dateRange.from_date}
              onChange={(e) => setDateRange(prev => ({ ...prev, from_date: e.target.value }))}
            />
          </div>
          <div className={styles.dateInput}>
            <label>To</label>
            <input
              type="date"
              value={dateRange.to_date}
              onChange={(e) => setDateRange(prev => ({ ...prev, to_date: e.target.value }))}
            />
          </div>
          <button className={styles.refreshBtn} onClick={loadDashboardData} disabled={loading}>
            <i className={`fas fa-sync-alt ${loading ? 'fa-spin' : ''}`}></i>
          </button>
          <a href="/docs" className={styles.docsLink} target="_blank" rel="noopener noreferrer">
            <i className="fas fa-book"></i>
            API Docs
          </a>
        </div>
      </div>

      {loading && !analytics ? (
        <div className={styles.loadingState}>
          <i className="fas fa-spinner fa-spin"></i>
          <p>Loading your dashboard...</p>
        </div>
      ) : (
        <>
          {!user?.has_transaction_pin && (
            <div className={`${styles.securityBanner} ${styles.pinBanner}`} style={{ borderLeftColor: '#f59e0b', marginBottom: '16px' }}>
              <div className={styles.bannerIcon} style={{ background: '#fef3c7' }}>
                <MdWarning color="#f59e0b" size={24} />
              </div>
              <div className={styles.bannerContent}>
                <h3 style={{ color: '#92400e' }}>Security setup required: 4-Digit PIN</h3>
                <p>A Transaction PIN is mandatory for all withdrawals and fund movements. Set your PIN now to enable financial actions.</p>
              </div>
              <button className={styles.bannerBtn} style={{ background: '#f59e0b' }} onClick={() => window.location.href='/settings?tab=security'}>
                Setup PIN <MdArrowForward />
              </button>
            </div>
          )}

          {alerts.length > 0 && (
            <div className={styles.securityBanner}>
              <div className={styles.bannerIcon}>
                <MdWarning color="#ef4444" size={24} />
              </div>
              <div className={styles.bannerContent}>
                <h3>Action Required: {alerts.length} Security Alerts</h3>
                <p>Potential unauthorized access or system warnings detected. Please review your security logs immediately.</p>
              </div>
              <button className={styles.bannerBtn} onClick={() => window.location.href='/security'}>
                Go to Security Hub <MdArrowForward />
              </button>
            </div>
          )}

          {/* Stats Row */}
          <div className={styles.statsGrid}>
            <div className={styles.statCard}>
              <div className={styles.statHeader}>
                <span className={styles.statLabel}>Total Payments</span>
                <i className="fas fa-receipt" style={{ color: '#3b82f6' }}></i>
              </div>
              <div className={styles.statValue}>{totalPayments.toLocaleString()}</div>
              <div className={styles.statFooter}>{analytics?.successful_payments || 0} successful / {analytics?.failed_payments || 0} failed</div>
            </div>

            <div className={styles.statCard}>
              <div className={styles.statHeader}>
                <span className={styles.statLabel}>Total Volume</span>
                <i className="fas fa-chart-line" style={{ color: '#10b981' }}></i>
              </div>
              <div className={styles.statValue}>${parseFloat(analytics?.total_volume_usd || '0').toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 })}</div>
              <div className={styles.statFooter}>Total revenue processed</div>
            </div>

            <div className={styles.statCard}>
              <div className={styles.statHeader}>
                <span className={styles.statLabel}>Success Rate</span>
                <i className="fas fa-check-circle" style={{ color: '#22c55e' }}></i>
              </div>
              <div className={styles.statValue}>{successRate}%</div>
              <div className={styles.statFooter}>Payment completion rate</div>
            </div>

            <div className={styles.statCard}>
              <div className={styles.statHeader}>
                <span className={styles.statLabel}>Balance</span>
                <i className="fas fa-wallet" style={{ color: '#8b5cf6' }}></i>
              </div>
              <div className={styles.statValue}>${parseFloat(balance?.total_usd || '0').toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 })}</div>
              <div className={styles.statFooter}>Available: ${parseFloat(balance?.available_usd || '0').toLocaleString()}</div>
            </div>
          </div>

          {/* Revenue Trend Chart */}
          <div className={styles.chartContainer}>
            <div className={styles.sectionHeader}>
              <h2>Revenue Trend</h2>
              <p>Daily processing volume in USD</p>
            </div>
            <div className={styles.chartWrapper}>
              {chartData.length > 0 ? (
                <ResponsiveContainer width="100%" height={300}>
                  <AreaChart data={chartData}>
                    <defs>
                      <linearGradient id="colorVolume" x1="0" y1="0" x2="0" y2="1">
                        <stop offset="5%" stopColor="#3b82f6" stopOpacity={0.3} />
                        <stop offset="95%" stopColor="#3b82f6" stopOpacity={0} />
                      </linearGradient>
                    </defs>
                    <CartesianGrid strokeDasharray="3 3" vertical={false} stroke="#f0f0f0" />
                    <XAxis
                      dataKey="displayDate"
                      axisLine={false}
                      tickLine={false}
                      tick={{ fill: '#94a3b8', fontSize: 12 }}
                      dy={10}
                    />
                    <YAxis
                      axisLine={false}
                      tickLine={false}
                      tick={{ fill: '#94a3b8', fontSize: 12 }}
                      tickFormatter={(value: number | string) => `$${value}`}
                    />
                    <Tooltip
                      contentStyle={{
                        borderRadius: '12px',
                        border: 'none',
                        boxShadow: '0 10px 15px -3px rgba(0,0,0,0.1)',
                        padding: '12px'
                      }}
                      formatter={(value: any) => [`$${parseFloat(String(value ?? 0)).toLocaleString()}`, 'Volume']}
                    />
                    <Area
                      type="monotone"
                      dataKey="volume"
                      stroke="#3b82f6"
                      strokeWidth={3}
                      fillOpacity={1}
                      fill="url(#colorVolume)"
                      animationDuration={1500}
                    />
                  </AreaChart>
                </ResponsiveContainer>
              ) : (
                <div className={styles.emptyState}>
                  <i className="fas fa-chart-area"></i>
                  <p>No trend data available for this period.</p>
                </div>
              )}
            </div>
          </div>

          {/* Middle Grid: Network Breakdown + Balance Overview */}
          <div className={styles.mainGrid}>
            <div className={styles.sectionCard}>
              <div className={styles.sectionHeader}>
                <h2>Network Breakdown</h2>
                <p>Volume distribution across blockchains</p>
              </div>
              <div className={styles.networkList}>
                {analytics?.by_blockchain && Object.keys(analytics.by_blockchain).length > 0 ? (
                  Object.entries(analytics.by_blockchain)
                    .sort((a, b) => parseFloat(b[1].volume_usd) - parseFloat(a[1].volume_usd))
                    .map(([network, stats]) => (
                      <div key={network} className={styles.networkItem}>
                        <div className={styles.networkInfo}>
                          <span className={styles.networkName}>{network}</span>
                          <span className={styles.networkCount}>{stats.payment_count} payments</span>
                        </div>
                        <div className={styles.networkValue}>
                          <span className={styles.networkUsd}>
                            ${parseFloat(stats.volume_usd).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 })}
                          </span>
                          <div className={styles.networkBar}>
                            <div
                              className={styles.networkBarFill}
                              style={{
                                width: `${(parseFloat(stats.volume_usd) / (parseFloat(analytics.total_volume_usd) || 1) * 100)}%`
                              }}
                            ></div>
                          </div>
                        </div>
                      </div>
                    ))
                ) : (
                  <div className={styles.emptyState}>
                    <i className="fas fa-globe"></i>
                    <p>No network data available.</p>
                  </div>
                )}
              </div>
            </div>

            <div className={styles.sectionCard}>
              <div className={styles.sectionHeader}>
                <h2>Balance Overview</h2>
                <p>Current wallet balances</p>
              </div>
              {balance ? (
                <div className={styles.balanceList}>
                  <div className={styles.balanceRow}>
                    <span className={styles.balanceLabel}>Available</span>
                    <span className={styles.balanceAmount}>${parseFloat(balance.available_usd).toLocaleString(undefined, { minimumFractionDigits: 2 })}</span>
                  </div>
                  <div className={styles.balanceRow}>
                    <span className={styles.balanceLabel}>Processing</span>
                    <span className={styles.balanceAmount}>${parseFloat(balance.reserved_usd).toLocaleString(undefined, { minimumFractionDigits: 2 })}</span>
                  </div>
                </div>
              ) : (
                <div className={styles.emptyState}>
                  <i className="fas fa-wallet"></i>
                  <p>No balance data available.</p>
                </div>
              )}
            </div>
          </div>

          {/* Bottom Grid: Recent Activity + Performance */}
          <div className={styles.mainGrid}>
            <div className={styles.sectionCard}>
              <div className={styles.sectionHeader}>
                <h2>Recent Activity</h2>
                <p>Latest transactions</p>
              </div>
              <RecentActivityList />
            </div>

            <div className={styles.sectionCard}>
              <div className={styles.sectionHeader}>
                <h2>Performance</h2>
                <p>Key metrics summary</p>
              </div>
              <div className={styles.performanceList}>
                <div className={styles.perfItem}>
                  <span className={styles.perfLabel}>Successful</span>
                  <span className={`${styles.perfValue} ${styles.positive}`}>{analytics?.successful_payments || 0}</span>
                </div>
                <div className={styles.perfItem}>
                  <span className={styles.perfLabel}>Failed / Expired</span>
                  <span className={`${styles.perfValue} ${styles.negative}`}>{analytics?.failed_payments || 0}</span>
                </div>
                <div className={styles.perfItem}>
                  <span className={styles.perfLabel}>Avg. Transaction</span>
                  <span className={styles.perfValue}>${parseFloat(analytics?.average_transaction_value || analytics?.average_payment_usd || '0').toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 })}</span>
                </div>
                <div className={styles.perfItem}>
                  <span className={styles.perfLabel}>Total Fees</span>
                  <span className={styles.perfValue}>${parseFloat(analytics?.total_fees_paid || '0').toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 })}</span>
                </div>
              </div>
            </div>
          </div>

          {/* Daily Volume Limit */}
          {user && !user.kyc_verified && (
            <div className={styles.volumeLimitCard}>
              <div className={styles.volumeLimitHeader}>
                <div className={styles.volumeLimitIcon}>
                  <i className="fas fa-exclamation-triangle"></i>
                </div>
                <div>
                  <h3 className={styles.volumeLimitTitle}>Daily Volume Limit</h3>
                  <p className={styles.volumeLimitSubtitle}>${user.daily_volume_remaining} remaining today</p>
                </div>
              </div>
              <div className={styles.volumeProgressWrap}>
                <div className={styles.volumeProgressBar}>
                  <div
                    className={styles.volumeProgressFill}
                    style={{
                      width: `${user.daily_limit_usd && parseFloat(user.daily_limit_usd) > 0
                        ? ((parseFloat(user.daily_limit_usd) - parseFloat(user.daily_volume_remaining)) / parseFloat(user.daily_limit_usd)) * 100
                        : ((1000 - parseFloat(user.daily_volume_remaining)) / 1000) * 100}%`
                    }}
                  />
                </div>
                <div className={styles.volumeProgressLabels}>
                  <span>${dailyVolumeUsed.toFixed(2)} used</span>
                  <span>${user.daily_limit_usd ? parseFloat(user.daily_limit_usd).toLocaleString() : '1,000.00'} limit</span>
                </div>
              </div>
            </div>
          )}
        </>
      )}
    </div>
  )
}

// Recent Activity Component
const RecentActivityList: React.FC = () => {
  const { user } = useAuthStore()
  const [activities, setActivities] = useState<any[]>([])
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    loadRecentActivity()
  }, [user?.sandbox_mode])

  const loadRecentActivity = async () => {
    try {
      const response = await paymentAPI.getUnifiedTransactions({ limit: 5 })
      if (response.data && Array.isArray(response.data.transactions)) {
        setActivities(response.data.transactions)
      } else {
        setActivities([])
      }
    } catch (error) {
      console.error('Failed to load activity:', error)
      setActivities([])
    } finally {
      setLoading(false)
    }
  }

  if (loading) {
    return (
      <div className={styles.loadingState} style={{ padding: '2rem' }}>
        <i className="fas fa-spinner fa-spin"></i>
      </div>
    )
  }

  if (activities.length === 0) {
    return (
      <div className={styles.emptyState}>
        <i className="fas fa-inbox"></i>
        <p>No recent activity</p>
      </div>
    )
  }

  const getTypeIcon = (type: string) => {
    switch (type) {
      case 'payment': return 'fa-arrow-down text-green-500'
      case 'refund': return 'fa-undo text-orange-500'
      case 'withdrawal': return 'fa-arrow-up text-blue-500'
      default: return 'fa-exchange-alt'
    }
  }

  return (
    <div className={styles.activityList}>
      {activities.map((activity: any) => (
        <div key={`${activity.type}-${activity.id}`} className={styles.activityItem}>
          <div className={styles.activityInfo}>
            <div className={styles.activityLeft}>
              <i className={`fas ${getTypeIcon(activity.type)}`} style={{ width: '16px', textAlign: 'center' }}></i>
              <span className={styles.activityId}>
                {activity.type === 'payment' ? 'Deposit' :
                  activity.type.charAt(0).toUpperCase() + activity.type.slice(1)} ({activity.id.substring(0, 8)}...)
              </span>
            </div>
            <div className={styles.activityRight}>
              {(() => {
                const isStablecoin = activity.crypto_type?.toUpperCase().includes('USDT')
                const sign = (activity.type === 'withdrawal' || activity.type === 'refund') ? '-' : ''
                const parts = activity.crypto_type?.split('_') || ['', '']
                const coin = parts[0]
                const cryptoAmt = parseFloat(activity.crypto_amount || activity.usd_amount).toFixed(6)

                if (isStablecoin) {
                  return (
                    <>
                      <span className={styles.activityAmount}>{sign}{cryptoAmt} {coin}</span>
                    </>
                  )
                } else {
                  return (
                    <>
                      <span className={styles.activityAmount}>{sign}${parseFloat(activity.usd_amount).toFixed(2)}</span>
                      <span className={styles.activityCrypto}>{sign}{cryptoAmt} {coin}</span>
                    </>
                  )
                }
              })()}
            </div>
          </div>
          <div className={styles.activityMeta}>
            <span className={`${styles.activityStatus} ${styles[activity.status.toLowerCase()]}`}>
              {activity.status}
            </span>
            <span className={styles.activityDate}>
              {new Date(activity.created_at).toLocaleDateString()}
            </span>
          </div>
        </div>
      ))}
    </div>
  )
}

export default DashboardPage
