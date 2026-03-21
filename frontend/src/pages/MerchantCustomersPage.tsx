import React, { useState, useEffect, useMemo } from 'react'
import { customerAPI, publicAPI } from '@/services/apiService'
import styles from '@/styles/pages/MerchantCustomersPage.module.css'
import { useToast } from '@/contexts/ToastContext'

interface Customer {
    id: string;
    merchant_id: string;
    external_id: string;
    email?: string;
    first_name?: string;
    last_name?: string;
    is_active: boolean;
    status: string;
    status_reason?: string;
    can_withdraw: boolean;
    withdrawal_limit?: string;
    created_at: string;
}

interface Wallet {
    crypto_type: string;
    network: string;
    address: string;
    created_at: string;
}

interface CustomerTx {
    id: string;
    type: string;
    crypto_type: string;
    amount: string;
    fee: string;
    status: string;
    destination_address?: string;
    transaction_hash?: string;
    reference_id?: string;
    description?: string;
    created_at: string;
}

const STATUS_STYLES: Record<string, { color: string; bg: string; icon: string; label: string }> = {
    active:    { color: '#059669', bg: '#d1fae5', icon: 'fa-check-circle', label: 'Active' },
    flagged:   { color: '#d97706', bg: '#fef3c7', icon: 'fa-flag',         label: 'Flagged' },
    suspended: { color: '#dc2626', bg: '#fee2e2', icon: 'fa-pause-circle', label: 'Suspended' },
    blocked:   { color: '#6b7280', bg: '#f3f4f6', icon: 'fa-ban',          label: 'Blocked' },
}

const TX_BADGES: Record<string, { color: string; bg: string; icon: string }> = {
    WITHDRAWAL:        { color: '#dc2626', bg: '#fee2e2', icon: 'fa-arrow-up' },
    MERCHANT_PAYMENT:  { color: '#7c3aed', bg: '#ede9fe', icon: 'fa-shopping-cart' },
    SWEEP:             { color: '#2563eb', bg: '#dbeafe', icon: 'fa-broom' },
    DEPOSIT:           { color: '#059669', bg: '#d1fae5', icon: 'fa-arrow-down' },
}

