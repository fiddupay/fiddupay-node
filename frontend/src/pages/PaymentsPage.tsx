import React, { useState, useEffect } from 'react'
import { useToast } from '@/contexts/ToastContext'
import { merchantAPI, paymentAPI, publicAPI } from '@/services/apiService'
import { useAuthStore } from '@/stores/authStore'
import { Payment, PaymentFilters } from '@/types'
import styles from '@/styles/pages/PaymentsPage.module.css'

const PaymentsPage: React.FC = () => {
  const [payments, setPayments] = useState<Payment[]>([])
  const [supportedCryptos, setSupportedCryptos] = useState<any[]>([])
  const [loading, setLoading] = useState(true)
  const [stats, setStats] = useState({
    totalPayments: 0,
    totalVolume: '$0.00',
    successRate: '0%'
  })
  const [pagination, setPagination] = useState({
    page: 1,
    page_size: 20,
    total_pages: 1,
    total_count: 0
  })
  const [filters, setFilters] = useState<PaymentFilters>({
    page: 1,
    page_size: 20
  })
  const [showCreateModal, setShowCreateModal] = useState(false)
  const [paymentType, setPaymentType] = useState<'standard' | 'address-only'>('standard')
  const [newPayment, setNewPayment] = useState({
    amount_usd: '',
    crypto_type: '',
    description: '',
    merchant_address: '',
    // Invoicing fields
    is_invoice: false,
    customer_name: '',
    customer_email: '',
    notes: '',
    tax_percentage: '0',
    items: [{ description: '', quantity: 1, unit_price: '' }]
  })
  const [createdPayment, setCreatedPayment] = useState<Payment | null>(null)
  const [showSuccessModal, setShowSuccessModal] = useState(false)
  const [selectedPaymentForDetails, setSelectedPaymentForDetails] = useState<Payment | null>(null)

  const { showToast } = useToast()
  const { user } = useAuthStore()

  useEffect(() => {
    loadPayments()
    loadStats()
    loadSupportedCurrencies()
  }, [filters, user?.sandbox_mode])

  const loadSupportedCurrencies = async () => {
    try {
      const response = await publicAPI.getSupportedCurrencies(user?.id)
      const groups = response.data.currency_groups
      const flattenedCurrencies = Object.values(groups).flat() as any[]
      setSupportedCryptos(flattenedCurrencies)

      // Auto-select first available currency
      if (flattenedCurrencies.length > 0) {
        setNewPayment(prev => ({ ...prev, crypto_type: flattenedCurrencies[0].crypto_type }))
      }
    } catch (error) {
      console.error('Failed to load supported currencies', error)
    }
  }

  const loadPayments = async () => {
    setLoading(true)
    try {
      const response = await paymentAPI.getHistory(filters)
      setPayments(response.data.data || [])
      if (response.data.pagination) {
        setPagination(response.data.pagination)
      }
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

    if (newPayment.description && newPayment.description.length > 500) {
      showToast('Description must be less than 500 characters', 'error')
      return
    }

    setLoading(true)
    try {
      if (paymentType === 'address-only') {
        await paymentAPI.create({
          requested_amount: newPayment.amount_usd,
          crypto_type: newPayment.crypto_type,
          merchant_address: newPayment.merchant_address,
          description: newPayment.description || undefined
        })
        showToast('Address-only payment created successfully!', 'success')
      } else {
        const payment = await paymentAPI.create({
          amount_usd: newPayment.amount_usd,
          crypto_type: newPayment.crypto_type,
          description: newPayment.description || undefined,
          is_invoice: newPayment.is_invoice,
          customer_name: newPayment.is_invoice ? newPayment.customer_name : undefined,
          customer_email: newPayment.is_invoice ? newPayment.customer_email : undefined,
          notes: newPayment.is_invoice ? newPayment.notes : undefined,
          tax_percentage: newPayment.is_invoice ? parseFloat(newPayment.tax_percentage) : undefined,
          items: newPayment.is_invoice ? newPayment.items.map(item => ({
            ...item,
            unit_price: parseFloat(item.unit_price as any)
          })) : undefined
        })
        setPayments(prev => [payment.data, ...prev])
        setCreatedPayment(payment.data)
        setShowSuccessModal(true)
        showToast('Payment created successfully!', 'success')
      }

      setShowCreateModal(false)
      setFilters(prev => ({ ...prev, page: 1 }))
      setNewPayment({
        amount_usd: '',
        crypto_type: 'USDT_ETH',
        description: '',
        merchant_address: '',
        is_invoice: false,
        customer_name: '',
        customer_email: '',
        notes: '',
        tax_percentage: '0',
        items: [{ description: '', quantity: 1, unit_price: '' }]
      })
      loadPayments()
    } catch (error) {
      showToast('Failed to create payment', 'error')
    } finally {
      setLoading(false)
    }
  }

  const handleCancelPayment = async (paymentId: string) => {
    if (!window.confirm('Are you sure you want to cancel this payment link? This cannot be undone.')) {
      return
    }

    try {
      setLoading(true)
      await paymentAPI.cancel(paymentId)
      showToast('Payment cancelled successfully', 'success')
      loadPayments()
    } catch (error) {
      showToast('Failed to cancel payment', 'error')
    } finally {
      setLoading(false)
    }
  }

  const handleAddItem = () => {
    setNewPayment(prev => ({
      ...prev,
      items: [...prev.items, { description: '', quantity: 1, unit_price: '' }]
    }))
  }

  const handleRemoveItem = (index: number) => {
    setNewPayment(prev => ({
      ...prev,
      items: prev.items.filter((_, i) => i !== index)
    }))
  }

  const handleItemChange = (index: number, field: string, value: any) => {
    setNewPayment(prev => {
      const newItems = [...prev.items]
      newItems[index] = { ...newItems[index], [field]: value }

      // Auto-update total amount if items exist
      if (prev.is_invoice) {
        let total = 0
        newItems.forEach(item => {
          const price = parseFloat(item.unit_price as any) || 0
          total += price * item.quantity
        })
        // Add tax
        const taxVal = parseFloat(prev.tax_percentage) || 0
        total = total * (1 + taxVal / 100)

        return {
          ...prev,
          items: newItems,
          amount_usd: total.toFixed(2)
        }
      }

      return { ...prev, items: newItems }
    })
  }

  const getStatusBadge = (status: string) => {
    const statusClasses = {
      PENDING: styles.statusPending,
      CONFIRMING: styles.statusConfirming,
      CONFIRMED: styles.statusConfirmed,
      FAILED: styles.statusFailed,
      EXPIRED: styles.statusExpired,
      CANCELLED: styles.statusCancelled
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
            <i className="fas fa-dollar-sign"></i>
          </div>
          <div className={styles.statContent}>
            <h3>Total Volume</h3>
            <div className={styles.statValue}>{stats.totalVolume}</div>
          </div>
        </div>
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
              onChange={(e) => setFilters(prev => ({ ...prev, status: e.target.value || undefined, page: 1 }))}
              className={styles.filterSelect}
            >
              <option value="">All Statuses</option>
              <option value="PENDING">Pending</option>
              <option value="CONFIRMING">Confirming</option>
              <option value="CONFIRMED">Confirmed</option>
              <option value="FAILED">Failed</option>
              <option value="EXPIRED">Expired</option>
              <option value="CANCELLED">Cancelled</option>
            </select>
          </div>
        </div>

        <div className={styles.table}>
          <div className={`${styles.tableHeader} ${styles.tableRow}`}>
            <div className={styles.tableCell}><strong>Payment / Link</strong></div>
            <div className={styles.tableCell}><strong>Amount</strong></div>
            <div className={styles.tableCell}><strong>Currency</strong></div>
            <div className={styles.tableCell}><strong>Status</strong></div>
            <div className={styles.tableCell}><strong>Created</strong></div>
            <div className={styles.tableCell}><strong>Actions</strong></div>
          </div>

          {loading ? (
            <div className={styles.loadingState}>
              <i className="fas fa-spinner fa-spin"></i>
              <p>Loading payments...</p>
            </div>
          ) : payments.length === 0 ? (
            <div className={styles.emptyState}>
              <i className="fas fa-receipt"></i>
              <h3>No payments yet</h3>
              <p>Create your first payment using the button at the top right</p>
            </div>
          ) : (
            payments.map((payment) => (
              <div key={payment.payment_id} className={styles.tableRow}>
                <div className={styles.tableCell}>
                  <div className={styles.paymentInfoCell}>
                    <code>{payment.payment_id.slice(0, 12)}...</code>
                    {payment.payment_link && (
                      <>
                        <a
                          href={payment.payment_link}
                          target="_blank"
                          rel="noopener noreferrer"
                          title="Open Payment Page"
                          className={styles.externalLinkBtn}
                        >
                          <i className="fas fa-external-link-alt"></i>
                        </a>
                        <button
                          className={styles.copyButton}
                          onClick={() => {
                            navigator.clipboard.writeText(payment.payment_link!)
                            showToast('Link copied!', 'success')
                          }}
                          title="Copy Link"
                        >
                          <i className="fas fa-copy"></i>
                        </button>
                      </>
                    )}
                  </div>
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
                  <button
                    className={styles.actionBtn}
                    title="View Details"
                    onClick={() => setSelectedPaymentForDetails(payment)}
                  >
                    <i className="fas fa-eye"></i>
                  </button>
                  {(payment.status === 'PENDING' || payment.status === 'SELECTION_REQUIRED') && (
                    <button
                      className={`${styles.actionBtn} ${styles.cancelActionBtn}`}
                      title="Cancel Payment"
                      onClick={() => handleCancelPayment(payment.payment_id)}
                      disabled={loading}
                    >
                      <i className="fas fa-times-circle"></i>
                    </button>
                  )}
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

        {pagination.total_pages > 1 && (
          <div className={styles.pagination}>
            <div className={styles.paginationInfo}>
              Showing <strong>{((pagination.page - 1) * pagination.page_size) + 1}</strong> to <strong>{Math.min(pagination.page * pagination.page_size, pagination.total_count)}</strong> of <strong>{pagination.total_count}</strong> payments
            </div>
            <div className={styles.paginationControls}>
              <button
                className={styles.pageBtn}
                disabled={pagination.page <= 1 || loading}
                onClick={() => setFilters(prev => ({ ...prev, page: pagination.page - 1 }))}
              >
                <i className="fas fa-chevron-left"></i> Previous
              </button>

              <div className={styles.pageNumbers}>
                {[...Array(pagination.total_pages)].map((_, i) => {
                  const p = i + 1;
                  // Only show current page, 1, last page, and neighbors
                  if (
                    p === 1 ||
                    p === pagination.total_pages ||
                    Math.abs(p - pagination.page) <= 1
                  ) {
                    return (
                      <button
                        key={p}
                        className={`${styles.pageNumber} ${pagination.page === p ? styles.activePage : ''}`}
                        onClick={() => setFilters(prev => ({ ...prev, page: p }))}
                        disabled={loading}
                      >
                        {p}
                      </button>
                    )
                  } else if (
                    (p === 2 && pagination.page > 3) ||
                    (p === pagination.total_pages - 1 && pagination.page < pagination.total_pages - 2)
                  ) {
                    return <span key={p} className={styles.paginationEllipsis}>...</span>
                  }
                  return null;
                })}
              </div>

              <button
                className={styles.pageBtn}
                disabled={pagination.page >= pagination.total_pages || loading}
                onClick={() => setFilters(prev => ({ ...prev, page: pagination.page + 1 }))}
              >
                Next <i className="fas fa-chevron-right"></i>
              </button>
            </div>
          </div>
        )}
      </div>

      {/* Payment Details Modal */}
      {selectedPaymentForDetails && (
        <div className={styles.modal} onClick={() => setSelectedPaymentForDetails(null)}>
          <div className={styles.modalContent} onClick={e => e.stopPropagation()}>
            <div className={styles.modalHeader}>
              <h2><i className="fas fa-receipt"></i> Payment Details</h2>
              <button
                className={styles.closeBtn}
                onClick={() => setSelectedPaymentForDetails(null)}
              >
                <i className="fas fa-times"></i>
              </button>
            </div>

            <div className={styles.detailsContent}>
              <div className={styles.detailsGrid}>
                <div className={styles.detailItem}>
                  <label>Status</label>
                  <span className={`${styles.statusBadge} ${getStatusBadge(selectedPaymentForDetails.status)}`}>
                    {selectedPaymentForDetails.status}
                  </span>
                </div>
                <div className={styles.detailItem}>
                  <label>Payment ID</label>
                  <code>{selectedPaymentForDetails.payment_id}</code>
                </div>
                <div className={styles.detailItem}>
                  <label>Created At</label>
                  <span>{formatDate(selectedPaymentForDetails.created_at)}</span>
                </div>
                {selectedPaymentForDetails.confirmed_at && (
                  <div className={styles.detailItem}>
                    <label>Confirmed At</label>
                    <span>{formatDate(selectedPaymentForDetails.confirmed_at)}</span>
                  </div>
                )}
                <div className={styles.detailItem}>
                  <label>Network</label>
                  <span className={styles.cryptoBadge}>{selectedPaymentForDetails.network}</span>
                </div>
                <div className={styles.detailItem}>
                  <label>Currency</label>
                  <span className={styles.cryptoBadge}>{selectedPaymentForDetails.crypto_type || 'USDC'}</span>
                </div>
              </div>

              <div className={styles.detailsSection}>
                <h3>Amount Breakdown</h3>
                <div className={styles.amountBreakdown}>
                  <div className={styles.amountRow}>
                    <span>Requested Amount:</span>
                    <strong>${selectedPaymentForDetails.amount_usd}</strong>
                  </div>
                  <div className={styles.amountRow}>
                    <span>Crypto Amount:</span>
                    <span>{selectedPaymentForDetails.amount} {selectedPaymentForDetails.crypto_type}</span>
                  </div>
                  <div className={styles.amountRow}>
                    <span>Processing Fee:</span>
                    <span>${selectedPaymentForDetails.fee_amount_usd} ({selectedPaymentForDetails.fee_amount} {selectedPaymentForDetails.crypto_type})</span>
                  </div>
                </div>
              </div>

              {selectedPaymentForDetails.description && (
                <div className={styles.detailsSection}>
                  <h3>Description</h3>
                  <p className={styles.descriptionText}>{selectedPaymentForDetails.description}</p>
                </div>
              )}

              {selectedPaymentForDetails.deposit_address && (
                <div className={styles.detailsSection}>
                  <h3>Deposit Address</h3>
                  <div className={styles.addressBox}>
                    <code>{selectedPaymentForDetails.deposit_address}</code>
                    <button
                      className={styles.copyButton}
                      onClick={() => {
                        navigator.clipboard.writeText(selectedPaymentForDetails.deposit_address)
                        showToast('Address copied!', 'success')
                      }}
                    >
                      <i className="fas fa-copy"></i>
                    </button>
                  </div>
                </div>
              )}

              {selectedPaymentForDetails.transaction_hash && (
                <div className={styles.detailsSection}>
                  <h3>Transaction</h3>
                  <div className={styles.addressBox}>
                    <code className={styles.truncatedCode}>{selectedPaymentForDetails.transaction_hash}</code>
                    <a
                      href={`https://solscan.io/tx/${selectedPaymentForDetails.transaction_hash}`}
                      target="_blank"
                      rel="noopener noreferrer"
                      className={styles.externalLinkBtn}
                    >
                      <i className="fas fa-external-link-alt"></i>
                    </a>
                  </div>
                </div>
              )}
            </div>

            <div className={styles.modalActions}>
              <button
                className={styles.cancelBtn}
                onClick={() => setSelectedPaymentForDetails(null)}
              >
                Close
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Create Payment Modal */}
      {
        showCreateModal && (
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
                    {supportedCryptos.map((crypto: any) => (
                      <option key={crypto.crypto_type} value={crypto.crypto_type}>
                        {crypto.crypto_type.split('_')[0]} ({crypto.network})
                      </option>
                    ))}
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
                    placeholder="Product name, order #, etc."
                  />
                </div>

                {paymentType === 'standard' && (
                  <div className={styles.invoicingSection}>
                    <div className={styles.toggleGroup}>
                      <div className={styles.toggleLabel}>
                        <label>Create as Invoice</label>
                        <small>Include professional itemized breakdown and customer details</small>
                      </div>
                      <label className={styles.switch}>
                        <input
                          type="checkbox"
                          checked={newPayment.is_invoice}
                          onChange={(e) => setNewPayment(prev => ({ ...prev, is_invoice: e.target.checked }))}
                        />
                        <span className={styles.slider}></span>
                      </label>
                    </div>

                    {newPayment.is_invoice && (
                      <div className={styles.invoiceFields}>
                        <div className={styles.row}>
                          <div className={styles.inputGroup}>
                            <label>Customer Name</label>
                            <input
                              type="text"
                              value={newPayment.customer_name}
                              onChange={(e) => setNewPayment(prev => ({ ...prev, customer_name: e.target.value }))}
                              placeholder="John Doe"
                              required={newPayment.is_invoice}
                            />
                          </div>
                          <div className={styles.inputGroup}>
                            <label>Customer Email</label>
                            <input
                              type="email"
                              value={newPayment.customer_email}
                              onChange={(e) => setNewPayment(prev => ({ ...prev, customer_email: e.target.value }))}
                              placeholder="john@example.com"
                              required={newPayment.is_invoice}
                            />
                          </div>
                        </div>

                        <div className={styles.itemsSection}>
                          <div className={styles.itemsHeader}>
                            <label>Line Items</label>
                            <button type="button" onClick={handleAddItem} className={styles.addBtn}>
                              <i className="fas fa-plus"></i> Add Item
                            </button>
                          </div>
                          <div className={styles.itemsTable}>
                            <div className={styles.itemTableHeader}>
                              <span>Description</span>
                              <span>Qty</span>
                              <span>Price</span>
                              <span></span>
                            </div>
                            {newPayment.items.map((item, index) => (
                              <div key={index} className={styles.itemRow}>
                                <input
                                  type="text"
                                  placeholder="Service or product description"
                                  value={item.description}
                                  onChange={(e) => handleItemChange(index, 'description', e.target.value)}
                                  className={styles.itemDesc}
                                  required={newPayment.is_invoice}
                                />
                                <input
                                  type="number"
                                  placeholder="1"
                                  value={item.quantity}
                                  onChange={(e) => handleItemChange(index, 'quantity', parseInt(e.target.value))}
                                  className={styles.itemQty}
                                  min="1"
                                  required={newPayment.is_invoice}
                                />
                                <input
                                  type="number"
                                  placeholder="0.00"
                                  value={item.unit_price}
                                  onChange={(e) => handleItemChange(index, 'unit_price', e.target.value)}
                                  className={styles.itemPrice}
                                  step="0.01"
                                  min="0"
                                  required={newPayment.is_invoice}
                                />
                                {newPayment.items.length > 1 && (
                                  <button type="button" onClick={() => handleRemoveItem(index)} className={styles.removeBtn} title="Remove Item">
                                    <i className="fas fa-trash"></i>
                                  </button>
                                )}
                              </div>
                            ))}
                          </div>
                        </div>

                        <div className={styles.invoiceFooter}>
                          <div className={styles.inputGroup}>
                            <label>Tax Percentage (%)</label>
                            <input
                              type="number"
                              value={newPayment.tax_percentage}
                              onChange={(e) => {
                                const val = e.target.value
                                setNewPayment(prev => ({ ...prev, tax_percentage: val }))
                              }}
                              placeholder="0"
                            />
                          </div>

                          <div className={styles.inputGroup}>
                            <label>Notes</label>
                            <textarea
                              value={newPayment.notes}
                              onChange={(e) => setNewPayment(prev => ({ ...prev, notes: e.target.value }))}
                              placeholder="Thank you for your business!"
                            />
                          </div>
                        </div>
                      </div>
                    )}
                  </div>
                )}

                <div className={styles.modalActions}>
                  <button
                    type="button"
                    className={styles.cancelBtn}
                    onClick={() => setShowCreateModal(false)}
                    disabled={loading}
                  >
                    Cancel
                  </button>
                  <button
                    type="submit"
                    className={styles.submitBtn}
                    disabled={loading}
                  >
                    {loading ? (
                      <><i className="fas fa-spinner fa-spin"></i> Creating...</>
                    ) : (
                      <><i className="fas fa-check"></i> Create Payment</>
                    )}
                  </button>
                </div>
              </form>
            </div>
          </div>
        )
      }

      {/* Success Modal */}
      {
        showSuccessModal && createdPayment && (
          <div className={styles.modal}>
            <div className={styles.modalContent}>
              <div className={styles.modalHeader}>
                <h2><i className="fas fa-check-circle" style={{ color: 'var(--fiddu-success)' }}></i> Payment Created</h2>
                <button
                  className={styles.closeBtn}
                  onClick={() => setShowSuccessModal(false)}
                >
                  <i className="fas fa-times"></i>
                </button>
              </div>
              <div className={styles.successBody}>
                <p>Payment <strong>{createdPayment.payment_id}</strong> has been created successfully.</p>

                <div className={styles.linkCard}>
                  <label>Shareable Payment Link</label>
                  <div className={styles.linkWrapper}>
                    <input
                      type="text"
                      readOnly
                      value={createdPayment.payment_link}
                      className={styles.linkInput}
                    />
                    <button
                      className={styles.copyBtn}
                      onClick={() => {
                        navigator.clipboard.writeText(createdPayment.payment_link)
                        showToast('Link copied to clipboard!', 'success')
                      }}
                    >
                      <i className="fas fa-copy"></i> Copy
                    </button>
                    <a
                      href={createdPayment.payment_link}
                      target="_blank"
                      rel="noopener noreferrer"
                      className={styles.openBtn}
                    >
                      <i className="fas fa-external-link-alt"></i> Open
                    </a>
                  </div>
                </div>

                <div className={styles.modalActions} style={{ marginTop: '24px' }}>
                  <button
                    className={styles.submitBtn}
                    onClick={() => setShowSuccessModal(false)}
                  >
                    Got it
                  </button>
                </div>
              </div>
            </div>
          </div>
        )
      }

    </div >
  )
}

export default PaymentsPage
