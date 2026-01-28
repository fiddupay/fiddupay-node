import React, { useState, useEffect } from 'react'
import { useToast } from '@/contexts/ToastContext'
import { useLoading } from '@/contexts/LoadingContext'
import { merchantAPI, paymentAPI } from '@/services/apiService'
import { Payment, PaymentFilters, FeeSettingResponse } from '@/types'
import styles from './PaymentsPage.module.css'

const PaymentsPage: React.FC = () => {
  const [payments, setPayments] = useState<Payment[]>([])
  const [feeSetting, setFeeSetting] = useState<FeeSettingResponse | null>(null)
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
  const [showFeeSettingModal, setShowFeeSettingModal] = useState(false)
  const [paymentType, setPaymentType] = useState<'standard' | 'address-only'>('standard')
  const [newPayment, setNewPayment] = useState({
    amount_usd: '',
    crypto_type: 'USDT_ETH',
    description: '',
    merchant_address: ''
  })
  
  const { showToast } = useToast()
  const { setLoading } = useLoading()

  useEffect(() => {
    loadPayments()
    loadStats()
    loadFeeSetting()
  }, [filters])

  const loadFeeSetting = async () => {
    try {
      const setting = await merchantAPI.getFeeSetting()
      setFeeSetting(setting.data)
    } catch (error) {
      console.error('Failed to load fee setting:', error)
    }
  }

  const loadPayments = async () => {
    setLoading(true)
    try {
      const response = await paymentAPI.getHistory(filters)
      setPayments(response.data || [])
    } catch (error) {
      showToast('Failed to load payments', 'error')
    } finally {
      setLoading(false)
    }
  }

  const loadStats = async () => {
    try {
      const analytics = await merchantAPI.getAnalytics()
      if (analytics.data) {
        const successfulPayments = analytics.data.successful_payments || 0
        const totalPayments = analytics.data.total_payments || 0
        const successRate = totalPayments > 0 ? ((successfulPayments / totalPayments) * 100).toFixed(1) + '%' : '0%'
        
        setStats({
          totalPayments: analytics.data.total_payments || 0,
          totalVolume: `$${analytics.data.total_volume_usd || '0.00'}`,
          successRate: successRate
        })
      }
    } catch (error) {
      console.error('Failed to load stats:', error)
    }
  }

  const handleCreatePayment = async (e: React.FormEvent) => {
    e.preventDefault()
    
    // Comprehensive validation
    const amount = parseFloat(newPayment.amount_usd)
    if (!newPayment.amount_usd || isNaN(amount) || amount <= 0) {
      showToast('Please enter a valid amount greater than 0', 'error')
      return
    }

    if (amount < 0.01) {
      showToast('Minimum payment amount is $0.01', 'error')
      return
    }

    if (amount > 100000) {
      showToast('Maximum payment amount is $100,000', 'error')
      return
    }

    if (!newPayment.crypto_type) {
      showToast('Please select a cryptocurrency', 'error')
      return
    }

    if (paymentType === 'address-only' && !newPayment.merchant_address) {
      showToast('Please enter your wallet address for address-only payments', 'error')
      return
    }

    if (paymentType === 'address-only' && newPayment.merchant_address) {
      // Basic address validation
      const address = newPayment.merchant_address.trim()
      if (newPayment.crypto_type === 'SOL' && (address.length < 32 || address.length > 44)) {
        showToast('Invalid Solana address format', 'error')
        return
      }
      if (newPayment.crypto_type.includes('ETH') && (!address.startsWith('0x') || address.length !== 42)) {
        showToast('Invalid Ethereum address format', 'error')
        return
      }
    }

    if (newPayment.description && newPayment.description.length > 500) {
      showToast('Description must be less than 500 characters', 'error')
      return
    }

    setLoading(true)
    try {
      if (paymentType === 'address-only') {
        if (!newPayment.merchant_address) {
          showToast('Merchant address is required for address-only payments', 'error')
          return
        }
        const payment = await paymentAPI.create({
          requested_amount: newPayment.amount_usd,
          crypto_type: newPayment.crypto_type,
          merchant_address: newPayment.merchant_address,
          description: newPayment.description || undefined
        })
        console.log('Address-only payment created:', payment.data.payment_id)
        showToast('Address-only payment created successfully!', 'success')
      } else {
        const payment = await paymentAPI.create({
          amount_usd: newPayment.amount_usd,
          crypto_type: newPayment.crypto_type,
          description: newPayment.description || undefined
        })
        setPayments(prev => [payment.data, ...prev])
        showToast('Payment created successfully!', 'success')
      }
      
      setShowCreateModal(false)
      setNewPayment({ amount_usd: '', crypto_type: 'USDT_ETH', description: '', merchant_address: '' })
      loadPayments()
    } catch (error) {
      showToast('Failed to create payment', 'error')
    } finally {
      setLoading(false)
    }
  }

  const handleUpdateFeeSetting = async (customerPaysFee: boolean) => {
    setLoading(true)
    try {
      await merchantAPI.updateFeeSetting({ fee_percentage: feeSetting?.fee_percentage || 0.75 })
      showToast(`Fee setting updated: ${customerPaysFee ? 'Customer pays fee' : 'Merchant pays fee'}`, 'success')
      setShowFeeSettingModal(false)
      loadFeeSetting()
    } catch (error) {
      showToast('Failed to update fee setting', 'error')
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
        <div className={styles.headerActions}>
          <button 
            className={styles.feeSettingBtn}
            onClick={() => setShowFeeSettingModal(true)}
          >
            <i className="fas fa-cog"></i>
            Fee Settings
          </button>
          <button 
            className={styles.createBtn}
            onClick={() => setShowCreateModal(true)}
          >
            <i className="fas fa-plus"></i>
            Create Payment
          </button>
        </div>
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
        {feeSetting && (
          <div className={styles.statCard}>
            <div className={styles.statIcon}>
              <i className="fas fa-percentage"></i>
            </div>
            <div className={styles.statContent}>
              <h3>Fee Model</h3>
              <div className={styles.statValue}>
                {feeSetting?.customer_pays_fee ? 'Customer Pays' : 'Merchant Pays'}
              </div>
            </div>
          </div>
        )}
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
                <label>Payment Type</label>
                <div className={styles.radioGroup}>
                  <label>
                    <input
                      type="radio"
                      value="standard"
                      checked={paymentType === 'standard'}
                      onChange={(e) => setPaymentType(e.target.value as 'standard' | 'address-only')}
                    />
                    Standard Payment
                  </label>
                  <label>
                    <input
                      type="radio"
                      value="address-only"
                      checked={paymentType === 'address-only'}
                      onChange={(e) => setPaymentType(e.target.value as 'standard' | 'address-only')}
                    />
                    Address-Only Payment
                  </label>
                </div>
              </div>

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

              {paymentType === 'address-only' && (
                <div className={styles.inputGroup}>
                  <label htmlFor="merchant_address">Merchant Address *</label>
                  <input
                    type="text"
                    id="merchant_address"
                    value={newPayment.merchant_address}
                    onChange={(e) => setNewPayment(prev => ({ ...prev, merchant_address: e.target.value }))}
                    placeholder="0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb"
                    required={paymentType === 'address-only'}
                  />
                </div>
              )}
              
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

      {/* Fee Setting Modal */}
      {showFeeSettingModal && (
        <div className={styles.modal}>
          <div className={styles.modalContent}>
            <div className={styles.modalHeader}>
              <h2>Fee Settings</h2>
              <button 
                className={styles.closeBtn}
                onClick={() => setShowFeeSettingModal(false)}
              >
                <i className="fas fa-times"></i>
              </button>
            </div>
            
            <div className={styles.feeSettingOptions}>
              <p>Choose who pays the processing fee:</p>
              
              <div className={styles.feeOption}>
                <button
                  className={`${styles.feeOptionBtn} ${feeSetting?.customer_pays_fee ? styles.active : ''}`}
                  onClick={() => handleUpdateFeeSetting(true)}
                >
                  <div className={styles.feeOptionIcon}>
                    <i className="fas fa-user"></i>
                  </div>
                  <div className={styles.feeOptionContent}>
                    <h3>Customer Pays Fee</h3>
                    <p>Customer pays the requested amount plus processing fee</p>
                  </div>
                </button>
              </div>

              <div className={styles.feeOption}>
                <button
                  className={`${styles.feeOptionBtn} ${!feeSetting?.customer_pays_fee ? styles.active : ''}`}
                  onClick={() => handleUpdateFeeSetting(false)}
                >
                  <div className={styles.feeOptionIcon}>
                    <i className="fas fa-store"></i>
                  </div>
                  <div className={styles.feeOptionContent}>
                    <h3>Merchant Pays Fee</h3>
                    <p>Customer pays the requested amount, fee deducted from merchant</p>
                  </div>
                </button>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}

export default PaymentsPage
