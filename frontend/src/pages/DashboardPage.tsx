import React, { useEffect, useState } from 'react'
import { MdPayment, MdTrendingUp, MdAccountBalance, MdPending, MdVerifiedUser, MdWarning } from 'react-icons/md'
import { useAuth } from '../contexts/AuthContext'
import styles from './DashboardPage.module.css'

const DashboardPage: React.FC = () => {
  const { user } = useAuth()
  const [dailyVolumeUsed, setDailyVolumeUsed] = useState(0)

  useEffect(() => {
    // Calculate daily volume used
    if (user?.daily_volume_remaining) {
      const remaining = parseFloat(user.daily_volume_remaining)
      const limit = user.kyc_verified ? 0 : 1000 // $1000 limit for non-KYC
      const used = limit > 0 ? limit - remaining : 0
      setDailyVolumeUsed(used)
    }
  }, [user])

  // Mock data - replace with real data from API
  const stats = [
    {
      name: 'Total Payments',
      value: '1,234',
      change: '+12%',
      changeType: 'positive' as const,
      icon: MdPayment,
    },
    {
      name: 'Total Volume',
      value: '$45,678',
      change: '+8%',
      changeType: 'positive' as const,
      icon: MdTrendingUp,
    },
    {
      name: 'Balance',
      value: '$12,345',
      change: '+5%',
      changeType: 'positive' as const,
      icon: MdAccountBalance,
    },
    {
      name: 'Pending',
      value: '23',
      change: '-2%',
      changeType: 'negative' as const,
      icon: MdPending,
    },
  ]

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
          <div className={styles.placeholder}>
            <p>Payment list will be implemented here</p>
          </div>
        </div>

        <div className={styles.section}>
          <h2 className={styles.sectionTitle}>Analytics</h2>
          <div className={styles.placeholder}>
            <p>Charts and analytics will be implemented here</p>
          </div>
        </div>
      </div>
    </div>
  )
}

export default DashboardPage
