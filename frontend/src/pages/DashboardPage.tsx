import React from 'react'
import { MdPayment, MdTrendingUp, MdAccountBalance, MdPending } from 'react-icons/md'
import styles from './DashboardPage.module.css'

const DashboardPage: React.FC = () => {
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
