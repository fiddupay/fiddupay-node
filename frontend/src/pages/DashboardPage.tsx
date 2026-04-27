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
import { useNavigate } from 'react-router-dom'
import { useAuthStore } from '@/stores/authStore'
import { useDataStore } from '@/stores/dataStore'
import { 
  MdWarning, 
  MdArrowForward, 
  MdBolt,
  MdShield,
  MdClose
} from 'react-icons/md'
import { UniversalPayForm } from '@/components/ui/UniversalPayForm'
import styles from '@/styles/pages/DashboardPage.module.css'
import { ActivityListSkeleton, DashboardSkeleton } from '@/components/layout/PageSkeletons'
import SEO from '@/components/ui/SEO'
import { TrustScoreWidget } from '@/components/ui/TrustScoreWidget'
import { SwarmIntelligenceWidget } from '@/components/ui/SwarmIntelligenceWidget'
import { Badge } from '@/components/ui/badge'

// Recent Activity Component
const RecentActivityList: React.FC = () => {
  const { user } = useAuthStore()
  const { recentActivity: activityCache, fetchRecentActivity } = useDataStore()
  const activities = activityCache.data || []
  const loading = activityCache.loading && activities.length === 0

  useEffect(() => {
    fetchRecentActivity()
  }, [user?.sandbox_mode])

  if (loading) {
    return <ActivityListSkeleton />
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
      case 'payment': return 'fa-arrow-down'
      case 'refund': return 'fa-undo'
      case 'withdrawal': return 'fa-arrow-up'
      default: return 'fa-exchange-alt'
    }
  }

  return (
    <div className={styles.activityList}>
      {activities.map((activity: any) => (
        <div key={`${activity.type}-${activity.id}`} className={styles.activityItem}>
          <div className={styles.activityInfo}>
            <div className={styles.activityLeft}>
              <i className={`fas ${getTypeIcon(activity.type)}`} style={{ width: '16px', textAlign: 'center', color: activity.type === 'payment' ? '#10b981' : (activity.type === 'withdrawal' ? 'var(--primary)' : 'var(--secondary)') }}></i>
              <span className={styles.activityId}>
                {activity.type === 'payment' ? 'Deposit' :
                  activity.type.charAt(0).toUpperCase() + activity.type.slice(1)} ({activity.id.substring(0, 8)}...)
              </span>
            </div>
            <div className={styles.activityRight}>
              {(() => {
                const sign = (activity.type === 'withdrawal' || activity.type === 'refund') ? '-' : ''
                const parts = activity.crypto_type?.split('_') || ['', '']
                const coin = parts[0]
                const cryptoAmt = (parseFloat(activity.crypto_amount || activity.usd_amount) || 0).toFixed(6)

                return (
                  <>
                    <span className={styles.activityAmount}>{sign}${(parseFloat(activity.usd_amount) || 0).toFixed(2)}</span>
                    <span className={styles.activityCrypto}>{sign}{cryptoAmt} {coin}</span>
                  </>
                )
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

import { useBalanceStore } from '@/stores/balanceStore'

const DashboardPage: React.FC = () => {
  const { user } = useAuthStore()
  const { balance, fetchBalance } = useBalanceStore()
  const { 
    analytics: analyticsCache,
    securityAlerts: alertsCache,
    fetchAnalytics,
    fetchSecurityAlerts 
  } = useDataStore()
  const navigate = useNavigate()
  const analytics = analyticsCache.data as AnalyticsData | null
  const alerts = alertsCache.data || []
  const loading = analyticsCache.loading && !analytics
  const [showQuickPayModal, setShowQuickPayModal] = useState(false)
  const [dailyVolumeUsed, setDailyVolumeUsed] = useState(0)
  const [dateRange, setDateRange] = useState(() => {
    const now = new Date();
    const firstDayOfMonth = new Date(now.getFullYear(), now.getMonth(), 1);

    return {
      from_date: firstDayOfMonth.toISOString().split('T')[0],
      to_date: new Date().toISOString().split('T')[0]
    };
  })

  useEffect(() => {
    loadDashboardData()
  }, [dateRange, user?.sandbox_mode])

  useEffect(() => {
    if (user?.daily_volume_remaining) {
      const remaining = parseFloat(user.daily_volume_remaining) || 0
      const limit = user.daily_limit_usd ? (parseFloat(user.daily_limit_usd) || 0) : 0
      const used = limit > 0 ? Math.max(0, limit - remaining) : 0
      setDailyVolumeUsed(used)
    }
  }, [user])

  const loadDashboardData = async () => {
    try {
      // Use the global data store — SWR pattern handles caching automatically
      await Promise.all([
        fetchAnalytics({
          from_date: new Date(dateRange.from_date).toISOString(),
          to_date: new Date(dateRange.to_date + 'T23:59:59Z').toISOString()
        }),
        fetchSecurityAlerts(),
        fetchBalance()
      ])
    } catch (error) {
      console.error('Failed to load dashboard data:', error)
    }
  }


  const totalPayments = (analytics?.successful_payments || 0) + (analytics?.failed_payments || 0) + (analytics?.pending_payments || 0)
  const successRate = totalPayments > 0
    ? ((analytics?.successful_payments || 0) / totalPayments * 100).toFixed(1)
    : '0'

  const chartData = analytics?.payment_trends?.map(point => ({
    ...point,
    volume: parseFloat(point.volume_usd) || 0,
    displayDate: new Date(point.date).toLocaleDateString(undefined, { month: 'short', day: 'numeric' })
  })) || []

  return (
    <div className={styles.page}>
      <SEO 
        title="Merchant Dashboard" 
        description="View your crypto payment analytics, recent activity, and wallet balances in real-time."
      />
      {/* Header */}
      <div className={styles.header}>
        <div>
          <h1 className={styles.title}>Dashboard</h1>
          <div className="flex items-center gap-3 mt-1">
            <p className={styles.subtitle}>Welcome back! Here's your business at a glance.</p>
            {user?.username && (
              <Badge className="bg-primary/20 text-primary border-primary/30 px-2 py-0.5 text-[10px] font-bold">
                @{user.username}
              </Badge>
            )}
            {user?.pay_id && (
              <Badge className="bg-secondary/20 text-secondary border-secondary/30 px-2 py-0.5 text-[10px] font-bold">
                {user.pay_id}
              </Badge>
            )}
          </div>
        </div>
        <div className={styles.filters}>
          <div className={styles.dateInput}>
            <label>From</label>
            <div className={styles.dateInputContainer}>
              <i className="fas fa-calendar-alt"></i>
              <input
                type="date"
                value={dateRange.from_date}
                onChange={(e) => setDateRange(prev => ({ ...prev, from_date: e.target.value }))}
              />
            </div>
          </div>
          <div className={styles.dateInput}>
            <label>To</label>
            <div className={styles.dateInputContainer}>
              <i className="fas fa-calendar-alt"></i>
              <input
                type="date"
                value={dateRange.to_date}
                onChange={(e) => setDateRange(prev => ({ ...prev, to_date: e.target.value }))}
              />
            </div>
          </div>
          <button className={styles.refreshBtn} onClick={loadDashboardData} disabled={loading} title="Refresh Dashboard">
            <i className={`fas fa-sync-alt ${loading ? 'fa-spin' : ''}`}></i>
          </button>
          <button className={styles.refreshBtn} onClick={() => setShowQuickPayModal(true)} title="Quick Interop Pay" style={{ color: 'var(--secondary)', background: 'rgba(245, 158, 11, 0.1)' }}>
            <MdBolt size={20} />
          </button>
          <button onClick={() => navigate('/app/settings?tab=verification')} title="Trust Intelligence Status" style={{ 
            color: (user?.kyc_tier || 0) >= 2 ? '#fbbf24' : ((user?.kyc_tier || 0) === 1 ? 'var(--primary)' : '#f59e0b'), 
            background: (user?.kyc_tier || 0) >= 2 ? 'rgba(251, 191, 36, 0.1)' : ((user?.kyc_tier || 0) === 1 ? 'rgba(99, 102, 241, 0.1)' : 'rgba(245, 158, 11, 0.1)'),
            padding: '8px 16px',
            borderRadius: '12px',
            border: '1px solid currentColor',
            display: 'flex',
            alignItems: 'center',
            gap: '8px',
            cursor: 'pointer',
            transition: 'all 0.2s',
            height: '42px',
            fontWeight: 'bold'
          }}>
            <MdShield size={18} />
            <span style={{ fontSize: '11px', fontWeight: '900', textTransform: 'uppercase', letterSpacing: '0.05em' }}>
              {(user?.kyc_tier || 0) >= 2 ? 'Gold Tier' : ((user?.kyc_tier || 0) === 1 ? 'Verified' : 'Sandbox')}
            </span>
          </button>
          <a href="/docs" className={styles.docsLink} target="_blank" rel="noopener noreferrer">
            <i className="fas fa-book"></i>
            API Docs
          </a>
        </div>
      </div>

      {loading && !analytics ? (
        <DashboardSkeleton />
      ) : (
        <>
          {!user?.has_transaction_pin && (
            <div className={`${styles.securityBanner} ${styles.pinBanner}`} style={{ borderLeft: '4px solid var(--secondary)', marginBottom: '16px' }}>
              <div className={styles.bannerIcon} style={{ background: 'rgba(245, 158, 11, 0.1)' }}>
                <MdWarning color="var(--secondary)" size={24} />
              </div>
              <div className={styles.bannerContent}>
                <h3 style={{ color: 'var(--text-main)' }}>Security setup required: Transaction PIN</h3>
                <p>A Transaction PIN is mandatory for all withdrawals and fund movements. Set your PIN now to enable financial actions.</p>
              </div>
              <button className={styles.bannerBtn} style={{ background: 'var(--secondary)', boxShadow: '0 0 15px var(--secondary-glow)' }} onClick={() => navigate('/app/settings?tab=security')}>
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
              <button className={styles.bannerBtn} onClick={() => navigate('/security')}>
                Go to Security Hub <MdArrowForward />
              </button>
            </div>
          )}

          {/* Trust Intelligence — Full Width */}
          <div className={styles.trustRow}>
            <TrustScoreWidget user={user} className="h-full" />
          </div>

          {/* Stats Grid — 4 cards */}
          <div className={styles.statsGrid}>
            <div className={styles.statCard}>
                <div className={styles.statHeader}>
                  <span className={styles.statLabel}>Total Payments</span>
                  <i className="fas fa-receipt" style={{ color: 'var(--primary)' }}></i>
                </div>
                <div className={styles.statValue}>{totalPayments.toLocaleString()}</div>
                <div className={styles.statFooter}>{analytics?.successful_payments || 0} successful / {analytics?.failed_payments || 0} failed</div>
            </div>

            <div className={styles.statCard}>
                <div className={styles.statHeader}>
                  <span className={styles.statLabel}>Signal Strength</span>
                  <i className="fas fa-satellite-dish" style={{ color: '#10b981' }}></i>
                </div>
                <div className={styles.statValue}>{user?.trust_score?.score || 0}%</div>
                <div className={styles.statFooter}>Intelligence Layer Pulse</div>
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
                  <span className={styles.statLabel}>Available Balance</span>
                  <i className="fas fa-wallet" style={{ color: 'var(--secondary)' }}></i>
                </div>
                <div className={`${styles.statValue} ${(parseFloat(balance?.total_usd || '0') < 0) ? styles.negativeValue : ''}`}>
                  ${(parseFloat(balance?.total_usd || '0') || 0).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 })}
                </div>
                <div className={styles.statFooter}>
                  Settled: <span className={(parseFloat(balance?.available_usd || '0') < 0) ? styles.negativeValue : ''}>
                    ${(parseFloat(balance?.available_usd || '0') || 0).toLocaleString()}
                  </span>
                </div>
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
                        <stop offset="5%" stopColor="var(--primary)" stopOpacity={0.3} />
                        <stop offset="95%" stopColor="var(--primary)" stopOpacity={0} />
                      </linearGradient>
                    </defs>
                    <CartesianGrid strokeDasharray="3 3" vertical={false} stroke="rgba(255, 255, 255, 0.05)" />
                    <XAxis
                      dataKey="displayDate"
                      axisLine={false}
                      tickLine={false}
                      tick={{ fill: 'var(--text-muted)', fontSize: 12 }}
                      dy={10}
                    />
                    <YAxis
                      axisLine={false}
                      tickLine={false}
                      tick={{ fill: 'var(--text-muted)', fontSize: 12 }}
                      tickFormatter={(value: number | string) => `$${value}`}
                    />
                    <Tooltip
                      contentStyle={{
                        background: '#1a1f2e',
                        borderRadius: '16px',
                        border: '1px solid var(--border)',
                        boxShadow: '0 20px 40px rgba(0,0,0,0.5)',
                        padding: '16px',
                        backdropFilter: 'blur(10px)'
                      }}
                      itemStyle={{ color: 'var(--text-main)', fontWeight: 700 }}
                      labelStyle={{ color: 'var(--text-muted)', marginBottom: '8px' }}
                      formatter={(value: any) => [`$${(parseFloat(String(value ?? 0)) || 0).toLocaleString()}`, 'Volume']}
                    />
                    <Area
                      type="monotone"
                      dataKey="volume"
                      stroke="var(--primary)"
                      strokeWidth={4}
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
                    .sort((a, b) => (parseFloat(b[1].volume_usd) || 0) - (parseFloat(a[1].volume_usd) || 0))
                    .map(([network, stats]) => (
                      <div key={network} className={styles.networkItem}>
                        <div className={styles.networkInfo}>
                          <span className={styles.networkName}>{network}</span>
                          <span className={styles.networkCount}>{stats.payment_count} payments</span>
                        </div>
                        <div className={styles.networkValue}>
                          <span className={styles.networkUsd}>
                            ${(parseFloat(stats.volume_usd) || 0).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 })}
                          </span>
                          <div className={styles.networkBar}>
                            <div
                              className={styles.networkBarFill}
                              style={{
                                width: `${(Math.min(100, (parseFloat(stats.volume_usd) || 0) / (parseFloat(analytics.total_volume_usd) || 1) * 100))}%`
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
                    <span className={`${styles.balanceAmount} ${(parseFloat(balance.available_usd) < 0) ? styles.negativeValue : ''}`}>
                      ${(parseFloat(balance.available_usd) || 0).toLocaleString(undefined, { minimumFractionDigits: 2 })}
                    </span>
                  </div>
                  <div className={styles.balanceRow}>
                    <span className={styles.balanceLabel}>Processing</span>
                    <span className={`${styles.balanceAmount} ${(parseFloat(balance.reserved_usd) < 0) ? styles.negativeValue : ''}`}>
                      ${(parseFloat(balance.reserved_usd) || 0).toLocaleString(undefined, { minimumFractionDigits: 2 })}
                    </span>
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

            <div className="xl:col-span-1">
              <SwarmIntelligenceWidget user={user} />
            </div>
          </div>

          {/* Performance Charts */}
          <div className={styles.chartContainer}>
            <div className={styles.sectionHeader}>
              <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                <div>
                  <h2>Volume Intelligence</h2>
                  <p>Transaction volume across selected period</p>
                </div>
                <div style={{ textAlign: 'right' }}>
                  <div style={{ fontSize: '10px', color: 'var(--text-muted)', fontWeight: 800, textTransform: 'uppercase', marginBottom: '4px' }}>Peak Volume</div>
                  <div style={{ color: 'var(--primary)', fontWeight: 900, fontSize: '1.2rem' }}>
                    ${Math.max(...(chartData.map(d => d.volume) || [0])).toLocaleString()}
                  </div>
                </div>
              </div>
            </div>
            
            <div className={styles.chartWrapper} style={{ height: '320px' }}>
              <ResponsiveContainer width="100%" height="100%">
                <AreaChart data={chartData}>
                  <defs>
                    <linearGradient id="colorVolume" x1="0" y1="0" x2="0" y2="1">
                      <stop offset="5%" stopColor="var(--primary)" stopOpacity={0.3}/>
                      <stop offset="95%" stopColor="var(--primary)" stopOpacity={0}/>
                    </linearGradient>
                  </defs>
                  <CartesianGrid strokeDasharray="3 3" vertical={false} stroke="rgba(255,255,255,0.05)" />
                  <XAxis 
                    dataKey="displayDate" 
                    axisLine={false}
                    tickLine={false}
                    tick={{ fill: 'var(--text-muted)', fontSize: 11, fontWeight: 600 }}
                    dy={10}
                  />
                  <YAxis 
                    axisLine={false}
                    tickLine={false}
                    tick={{ fill: 'var(--text-muted)', fontSize: 11, fontWeight: 600 }}
                    tickFormatter={(value) => `$${value}`}
                  />
                  <Tooltip 
                    contentStyle={{ 
                      backgroundColor: 'rgba(15, 23, 42, 0.9)', 
                      border: '1px solid rgba(255,255,255,0.1)',
                      borderRadius: '12px',
                      backdropFilter: 'blur(10px)',
                      color: 'white'
                    }}
                    itemStyle={{ color: 'var(--primary)', fontWeight: 800 }}
                  />
                  <Area 
                    type="monotone" 
                    dataKey="volume" 
                    stroke="var(--primary)" 
                    strokeWidth={4}
                    fillOpacity={1} 
                    fill="url(#colorVolume)" 
                    animationDuration={1500}
                  />
                </AreaChart>
              </ResponsiveContainer>
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
                        : 0}%`
                    }}
                  />
                </div>
                <div className={styles.volumeProgressLabels}>
                  <span>${dailyVolumeUsed.toFixed(2)} used</span>
                  <span>${user.daily_limit_usd ? (parseFloat(user.daily_limit_usd) || 0).toLocaleString() : 'N/A'} limit</span>
                </div>
              </div>
            </div>
          )}
        </>
      )}

      {showQuickPayModal && (
        <div className={styles.modalOverlay} onClick={() => setShowQuickPayModal(false)}>
          <div className={styles.modalContent} style={{ maxWidth: '480px', background: 'transparent', border: 'none', boxShadow: 'none' }} onClick={(e) => e.stopPropagation()}>
             <div style={{ display: 'flex', justifyContent: 'flex-end', marginBottom: '12px' }}>
                <button 
                  onClick={() => setShowQuickPayModal(false)}
                  className={styles.closeButton}
                  style={{ width: '44px', height: '44px', borderRadius: '50%', background: 'rgba(255,255,255,0.1)', color: '#fff' }}
                >
                  <MdClose size={24} />
                </button>
             </div>
             <UniversalPayForm />
          </div>
        </div>
      )}
    </div>
  )
}


export default DashboardPage