const MerchantCustomersPage: React.FC = () => {
    const { showToast } = useToast()
    const [customers, setCustomers] = useState<Customer[]>([])
    const [loading, setLoading] = useState(true)
    const [searchTerm, setSearchTerm] = useState('')
    const [statusFilter, setStatusFilter] = useState<string>('all')

    // Drawer States
    const [isCreateDrawerOpen, setIsCreateDrawerOpen] = useState(false)
    const [selectedCustomer, setSelectedCustomer] = useState<Customer | null>(null)
    const [drawerTab, setDrawerTab] = useState<'overview' | 'transactions' | 'permissions'>('overview')
    
    // Form States
    const [newCustomer, setNewCustomer] = useState({ external_id: '', email: '', first_name: '', last_name: '' })
    const [submitting, setSubmitting] = useState(false)
    const [detailsLoading, setDetailsLoading] = useState(false)
    const [customerWallets, setCustomerWallets] = useState<Wallet[]>([])
    const [customerBalances, setCustomerBalances] = useState<any>(null)
    const [customerTransactions, setCustomerTransactions] = useState<CustomerTx[]>([])
    const [sweepCryptoType, setSweepCryptoType] = useState('USDT')
    const [sweepAmount, setSweepAmount] = useState('')
    const [sweeping, setSweeping] = useState(false)

    // Status update states
    const [statusUpdating, setStatusUpdating] = useState(false)
    const [statusReason, setStatusReason] = useState('')
    const [showStatusModal, setShowStatusModal] = useState<string | null>(null) // target status

    // Permission states
    const [permUpdating, setPermUpdating] = useState(false)
    const [supportedCurrencies, setSupportedCurrencies] = useState<any[]>([])

    useEffect(() => {
        fetchCustomers()
        fetchSupportedCurrencies()
    }, [])

    const fetchSupportedCurrencies = async () => {
        try {
            const res = await publicAPI.getSupportedCurrencies()
            if (res.data?.currency_groups) {
                const flattened = Object.values(res.data.currency_groups).flat() as any[]
                setSupportedCurrencies(flattened)
                if (flattened.length > 0 && !sweepCryptoType) {
                    setSweepCryptoType(flattened[0].crypto_type)
                }
            }
        } catch (err) {
            console.error('Failed to fetch currencies', err)
        }
    }

    const fetchCustomers = async () => {
        try {
            setLoading(true)
            const res = await customerAPI.list()
            if (res.data?.customers) {
                setCustomers(res.data.customers)
            }
        } catch (error: any) {
            showToast(error.response?.data?.error || error.message || 'Failed to list customers', 'error')
        } finally {
            setLoading(false)
        }
    }

    const stats = useMemo(() => {
        const total = customers.length
        const active = customers.filter(c => c.status === 'active' && c.is_active).length
        const flagged = customers.filter(c => c.status === 'flagged').length
        const recent = customers.filter(c => {
            const diff = Date.now() - new Date(c.created_at).getTime()
            return diff < 7 * 24 * 60 * 60 * 1000
        }).length
        return { total, active, flagged, recent }
    }, [customers])

    const filteredCustomers = useMemo(() => {
        return customers.filter(c => {
            const matchesSearch = 
                c.external_id.toLowerCase().includes(searchTerm.toLowerCase()) ||
                (c.email?.toLowerCase().includes(searchTerm.toLowerCase())) ||
                (`${c.first_name || ''} ${c.last_name || ''}`.toLowerCase().includes(searchTerm.toLowerCase()))
            
            const matchesStatus = 
                statusFilter === 'all' || c.status === statusFilter ||
                (statusFilter === 'inactive' && !c.is_active)
                
            return matchesSearch && matchesStatus
        })
    }, [customers, searchTerm, statusFilter])

    const handleCreateCustomer = async (e: React.FormEvent) => {
        e.preventDefault()
        if (!newCustomer.external_id) {
            showToast('External ID is required', 'error')
            return;
        }

        try {
            setSubmitting(true)
            await customerAPI.create(newCustomer)
            showToast('Customer registered with wallets provisioned', 'success')
            setIsCreateDrawerOpen(false)
            setNewCustomer({ external_id: '', email: '', first_name: '', last_name: '' })
            fetchCustomers()
        } catch (error: any) {
            showToast(error.response?.data?.error || error.message || 'Failed to register customer', 'error')
        } finally {
            setSubmitting(false)
        }
    }

    const openCustomerDetails = async (customer: Customer) => {
        setSelectedCustomer(customer)
        setDetailsLoading(true)
        setDrawerTab('overview')
        setCustomerWallets([])
        setCustomerBalances(null)
        setCustomerTransactions([])

        try {
            const [walletRes, balRes, txRes] = await Promise.allSettled([
                customerAPI.getWallets(customer.external_id),
                customerAPI.getBalances(customer.external_id),
                customerAPI.getTransactions(customer.external_id, { limit: 20 }),
            ])
            if (walletRes.status === 'fulfilled') setCustomerWallets(walletRes.value.data?.wallets || [])
            if (balRes.status === 'fulfilled') setCustomerBalances(balRes.value.data?.balances)
            if (txRes.status === 'fulfilled') setCustomerTransactions(txRes.value.data?.transactions || [])
        } catch { /* silent */ } finally {
            setDetailsLoading(false)
        }
    }

    const handleStatusUpdate = async (newStatus: string) => {
        if (!selectedCustomer) return;
        try {
            setStatusUpdating(true)
            const res = await customerAPI.updateStatus(selectedCustomer.external_id, { status: newStatus, reason: statusReason || undefined })
            const updated = res.data?.customer
            if (updated) {
                setSelectedCustomer(updated)
                setCustomers(prev => prev.map(c => c.id === updated.id ? updated : c))
            }
            showToast(`Customer status changed to ${newStatus}`, 'success')
            setShowStatusModal(null)
            setStatusReason('')
        } catch (error: any) {
            showToast(error.response?.data?.error || 'Failed to update status', 'error')
        } finally {
            setStatusUpdating(false)
        }
    }

    const handleToggleWithdraw = async () => {
        if (!selectedCustomer) return;
        try {
            setPermUpdating(true)
            const res = await customerAPI.updatePermissions(selectedCustomer.external_id, { can_withdraw: !selectedCustomer.can_withdraw })
            const updated = res.data?.customer
            if (updated) {
                setSelectedCustomer(updated)
                setCustomers(prev => prev.map(c => c.id === updated.id ? updated : c))
            }
            showToast(`Withdrawals ${!selectedCustomer.can_withdraw ? 'enabled' : 'disabled'}`, 'success')
        } catch (error: any) {
            showToast(error.response?.data?.error || 'Failed to update permissions', 'error')
        } finally {
            setPermUpdating(false)
        }
    }

    const handleSweep = async (e: React.FormEvent) => {
        e.preventDefault()
        if (!selectedCustomer || !sweepCryptoType || !sweepAmount) return;

        try {
            setSweeping(true)
            await customerAPI.sweep(selectedCustomer.external_id, { crypto_type: sweepCryptoType, amount: sweepAmount })
            showToast(`Successfully swept ${sweepAmount} ${sweepCryptoType}`, 'success')
            setSweepAmount('')
            const balRes = await customerAPI.getBalances(selectedCustomer.external_id)
            setCustomerBalances(balRes.data?.balances)
        } catch (error: any) {
            showToast(error.response?.data?.error || error.message || 'Failed to sweep funds', 'error')
        } finally {
            setSweeping(false)
        }
    }

    const getInitials = (c: Customer) => {
        if (c.first_name && c.last_name) return `${c.first_name[0]}${c.last_name[0]}`.toUpperCase()
        return c.external_id.substring(0, 2).toUpperCase()
    }

    const getStatusStyle = (status: string) => STATUS_STYLES[status] || STATUS_STYLES['active']

    return (
        <div className={styles.page}>
            <header className={styles.header}>
                <div className={styles.headerInfo}>
                    <h1>Customer Directory</h1>
                    <p>Manage your ecosystem of sub-accounts and dedicated wallets</p>
                </div>
                <div className={styles.headerActions}>
                    <button className={styles.addBtn} onClick={() => setIsCreateDrawerOpen(true)}>
                        <i className="fas fa-user-plus"></i>
                        Register Customer
                    </button>
                </div>
            </header>

            <section className={styles.statsGrid}>
                <div className={styles.statCard}>
                    <div className={`${styles.statIcon} ${styles.primary}`}>
                        <i className="fas fa-users"></i>
                    </div>
                    <div className={styles.statInfo}>
                        <h3>Total Customers</h3>
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
                        <i className="fas fa-sparkles"></i>
                    </div>
                    <div className={styles.statInfo}>
                        <h3>New This Week</h3>
                        <p className={styles.statValue}>{stats.recent}</p>
                    </div>
                </div>
            </section>

            <section className={styles.filterBar}>
                <div className={styles.searchWrapper}>
                    <i className="fas fa-search"></i>
                    <input 
                        className={styles.searchInput}
                        placeholder="Search ID, name, or email..."
                        value={searchTerm}
                        onChange={(e) => setSearchTerm(e.target.value)}
                    />
                </div>
                <div className={styles.filterActions}>
                    <select 
                        className={styles.filterSelect}
                        value={statusFilter}
                        onChange={(e: any) => setStatusFilter(e.target.value)}
                    >
                        <option value="all">All Statuses</option>
                        <option value="active">Active</option>
                        <option value="flagged">Flagged</option>
                        <option value="suspended">Suspended</option>
                        <option value="blocked">Blocked</option>
                    </select>
                    <button className={styles.actionBtn} style={{ background: 'white', color: '#1e293b', border: '1px solid #e2e8f0' }} onClick={fetchCustomers}>
                        <i className="fas fa-sync-alt"></i>
                    </button>
                </div>
            </section>

            <div className={styles.contentCard}>
                <div className={styles.tableHeader}>
                    <h2>Registered Entities</h2>
                    <span style={{ fontSize: '0.875rem', color: '#64748b', fontWeight: 600 }}>
                        {filteredCustomers.length} results found
                    </span>
                </div>

                {loading ? (
                    <div className={styles.loadingOverlay}>
                        <i className="fas fa-circle-notch fa-spin fa-3x"></i>
                    </div>
                ) : filteredCustomers.length === 0 ? (
                    <div className={styles.noData}>
                        <i className="fas fa-users-slash"></i>
                        <p>{searchTerm ? 'No results match your search' : 'No customers registered yet'}</p>
                    </div>
                ) : (
                    <div className={styles.tableContainer}>
                        <table className={styles.table}>
                            <thead>
                                <tr>
                                    <th>Customer Identity</th>
                                    <th>External ID</th>
                                    <th>Status</th>
                                    <th>Withdrawals</th>
                                    <th>Joined Date</th>
                                    <th style={{ textAlign: 'right' }}>Actions</th>
                                </tr>
                            </thead>
                            <tbody>
                                {filteredCustomers.map(c => {
                                    const st = getStatusStyle(c.status || 'active')
                                    return (
                                    <tr 
                                        key={c.id} 
                                        className={styles.customerRow}
                                        onClick={() => openCustomerDetails(c)}
                                    >
                                        <td>
                                            <div className={styles.customerInfo}>
                                                <div className={styles.avatar}>{getInitials(c)}</div>
                                                <div className={styles.customerMeta}>
                                                    <span className={styles.customerName}>
                                                        {c.first_name || c.last_name ? `${c.first_name || ''} ${c.last_name || ''}`.trim() : 'Unnamed Customer'}
                                                    </span>
                                                    <span className={styles.customerEmail}>{c.email || 'No email provided'}</span>
                                                </div>
                                            </div>
                                        </td>
                                        <td><span className={styles.externalId}>{c.external_id}</span></td>
                                        <td>
                                            <span style={{ display: 'inline-flex', alignItems: 'center', gap: '0.35rem', padding: '0.25rem 0.75rem', borderRadius: '999px', fontSize: '0.8rem', fontWeight: 600, color: st.color, background: st.bg }}>
                                                <i className={`fas ${st.icon}`} style={{ fontSize: '0.7rem' }}></i> {st.label}
                                            </span>
                                        </td>
                                        <td>
                                            <span style={{ color: c.can_withdraw ? '#059669' : '#dc2626', fontWeight: 600, fontSize: '0.85rem' }}>
                                                {c.can_withdraw ? '✓ Enabled' : '✗ Disabled'}
                                            </span>
                                        </td>
                                        <td>{new Date(c.created_at).toLocaleDateString(undefined, { month: 'short', day: 'numeric', year: 'numeric' })}</td>
                                        <td style={{ textAlign: 'right' }}>
                                            <button className={styles.actionBtn} style={{ padding: '0.5rem 1rem', background: '#f1f5f9', color: '#1e293b', display: 'inline-flex' }}>
                                                Manage <i className="fas fa-chevron-right ml-2"></i>
                                            </button>
                                        </td>
                                    </tr>
                                )})}
                            </tbody>
                        </table>
                    </div>
                )}
            </div>

            {/* Create Customer Drawer */}
            {isCreateDrawerOpen && (
                <div className={styles.overlay} onClick={() => setIsCreateDrawerOpen(false)}>
                    <div className={styles.drawer} onClick={e => e.stopPropagation()}>
                        <div className={styles.drawerHeader}>
                            <h2><i className="fas fa-user-plus" style={{ color: '#2563eb' }}></i> New Customer</h2>
                            <button className={styles.closeBtn} onClick={() => setIsCreateDrawerOpen(false)}>
                                <i className="fas fa-times"></i>
                            </button>
                        </div>
                        <div className={styles.drawerBody}>
                            <form onSubmit={handleCreateCustomer}>
                                <div className={styles.formGroup}>
                                    <label>External Reference ID*</label>
                                    <div className={styles.inputGroup}>
                                        <i className="fas fa-id-card"></i>
                                        <input 
                                            className={styles.inputStyle}
                                            required
                                            placeholder="e.g. system_user_99"
                                            value={newCustomer.external_id}
                                            onChange={e => setNewCustomer({ ...newCustomer, external_id: e.target.value })}
                                        />
                                    </div>
                                    <p style={{ fontSize: '0.75rem', color: '#64748b', marginTop: '0.5rem' }}>
                                        Must be unique. Wallets will be auto-provisioned upon registration.
                                    </p>
                                </div>
                                <div className={styles.formGroup}>
                                    <label>Email Address</label>
                                    <div className={styles.inputGroup}>
                                        <i className="fas fa-envelope"></i>
                                        <input 
                                            className={styles.inputStyle}
                                            type="email"
                                            placeholder="customer@domain.com"
                                            value={newCustomer.email}
                                            onChange={e => setNewCustomer({ ...newCustomer, email: e.target.value })}
                                        />
                                    </div>
                                </div>
                                <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '1.5rem' }}>
                                    <div className={styles.formGroup}>
                                        <label>First Name</label>
                                        <div className={styles.inputGroup}>
                                            <i className="fas fa-user-circle"></i>
                                            <input 
                                                className={styles.inputStyle}
                                                placeholder="John"
                                                value={newCustomer.first_name}
                                                onChange={e => setNewCustomer({ ...newCustomer, first_name: e.target.value })}
                                            />
                                        </div>
                                    </div>
                                    <div className={styles.formGroup}>
                                        <label>Last Name</label>
                                        <div className={styles.inputGroup}>
                                            <i className="fas fa-user-circle"></i>
                                            <input 
                                                className={styles.inputStyle}
                                                placeholder="Doe"
                                                value={newCustomer.last_name}
                                                onChange={e => setNewCustomer({ ...newCustomer, last_name: e.target.value })}
                                            />
                                        </div>
                                    </div>
                                </div>
                                <button className={styles.addBtn} style={{ width: '100%', marginTop: '2rem' }} disabled={submitting}>
                                    {submitting ? <i className="fas fa-circle-notch fa-spin"></i> : 'Complete Registration'}
                                </button>
                            </form>
                        </div>
                    </div>
                </div>
            )}

            {/* Customer Details Drawer */}
            {selectedCustomer && (
                <div className={styles.overlay} onClick={() => setSelectedCustomer(null)}>
                    <div className={styles.drawer} onClick={e => e.stopPropagation()}>
                        <div className={styles.drawerHeader}>
                            <div style={{ display: 'flex', alignItems: 'center', gap: '0.75rem' }}>
                                <h2><i className="fas fa-id-badge" style={{ color: '#2563eb' }}></i> {selectedCustomer.first_name || selectedCustomer.last_name ? `${selectedCustomer.first_name || ''} ${selectedCustomer.last_name || ''}`.trim() : selectedCustomer.external_id}</h2>
                                {(() => {
                                    const st = getStatusStyle(selectedCustomer.status || 'active')
                                    return <span style={{ display: 'inline-flex', alignItems: 'center', gap: '0.3rem', padding: '0.2rem 0.6rem', borderRadius: '999px', fontSize: '0.75rem', fontWeight: 700, color: st.color, background: st.bg }}>
                                        <i className={`fas ${st.icon}`} style={{ fontSize: '0.65rem' }}></i> {st.label}
                                    </span>
                                })()}
                            </div>
                            <button className={styles.closeBtn} onClick={() => setSelectedCustomer(null)}>
                                <i className="fas fa-times"></i>
                            </button>
                        </div>

                        {/* Drawer Tabs */}
                        <div style={{ display: 'flex', borderBottom: '2px solid #e2e8f0', padding: '0 2rem' }}>
                            {(['overview', 'transactions', 'permissions'] as const).map(tab => (
                                <button 
                                    key={tab}
                                    onClick={() => setDrawerTab(tab)}
                                    style={{
                                        padding: '0.75rem 1.25rem', border: 'none', background: 'none', cursor: 'pointer',
                                        fontWeight: 600, fontSize: '0.875rem', color: drawerTab === tab ? '#2563eb' : '#94a3b8',
                                        borderBottom: drawerTab === tab ? '2px solid #2563eb' : '2px solid transparent',
                                        marginBottom: '-2px', transition: 'all 0.2s', textTransform: 'capitalize',
                                    }}
                                >
                                    {tab === 'overview' && <i className="fas fa-wallet" style={{ marginRight: '0.4rem' }}></i>}
                                    {tab === 'transactions' && <i className="fas fa-exchange-alt" style={{ marginRight: '0.4rem' }}></i>}
                                    {tab === 'permissions' && <i className="fas fa-shield-alt" style={{ marginRight: '0.4rem' }}></i>}
                                    {tab}
                                </button>
                            ))}
                        </div>

                        <div className={styles.drawerBody}>
                            {detailsLoading ? (
                                <div className={styles.loadingOverlay}><i className="fas fa-circle-notch fa-spin fa-2x"></i></div>
                            ) : (
                                <>
                                    {/* =================== OVERVIEW TAB =================== */}
                                    {drawerTab === 'overview' && (
                                        <>
                                            {/* Wallets */}
                                            <div className={styles.drawerSection}>
                                                <h3><i className="fas fa-wallet" style={{ color: '#2563eb' }}></i> Dedicated Wallets</h3>
                                                {customerWallets.length > 0 ? (
                                                    <div style={{ display: 'flex', flexDirection: 'column', gap: '0.75rem' }}>
                                                                        {customerWallets.map((w, idx) => {
                                                                            const networkKey = w.network.toLowerCase().includes('ethereum') ? 'eth' : 
                                                                                             w.network.toLowerCase().includes('bep20') ? 'bsc' :
                                                                                             w.network.toLowerCase().includes('bsc') ? 'bsc' :
                                                                                             w.network.toLowerCase().includes('solana') ? 'sol' :
                                                                                             w.network.toLowerCase().includes('polygon') ? 'poly' :
                                                                                             w.network.toLowerCase().includes('arbitrum') ? 'arb' :
                                                                                             w.network.toLowerCase().includes('bitcoin') ? 'btc' : '';
                                                                            
                                                                            const assetIcon = w.crypto_type.toLowerCase().includes('usdt') ? 'fa-dollar-sign' :
                                                                                             w.crypto_type.toLowerCase().includes('eth') ? 'fa-ethereum' :
                                                                                             w.crypto_type.toLowerCase().includes('sol') ? 'fa-sun' :
                                                                                             w.crypto_type.toLowerCase().includes('bnb') ? 'fa-coins' :
                                                                                             w.crypto_type.toLowerCase().includes('btc') ? 'fa-bitcoin' :
                                                                                             w.crypto_type.toLowerCase().includes('matic') ? 'fa-polygon' : 'fa-wallet';

                                                                            return (
                                                                                <div key={idx} className={`${styles.walletItem} ${styles[networkKey]}`}>
                                                                                    <div className={styles.walletHeader}>
                                                                                        <div className={styles.walletMainInfo}>
                                                                                            <div className={styles.assetIcon}>
                                                                                                <i className={`fab ${assetIcon}`}></i>
                                                                                            </div>
                                                                                            <span className={styles.walletType}>
                                                                                                {w.crypto_type}
                                                                                                <sub>{w.crypto_type.includes('USDT') ? 'Stablecoin' : 'Native Token'}</sub>
                                                                                            </span>
                                                                                        </div>
                                                                                        <span className={styles.walletNetwork}>{w.network}</span>
                                                                                    </div>
                                                                                    <div className={styles.addressBox}>
                                                                                        <span className={styles.addressText}>{w.address}</span>
                                                                                        <button 
                                                                                            className={styles.copyBtn}
                                                                                            title="Copy Address"
                                                                                            onClick={() => { navigator.clipboard.writeText(w.address); showToast('Address copied!', 'success') }}
                                                                                        >
                                                                                            <i className="far fa-copy"></i>
                                                                                        </button>
                                                                                    </div>
                                                                                </div>
                                                                            );
                                                                        })}
                                                    </div>
                                                ) : (
                                                    <div style={{ textAlign: 'center', padding: '2rem 1rem', background: '#f8fafc', borderRadius: '12px', color: '#94a3b8' }}>
                                                        <i className="fas fa-wallet" style={{ fontSize: '1.5rem', marginBottom: '0.5rem', display: 'block' }}></i>
                                                        <p style={{ margin: 0, fontSize: '0.85rem' }}>No wallets provisioned yet</p>
                                                    </div>
                                                )}
                                            </div>

                                            {/* Balances & Sweep */}
                                            <div className={styles.drawerSection}>
                                                <h3><i className="fas fa-coins" style={{ color: '#f59e0b' }}></i> Asset Balances & Sweep</h3>
                                                {customerBalances && Array.isArray(customerBalances) && customerBalances.length > 0 ? (
                                                    <div style={{ display: 'flex', flexDirection: 'column', gap: '0.5rem', marginBottom: '1.5rem' }}>
                                                        {customerBalances.map((b: any, i: number) => (
                                                            <div key={i} style={{ display: 'flex', justifyContent: 'space-between', padding: '0.75rem 1rem', background: '#f8fafc', borderRadius: '10px', fontSize: '0.9rem' }}>
                                                                <span style={{ fontWeight: 600, color: '#334155' }}>{b.crypto_type}</span>
                                                                <span style={{ fontWeight: 700, color: '#059669' }}>{parseFloat(b.available_balance || '0').toFixed(6)}</span>
                                                            </div>
                                                        ))}
                                                    </div>
                                                ) : (
                                                    <p style={{ fontSize: '0.85rem', color: '#94a3b8', marginBottom: '1.5rem' }}>No balances found</p>
                                                )}

                                                <form className={styles.sweepForm} onSubmit={handleSweep}>
                                                    <div className={styles.formGroup} style={{ marginBottom: 0 }}>
                                                        <label>Asset</label>
                                                        <select 
                                                            className={styles.inputStyle} 
                                                            style={{ padding: '0.75rem 1rem', width: '100%' }}
                                                            value={sweepCryptoType}
                                                            onChange={e => setSweepCryptoType(e.target.value)}
                                                        >
                                                            {supportedCurrencies.length > 0 ? (
                                                                supportedCurrencies.map((c, idx) => (
                                                                    <option key={idx} value={c.crypto_type}>
                                                                        {c.crypto_type} ({c.network})
                                                                    </option>
                                                                ))
                                                            ) : (
                                                                <option disabled>Loading supported assets...</option>
                                                            )}
                                                        </select>
                                                    </div>
                                                    <div className={styles.formGroup} style={{ marginBottom: 0 }}>
                                                        <label>Amount</label>
                                                        <input className={styles.inputStyle} style={{ padding: '0.75rem 1rem' }} type="number" step="any" placeholder="0.00" value={sweepAmount} onChange={e => setSweepAmount(e.target.value)} />
                                                    </div>
                                                    <button className={styles.actionBtn} disabled={sweeping || !selectedCustomer.is_active}>
                                                        {sweeping ? <i className="fas fa-spinner fa-spin"></i> : 'Sweep'}
                                                    </button>
                                                </form>
                                            </div>
                                        </>
                                    )}

                                    {/* ================ TRANSACTIONS TAB ================ */}
                                    {drawerTab === 'transactions' && (
                                        <div className={styles.drawerSection}>
                                            <h3><i className="fas fa-exchange-alt" style={{ color: '#7c3aed' }}></i> Activity History</h3>
                                            {customerTransactions.length > 0 ? (
                                                <div style={{ display: 'flex', flexDirection: 'column', gap: '0.75rem' }}>
                                                    {customerTransactions.map(tx => {
                                                        const badge = TX_BADGES[tx.type] || TX_BADGES['WITHDRAWAL']
                                                        return (
                                                            <div key={tx.id} style={{ padding: '1rem', background: '#f8fafc', borderRadius: '12px', border: '1px solid #e2e8f0' }}>
                                                                <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '0.5rem' }}>
                                                                    <span style={{ display: 'inline-flex', alignItems: 'center', gap: '0.3rem', padding: '0.2rem 0.6rem', borderRadius: '6px', fontSize: '0.75rem', fontWeight: 700, color: badge.color, background: badge.bg }}>
                                                                        <i className={`fas ${badge.icon}`}></i> {tx.type.replace('_', ' ')}
                                                                    </span>
                                                                    <span style={{ fontSize: '0.75rem', color: '#94a3b8' }}>
                                                                        {new Date(tx.created_at).toLocaleString()}
                                                                    </span>
                                                                </div>
                                                                <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                                                                    <span style={{ fontWeight: 700, fontSize: '1rem', color: '#0f172a' }}>
                                                                        {parseFloat(tx.amount).toFixed(6)} {tx.crypto_type}
                                                                    </span>
                                                                    <span style={{ padding: '0.15rem 0.5rem', borderRadius: '6px', fontSize: '0.7rem', fontWeight: 600, color: tx.status === 'COMPLETED' ? '#059669' : '#d97706', background: tx.status === 'COMPLETED' ? '#d1fae5' : '#fef3c7' }}>
                                                                        {tx.status}
                                                                    </span>
                                                                </div>
                                                                {tx.description && <p style={{ margin: '0.5rem 0 0', fontSize: '0.8rem', color: '#64748b' }}>{tx.description}</p>}
                                                                {tx.transaction_hash && (
                                                                    <p style={{ margin: '0.25rem 0 0', fontSize: '0.75rem', color: '#94a3b8', fontFamily: 'monospace', wordBreak: 'break-all' }}>
                                                                        TX: {tx.transaction_hash}
                                                                    </p>
                                                                )}
                                                            </div>
                                                        )
                                                    })}
                                                </div>
                                            ) : (
                                                <div style={{ textAlign: 'center', padding: '3rem 1rem', color: '#94a3b8' }}>
                                                    <i className="fas fa-inbox" style={{ fontSize: '2rem', marginBottom: '0.75rem', display: 'block' }}></i>
                                                    <p style={{ margin: 0 }}>No transactions yet</p>
                                                </div>
                                            )}
                                        </div>
                                    )}

                                    {/* ================ PERMISSIONS TAB ================ */}
                                    {drawerTab === 'permissions' && (
                                        <>
                                            {/* Status Management */}
                                            <div className={styles.drawerSection}>
                                                <h3><i className="fas fa-user-shield" style={{ color: '#2563eb' }}></i> Account Status</h3>
                                                <p style={{ fontSize: '0.85rem', color: '#64748b', marginBottom: '1.5rem' }}>
                                                    Current: <strong style={{ color: getStatusStyle(selectedCustomer.status || 'active').color }}>{(selectedCustomer.status || 'active').toUpperCase()}</strong>
                                                    {selectedCustomer.status_reason && <span> — {selectedCustomer.status_reason}</span>}
                                                </p>

                                                <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '0.75rem' }}>
                                                    {selectedCustomer.status !== 'active' && (
                                                        <button onClick={() => handleStatusUpdate('active')} disabled={statusUpdating}
                                                            style={{ padding: '0.75rem', border: '1px solid #d1fae5', borderRadius: '10px', background: '#f0fdf4', color: '#059669', fontWeight: 600, cursor: 'pointer', fontSize: '0.85rem' }}>
                                                            <i className="fas fa-check-circle" style={{ marginRight: '0.3rem' }}></i> Activate
                                                        </button>
                                                    )}
                                                    {selectedCustomer.status !== 'flagged' && (
                                                        <button onClick={() => setShowStatusModal('flagged')} disabled={statusUpdating}
                                                            style={{ padding: '0.75rem', border: '1px solid #fef3c7', borderRadius: '10px', background: '#fffbeb', color: '#d97706', fontWeight: 600, cursor: 'pointer', fontSize: '0.85rem' }}>
                                                            <i className="fas fa-flag" style={{ marginRight: '0.3rem' }}></i> Flag
                                                        </button>
                                                    )}
                                                    {selectedCustomer.status !== 'suspended' && (
                                                        <button onClick={() => setShowStatusModal('suspended')} disabled={statusUpdating}
                                                            style={{ padding: '0.75rem', border: '1px solid #fee2e2', borderRadius: '10px', background: '#fef2f2', color: '#dc2626', fontWeight: 600, cursor: 'pointer', fontSize: '0.85rem' }}>
                                                            <i className="fas fa-pause-circle" style={{ marginRight: '0.3rem' }}></i> Suspend
                                                        </button>
                                                    )}
                                                    {selectedCustomer.status !== 'blocked' && (
                                                        <button onClick={() => setShowStatusModal('blocked')} disabled={statusUpdating}
                                                            style={{ padding: '0.75rem', border: '1px solid #f3f4f6', borderRadius: '10px', background: '#f9fafb', color: '#6b7280', fontWeight: 600, cursor: 'pointer', fontSize: '0.85rem' }}>
                                                            <i className="fas fa-ban" style={{ marginRight: '0.3rem' }}></i> Block
                                                        </button>
                                                    )}
                                                </div>
                                            </div>

                                            {/* Withdrawal Permissions */}
                                            <div className={styles.drawerSection}>
                                                <h3><i className="fas fa-shield-alt" style={{ color: '#7c3aed' }}></i> Withdrawal Permissions</h3>
                                                <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', padding: '1rem', background: '#f8fafc', borderRadius: '12px', marginBottom: '1rem' }}>
                                                    <div>
                                                        <p style={{ margin: 0, fontWeight: 600, color: '#334155' }}>Allow Withdrawals</p>
                                                        <p style={{ margin: '0.25rem 0 0', fontSize: '0.8rem', color: '#94a3b8' }}>Customer can withdraw funds to external wallets</p>
                                                    </div>
                                                    <button 
                                                        onClick={handleToggleWithdraw} 
                                                        disabled={permUpdating}
                                                        style={{
                                                            width: '52px', height: '28px', borderRadius: '14px', border: 'none', cursor: 'pointer',
                                                            background: selectedCustomer.can_withdraw ? '#059669' : '#d1d5db',
                                                            position: 'relative', transition: 'background 0.2s',
                                                        }}
                                                    >
                                                        <span style={{
                                                            width: '22px', height: '22px', borderRadius: '50%', background: 'white',
                                                            position: 'absolute', top: '3px', transition: 'left 0.2s', boxShadow: '0 1px 3px rgba(0,0,0,0.2)',
                                                            left: selectedCustomer.can_withdraw ? '27px' : '3px',
                                                        }}></span>
                                                    </button>
                                                </div>
                                                {selectedCustomer.withdrawal_limit && (
                                                    <p style={{ fontSize: '0.85rem', color: '#64748b' }}>
                                                        Per-transaction limit: <strong>{selectedCustomer.withdrawal_limit}</strong>
                                                    </p>
                                                )}
                                            </div>
                                        </>
                                    )}
                                </>
                            )}
                        </div>
                    </div>
                </div>
            )}

            {/* Status Change Reason Modal */}
            {showStatusModal && (
                <div style={{ position: 'fixed', inset: 0, zIndex: 1100, display: 'flex', alignItems: 'center', justifyContent: 'center', background: 'rgba(0,0,0,0.5)' }}
                     onClick={() => { setShowStatusModal(null); setStatusReason('') }}>
                    <div style={{ background: 'white', borderRadius: '16px', padding: '2rem', maxWidth: '420px', width: '90%', boxShadow: '0 20px 60px rgba(0,0,0,0.15)' }}
                         onClick={e => e.stopPropagation()}>
                        <h3 style={{ margin: '0 0 0.5rem', fontSize: '1.1rem' }}>
                            {showStatusModal === 'flagged' && '🚩 Flag Customer'}
                            {showStatusModal === 'suspended' && '⏸️ Suspend Customer'}
                            {showStatusModal === 'blocked' && '🚫 Block Customer'}
                        </h3>
                        <p style={{ margin: '0 0 1.25rem', fontSize: '0.85rem', color: '#64748b' }}>
                            {showStatusModal === 'flagged' && 'Customer will be limited to view-only access. They cannot withdraw or pay.'}
                            {showStatusModal === 'suspended' && 'Customer will lose all access. All operations will be rejected.'}
                            {showStatusModal === 'blocked' && 'Customer will be permanently blocked. All operations will be rejected.'}
                        </p>
                        <div style={{ marginBottom: '1.25rem' }}>
                            <label style={{ display: 'block', marginBottom: '0.5rem', fontSize: '0.85rem', fontWeight: 600, color: '#334155' }}>Reason (optional)</label>
                            <textarea 
                                value={statusReason}
                                onChange={e => setStatusReason(e.target.value)}
                                placeholder="Provide a reason for this action..."
                                rows={3}
                                style={{ width: '100%', padding: '0.75rem', borderRadius: '10px', border: '1px solid #e2e8f0', fontSize: '0.9rem', resize: 'none', fontFamily: 'inherit', boxSizing: 'border-box' }}
                            ></textarea>
                        </div>
                        <div style={{ display: 'flex', gap: '0.75rem' }}>
                            <button onClick={() => { setShowStatusModal(null); setStatusReason('') }}
                                style={{ flex: 1, padding: '0.75rem', border: '1px solid #e2e8f0', borderRadius: '10px', background: 'white', cursor: 'pointer', fontWeight: 600, color: '#64748b' }}>
                                Cancel
                            </button>
                            <button onClick={() => handleStatusUpdate(showStatusModal)} disabled={statusUpdating}
                                style={{ flex: 1, padding: '0.75rem', border: 'none', borderRadius: '10px', cursor: 'pointer', fontWeight: 600, color: 'white',
                                    background: showStatusModal === 'flagged' ? '#d97706' : showStatusModal === 'suspended' ? '#dc2626' : '#6b7280' }}>
                                {statusUpdating ? 'Updating...' : `Confirm ${showStatusModal}`}
                            </button>
                        </div>
                    </div>
                </div>
            )}
        </div>
    )
}

export default MerchantCustomersPage;
