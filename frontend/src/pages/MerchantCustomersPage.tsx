import React, { useState, useEffect, useMemo } from 'react'
import { customerAPI } from '@/services/apiService'
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
    created_at: string;
}

interface Wallet {
    crypto_type: string;
    network: string;
    address: string;
    created_at: string;
}



const MerchantCustomersPage: React.FC = () => {
    const { showToast } = useToast()
    const [customers, setCustomers] = useState<Customer[]>([])
    const [loading, setLoading] = useState(true)
    const [searchTerm, setSearchTerm] = useState('')
    const [statusFilter, setStatusFilter] = useState<'all' | 'active' | 'inactive'>('all')

    // Drawer States
    const [isCreateDrawerOpen, setIsCreateDrawerOpen] = useState(false)
    const [selectedCustomer, setSelectedCustomer] = useState<Customer | null>(null)
    
    // Form States
    const [newCustomer, setNewCustomer] = useState({ external_id: '', email: '', first_name: '', last_name: '' })
    const [submitting, setSubmitting] = useState(false)
    const [detailsLoading, setDetailsLoading] = useState(false)
    const [customerWallets, setCustomerWallets] = useState<Wallet[]>([])
    const [customerBalances, setCustomerBalances] = useState<any>(null)
    const [sweepCryptoType, setSweepCryptoType] = useState('USDT')
    const [sweepAmount, setSweepAmount] = useState('')
    const [sweeping, setSweeping] = useState(false)

    useEffect(() => {
        fetchCustomers()
    }, [])

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
        const active = customers.filter(c => c.is_active).length
        const recent = customers.filter(c => {
            const date = new Date(c.created_at)
            const now = new Date()
            const diff = now.getTime() - date.getTime()
            return diff < 7 * 24 * 60 * 60 * 1000 // Last 7 days
        }).length
        return { total, active, recent }
    }, [customers])

    const filteredCustomers = useMemo(() => {
        return customers.filter(c => {
            const matchesSearch = 
                c.external_id.toLowerCase().includes(searchTerm.toLowerCase()) ||
                (c.email?.toLowerCase().includes(searchTerm.toLowerCase())) ||
                (`${c.first_name || ''} ${c.last_name || ''}`.toLowerCase().includes(searchTerm.toLowerCase()))
            
            const matchesStatus = 
                statusFilter === 'all' || 
                (statusFilter === 'active' && c.is_active) || 
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
            showToast('Customer registered successfully', 'success')
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
        setCustomerWallets([])
        setCustomerBalances(null)

        try {
            const [walletRes, balRes] = await Promise.allSettled([
                customerAPI.getWallets(customer.external_id),
                customerAPI.getBalances(customer.external_id),
            ])
            if (walletRes.status === 'fulfilled') {
                setCustomerWallets(walletRes.value.data?.wallets || [])
            }
            if (balRes.status === 'fulfilled') {
                setCustomerBalances(balRes.value.data?.balances)
            }
        } catch {
            // silently handle — UI will show empty states
        } finally {
            setDetailsLoading(false)
        }
    }



    const handleSweep = async (e: React.FormEvent) => {
        e.preventDefault()
        if (!selectedCustomer || !sweepCryptoType || !sweepAmount) return;

        try {
            setSweeping(true)
            await customerAPI.sweep(selectedCustomer.external_id, {
                crypto_type: sweepCryptoType,
                amount: sweepAmount
            })
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

    const handleDeactivate = async () => {
        if (!selectedCustomer) return;
        if (!window.confirm('Are you sure? This customer will no longer be able to use their wallets.')) return;

        try {
            setDetailsLoading(true)
            await customerAPI.deactivate(selectedCustomer.external_id)
            showToast('Customer deactivated', 'success')
            setCustomers(prev => prev.map(c => c.id === selectedCustomer.id ? { ...c, is_active: false } : c))
            setSelectedCustomer({ ...selectedCustomer, is_active: false })
        } catch (error: any) {
            showToast(error.response?.data?.error || 'Failed to deactivate', 'error')
        } finally {
            setDetailsLoading(false)
        }
    }

    const getInitials = (c: Customer) => {
        if (c.first_name && c.last_name) return `${c.first_name[0]}${c.last_name[0]}`.toUpperCase()
        return c.external_id.substring(0, 2).toUpperCase()
    }

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
                        <i className="fas fa-sparkles"></i>
                    </div>
                    <div className={styles.statInfo}>
                        <h3>New This Week</h3>
                        <p className={styles.statValue}>
                            {stats.recent}
                            <span className={`${styles.statTrend} ${styles.up}`}>
                                <i className="fas fa-arrow-up"></i>
                            </span>
                        </p>
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
                        <option value="active">Active Only</option>
                        <option value="inactive">Inactive Only</option>
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
                                    <th>Joined Date</th>
                                    <th style={{ textAlign: 'right' }}>Actions</th>
                                </tr>
                            </thead>
                            <tbody>
                                {filteredCustomers.map(c => (
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
                                            <div className={`${styles.statusBadge} ${c.is_active ? styles.statusActive : styles.statusInactive}`}>
                                                <div className={styles.statusDot}></div>
                                                {c.is_active ? 'Online' : 'Deactivated'}
                                            </div>
                                        </td>
                                        <td>{new Date(c.created_at).toLocaleDateString(undefined, { month: 'short', day: 'numeric', year: 'numeric' })}</td>
                                        <td style={{ textAlign: 'right' }}>
                                            <button className={styles.actionBtn} style={{ padding: '0.5rem 1rem', background: '#f1f5f9', color: '#1e293b', display: 'inline-flex' }}>
                                                Manage <i className="fas fa-chevron-right ml-2"></i>
                                            </button>
                                        </td>
                                    </tr>
                                ))}
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
                                        Must be unique. Used to link this entity to your internal systems.
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
                            <h2><i className="fas fa-id-badge" style={{ color: '#2563eb' }}></i> Profile: {selectedCustomer.external_id}</h2>
                            <button className={styles.closeBtn} onClick={() => setSelectedCustomer(null)}>
                                <i className="fas fa-times"></i>
                            </button>
                        </div>
                        <div className={styles.drawerBody}>
                            {detailsLoading ? (
                                <div className={styles.loadingOverlay}><i className="fas fa-circle-notch fa-spin fa-2x"></i></div>
                            ) : (
                                <>
                                    <div className={styles.drawerSection}>
                                        <h3><i className="fas fa-wallet" style={{ color: '#2563eb' }}></i> Dedicated Wallets</h3>
                                        <p style={{ fontSize: '0.85rem', color: '#64748b', marginBottom: '1.5rem' }}>
                                            Auto-provisioned deposit addresses for this customer.
                                        </p>

                                        {customerWallets.length > 0 ? (
                                            <div style={{ display: 'flex', flexDirection: 'column', gap: '0.75rem' }}>
                                                {customerWallets.map((w, idx) => (
                                                    <div key={idx} className={styles.walletItem}>
                                                        <div className={styles.walletHeader}>
                                                            <span className={styles.walletType}>{w.crypto_type}</span>
                                                            <span className={styles.walletNetwork}>{w.network}</span>
                                                        </div>
                                                        <div className={styles.addressBox}>
                                                            <span className={styles.addressText}>{w.address}</span>
                                                            <button 
                                                                className={styles.copyBtn}
                                                                onClick={() => {
                                                                    navigator.clipboard.writeText(w.address)
                                                                    showToast('Address copied!', 'success')
                                                                }}
                                                            >
                                                                <i className="far fa-copy"></i>
                                                            </button>
                                                        </div>
                                                    </div>
                                                ))}
                                            </div>
                                        ) : (
                                            <div style={{ textAlign: 'center', padding: '2rem 1rem', background: '#f8fafc', borderRadius: '12px', color: '#94a3b8' }}>
                                                <i className="fas fa-wallet" style={{ fontSize: '1.5rem', marginBottom: '0.5rem', display: 'block' }}></i>
                                                <p style={{ margin: 0, fontSize: '0.85rem' }}>No wallets provisioned yet</p>
                                            </div>
                                        )}
                                    </div>

                                    <div className={styles.drawerSection}>
                                        <h3><i className="fas fa-coins" style={{ color: '#f59e0b' }}></i> Asset Balances & Sweep</h3>
                                        <p style={{ fontSize: '0.85rem', color: '#64748b', marginBottom: '1.5rem' }}>
                                            Consolidated balances across all provisioned wallets.
                                        </p>

                                        {customerBalances && (
                                            <div style={{ background: '#f1f5f9', padding: '1rem', borderRadius: '12px', marginBottom: '1.5rem', fontFamily: 'monospace', fontSize: '0.8rem' }}>
                                                <pre style={{ margin: 0 }}>{JSON.stringify(customerBalances, null, 2)}</pre>
                                            </div>
                                        )}

                                        <form className={styles.sweepForm} onSubmit={handleSweep}>
                                            <div className={styles.formGroup} style={{ marginBottom: 0 }}>
                                                <label>Asset</label>
                                                <input 
                                                    className={styles.inputStyle} 
                                                    style={{ padding: '0.75rem 1rem' }}
                                                    placeholder="USDT"
                                                    value={sweepCryptoType}
                                                    onChange={e => setSweepCryptoType(e.target.value.toUpperCase())}
                                                />
                                            </div>
                                            <div className={styles.formGroup} style={{ marginBottom: 0 }}>
                                                <label>Amount</label>
                                                <input 
                                                    className={styles.inputStyle} 
                                                    style={{ padding: '0.75rem 1rem' }}
                                                    type="number"
                                                    step="any"
                                                    placeholder="0.00"
                                                    value={sweepAmount}
                                                    onChange={e => setSweepAmount(e.target.value)}
                                                />
                                            </div>
                                            <button className={styles.actionBtn} disabled={sweeping || !selectedCustomer.is_active}>
                                                {sweeping ? <i className="fas fa-spinner fa-spin"></i> : 'Sweep'}
                                            </button>
                                        </form>
                                    </div>

                                    <div style={{ marginTop: '3rem', padding: '1.5rem', border: '1.5px dashed #fee2e2', borderRadius: '20px' }}>
                                        <h3 style={{ color: '#ef4444', fontSize: '1.1rem', fontWeight: 800, margin: '0 0 1rem 0' }}>Archive Area</h3>
                                        <p style={{ fontSize: '0.85rem', color: '#64748b', marginBottom: '1.5rem' }}>
                                            Deactivating this customer will disable all their provisioned wallets immediately.
                                        </p>
                                        <button 
                                            className={`${styles.actionBtn} ${styles.danger}`}
                                            style={{ width: '100%' }}
                                            onClick={handleDeactivate}
                                            disabled={!selectedCustomer.is_active || detailsLoading}
                                        >
                                            <i className="fas fa-user-slash"></i>
                                            {selectedCustomer.is_active ? 'Terminate Customer Account' : 'Account Terminated'}
                                        </button>
                                    </div>
                                </>
                            )}
                        </div>
                    </div>
                </div>
            )}
        </div>
    )
}

export default MerchantCustomersPage;

