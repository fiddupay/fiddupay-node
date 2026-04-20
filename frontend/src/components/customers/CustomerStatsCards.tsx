import React from 'react';
import styles from "./styles/CustomerStatsCards.module.css";

interface StatsProps {
  stats: {
    total: number;
    active: number;
    flagged: number;
    recent: number;
    totalBalanceUsd: number;
  };
}

const CustomerStatsCards: React.FC<StatsProps> = ({ stats }) => {
  return (
    <section className={styles.statsGrid}>
      <div className={styles.statCard}>
        <div className={`${styles.statIcon} ${styles.primary}`}>
          <i className="fas fa-wallet"></i>
        </div>
        <div className={styles.statInfo}>
          <h3>Total Deposits</h3>
          <p className={styles.statValue}>
            {new Intl.NumberFormat('en-US', { style: 'currency', currency: 'USD' }).format(stats.totalBalanceUsd)}
          </p>
          <span className={styles.statTrend}>Sub-ledger Total</span>
        </div>
      </div>
      <div className={styles.statCard}>
        <div className={`${styles.statIcon} ${styles.success}`}>
          <i className="fas fa-users"></i>
        </div>
        <div className={styles.statInfo}>
          <h3>Total Entities</h3>
          <p className={styles.statValue}>{stats.total}</p>
        </div>
      </div>
      <div className={styles.statCard}>
        <div className={`${styles.statIcon} ${styles.success}`}>
          <i className="fas fa-user-check"></i>
        </div>
        <div className={styles.statInfo}>
          <h3>Active Accounts</h3>
          <p className={styles.statValue}>{stats.active}</p>
        </div>
      </div>
      <div className={styles.statCard}>
        <div className={`${styles.statIcon} ${styles.warning}`}>
          <i className="fas fa-flag"></i>
        </div>
        <div className={styles.statInfo}>
          <h3>Flagged</h3>
          <p className={styles.statValue}>{stats.flagged}</p>
        </div>
      </div>
      <div className={styles.statCard}>
        <div className={`${styles.statIcon} ${styles.primary}`}>
          <i className="fas fa-arrow-trend-up"></i>
        </div>
        <div className={styles.statInfo}>
          <h3>New This Week</h3>
          <p className={styles.statValue}>{stats.recent}</p>
        </div>
      </div>
    </section>
  );
};

export default CustomerStatsCards;
