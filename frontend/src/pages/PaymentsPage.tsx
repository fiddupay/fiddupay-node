import React, { useState, useEffect } from 'react'
import { useToast } from '@/contexts/ToastContext'
import { useLoading } from '@/contexts/LoadingContext'
import { apiService } from '@/services/api'
import { Payment, PaymentFilters } from '@/types'
import styles from './PaymentsPage.module.css'

const PaymentsPage: React.FC = () => {
  const [payments, setPayments] = useState<Payment[]>([])
  const [stats, setStats] = useState({
    totalPayments: 0,
    totalVolume: '$0.00',
    successRate: '0%'
  })
  const [filters, setFilters] = useState<PaymentFilters>({
    page: 1,
    page_size: 20
  })
  const [showCreateModal, setShowCreateModal] = useState(false)
  const [newPayment, setNewPayment] = useState({
    amount_usd: '',
    crypto_type: 'USDT_ETH',
    description: ''
  })
  
  const { showToast } = useToast()
  const { setLoading } = useLoading()

  useEffect(() => {
    loadPayments()
    loadStats()
  }, [filters])

  const loadPayments = async () => {
    setLoading(true)
    try {
      const response = await apiService.getPayments(filters)
      setPayments(response.data || [])
    } catch (error) {
      showToast('Failed to load payments', 'error')
    } finally {
      setLoading(false)
    }
  }

  const loadStats = async () => {
    try {
      const analytics = await apiService.getAnalytics()
      if (analytics) {
        setStats({
          totalPayments: analytics.total_payments || 0,
          totalVolume: `$${analytics.total_volume_usd || '0.00'}`,
          successRate: '98.5%' // Placeholder until we get proper analytics structure
        })
      }
    } catch (error) {
      console.error('Failed to load stats:', error)
    }
  }

  const handleCreatePayment = async (e: React.FormEvent) => {
    e.preventDefault()
    
    if (!newPayment.amount_usd || parseFloat(newPayment.amount_usd) <= 0) {
      showToast('Please enter a valid amount', 'error')
      return
    }

    setLoading(true)
    try {
      const payment = await apiService.createPayment({
        amount_usd: newPayment.amount_usd,
        crypto_type: newPayment.crypto_type,
        description: newPayment.description || undefined
      })
      
      setPayments(prev => [payment, ...prev])
      setShowCreateModal(false)
      setNewPayment({ amount_usd: '', crypto_type: 'USDT_ETH', description: '' })
      showToast('Payment created successfully!', 'success')
    } catch (error) {
      showToast('Failed to create payment', 'error')
    } finally {
      setLoading(false)
    }
  }

  const getStatusBadge = (status: string) => {
    const statusClasses = {
      PENDING: styles.statusPending,
      CONFIRMING: styles.statusConfirming,
      CONFIRMED: styles.statusConfirmed,
      FAILED: styles.statusFailed,
      EXPIRED: styles.statusExpired
    }
    return statusClasses[status as keyof typeof statusClasses] || styles.statusPending
  }

  const formatDate = (dateString: string) => {
    return new Date(dateString).toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit'
    })
  }

  return (
    <div className={styles.paymentsPage}>
      <div className={styles.header}>
        <div>
          <h1><i className="fas fa-credit-card"></i> Payments</h1>
          <p>Manage and track all your cryptocurrency payments</p>
        </div>
        <button 
          className={styles.createBtn}
          onClick={() => setShowCreateModal(true)}
        >
          <i className="fas fa-plus"></i>
          Create Payment
        </button>
      </div>

      <div className={styles.stats}>
        <div className={styles.statCard}>
          <div className={styles.statIcon}>
            <i className="fas fa-receipt"></i>
          </div>
          <div className={styles.statContent}>
            <h3>Total Payments</h3>
            <div className={styles.statValue}>{stats.totalPayments.toLocaleString()}</div>
          </div>
        </div>
        <div className={styles.statCard}>
          <div className={styles.statIcon}>
            <i className="fas fa-dollar-sign"></i>
          </div>
          <div className={styles.statContent}>
            <h3>Total Volume</h3>
            <div className={styles.statValue}>{stats.totalVolume}</div>
          </div>
        </div>
        <div className={styles.statCard}>
          <div className={styles.statIcon}>
            <i className="fas fa-chart-line"></i>
          </div>
          <div className={styles.statContent}>
            <h3>Success Rate</h3>
            <div className={styles.statValue}>{stats.successRate}</div>
          </div>
        </div>
      </div>

      <div className={styles.tableContainer}>
        <div className={styles.tableHeader}>
          <h2>Recent Payments</h2>
          <div className={styles.filters}>
            <select 
              value={filters.status || ''}
              onChange={(e) => setFilters(prev => ({ ...prev, status: e.target.value || undefined }))}
              className={styles.filterSelect}
            >
              <option value="">All Statuses</option>
              <option value="PENDING">Pending</option>
              <option value="CONFIRMING">Confirming</option>
              <option value="CONFIRMED">Confirmed</option>
              <option value="FAILED">Failed</option>
              <option value="EXPIRED">Expired</option>
            </select>
          </div>
        </div>
        
        <div className={styles.table}>
          <div className={styles.tableHeader}>
            <div className={styles.tableCell}><strong>Payment ID</strong></div>
            <div className={styles.tableCell}><strong>Amount</strong></div>
            <div className={styles.tableCell}><strong>Currency</strong></div>
            <div className={styles.tableCell}><strong>Status</strong></div>
            <div className={styles.tableCell}><strong>Created</strong></div>
            <div className={styles.tableCell}><strong>Actions</strong></div>
          </div>
          
          {payments.length === 0 ? (
            <div className={styles.emptyState}>
              <i className="fas fa-receipt"></i>
              <h3>No payments yet</h3>
              <p>Create your first payment to get started</p>
              <button 
                className={styles.createBtn}
                onClick={() => setShowCreateModal(true)}
              >
                Create Payment
              </button>
            </div>
          ) : (
            payments.map((payment) => (
              <div key={payment.payment_id} className={styles.tableRow}>
                <div className={styles.tableCell}>
                  <code>{payment.payment_id}</code>
                </div>
                <div className={styles.tableCell}>
                  <div className={styles.amount}>
                    <div>${payment.amount_usd}</div>
                    <small>{payment.amount} {payment.crypto_type}</small>
                  </div>
                </div>
                <div className={styles.tableCell}>
                  <span className={styles.cryptoBadge}>{payment.crypto_type}</span>
                </div>
                <div className={styles.tableCell}>
                  <span className={`${styles.statusBadge} ${getStatusBadge(payment.status)}`}>
                    {payment.status}
                  </span>
                </div>
                <div className={styles.tableCell}>
                  {formatDate(payment.created_at)}
                </div>
                <div className={styles.tableCell}>
                  <button className={styles.actionBtn} title="View Details">
                    <i className="fas fa-eye"></i>
                  </button>
                  {payment.status === 'CONFIRMED' && (
                    <button className={styles.actionBtn} title="Create Refund">
                      <i className="fas fa-undo"></i>
                    </button>
                  )}
                </div>
              </div>
            ))
          )}
        </div>
      </div>

      {/* Create Payment Modal */}
      {showCreateModal && (
        <div className={styles.modal}>
          <div className={styles.modalContent}>
            <div className={styles.modalHeader}>
              <h2><i className="fas fa-plus"></i> Create New Payment</h2>
              <button 
                className={styles.closeBtn}
                onClick={() => setShowCreateModal(false)}
              >
                <i className="fas fa-times"></i>
              </button>
            </div>
            
            <form onSubmit={handleCreatePayment} className={styles.form}>
              <div className={styles.inputGroup}>
                <label htmlFor="amount">Amount (USD)</label>
                <input
                  type="number"
                  id="amount"
                  step="0.01"
                  min="0.01"
                  value={newPayment.amount_usd}
                  onChange={(e) => setNewPayment(prev => ({ ...prev, amount_usd: e.target.value }))}
                  placeholder="100.00"
                  required
                />
              </div>
              
              <div className={styles.inputGroup}>
                <label htmlFor="crypto_type">Cryptocurrency</label>
                <select
                  id="crypto_type"
                  value={newPayment.crypto_type}
                  onChange={(e) => setNewPayment(prev => ({ ...prev, crypto_type: e.target.value }))}
                >
                  <option value="ETH">ETH (Ethereum)</option>
                  <option value="BNB">BNB (BSC)</option>
                  <option value="MATIC">MATIC (Polygon)</option>
                  <option value="ARB">ARB (Arbitrum)</option>
                  <option value="SOL">SOL (Solana)</option>
                  <option value="USDT_ETH">USDT (Ethereum)</option>
                  <option value="USDT_BSC">USDT (BSC)</option>
                  <option value="USDT_POLYGON">USDT (Polygon)</option>
                  <option value="USDT_ARBITRUM">USDT (Arbitrum)</option>
                  <option value="USDT_SPL">USDT (Solana)</option>
                </select>
              </div>
              
              <div className={styles.inputGroup}>
                <label htmlFor="description">Description (Optional)</label>
                <input
                  type="text"
                  id="description"
                  value={newPayment.description}
                  onChange={(e) => setNewPayment(prev => ({ ...prev, description: e.target.value }))}
                  placeholder="Order #12345"
                />
              </div>
              
              <div className={styles.modalActions}>
                <button 
                  type="button" 
                  className={styles.cancelBtn}
                  onClick={() => setShowCreateModal(false)}
                >
                  Cancel
                </button>
                <button type="submit" className={styles.submitBtn}>
                  <i className="fas fa-plus"></i>
                  Create Payment
                </button>
              </div>
            </form>
          </div>
        </div>
      )}
    </div>
  )
}

export default PaymentsPage
