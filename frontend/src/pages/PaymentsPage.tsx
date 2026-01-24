import React from 'react'
import styles from './PaymentsPage.module.css'

const PaymentsPage: React.FC = () => {
  return (
    <div className={styles.paymentsPage}>
      <div className={styles.header}>
        <h1>Payments</h1>
        <p>Manage and track all your cryptocurrency payments</p>
      </div>

      <div className={styles.stats}>
        <div className={styles.statCard}>
          <h3>Total Payments</h3>
          <div className={styles.statValue}>1,234</div>
          <div className={styles.statChange}>+12% from last month</div>
        </div>
        <div className={styles.statCard}>
          <h3>Total Volume</h3>
          <div className={styles.statValue}>$45,678</div>
          <div className={styles.statChange}>+8% from last month</div>
        </div>
        <div className={styles.statCard}>
          <h3>Success Rate</h3>
          <div className={styles.statValue}>98.5%</div>
          <div className={styles.statChange}>+0.3% from last month</div>
        </div>
      </div>

      <div className={styles.tableContainer}>
        <div className={styles.tableHeader}>
          <h2>Recent Payments</h2>
          <button className={styles.createBtn}>Create Payment</button>
        </div>
        
        <div className={styles.table}>
          <div className={styles.tableRow}>
            <div className={styles.tableCell}>
              <strong>Payment ID</strong>
            </div>
            <div className={styles.tableCell}>
              <strong>Amount</strong>
            </div>
            <div className={styles.tableCell}>
              <strong>Currency</strong>
            </div>
            <div className={styles.tableCell}>
              <strong>Status</strong>
            </div>
            <div className={styles.tableCell}>
              <strong>Date</strong>
            </div>
          </div>
          
          <div className={styles.tableRow}>
            <div className={styles.tableCell}>pay_1234567890</div>
            <div className={styles.tableCell}>$100.00</div>
            <div className={styles.tableCell}>USDT</div>
            <div className={styles.tableCell}>
              <span className={styles.statusConfirmed}>Confirmed</span>
            </div>
            <div className={styles.tableCell}>Jan 24, 2026</div>
          </div>
          
          <div className={styles.tableRow}>
            <div className={styles.tableCell}>pay_0987654321</div>
            <div className={styles.tableCell}>$250.00</div>
            <div className={styles.tableCell}>SOL</div>
            <div className={styles.tableCell}>
              <span className={styles.statusPending}>Pending</span>
            </div>
            <div className={styles.tableCell}>Jan 24, 2026</div>
          </div>
        </div>
      </div>
    </div>
  )
}

export default PaymentsPage
