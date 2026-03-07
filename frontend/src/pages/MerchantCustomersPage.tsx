import React, { useState, useEffect } from 'react'
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

const NETWORKS = ['Native', 'ERC20', 'BEP20', 'TRC20', 'SOL', 'POLYGON', 'ARB'];

const MerchantCustomersPage: React.FC = () => {
    const { showToast } = useToast()
    const [customers, setCustomers] = useState<Customer[]>([])
    const [loading, setLoading] = useState(true)

    // Create Customer Modal State
    const [isCreateModalOpen, setIsCreateModalOpen] = useState(false)
    const [newCustomer, setNewCustomer] = useState({ external_id: '', email: '', first_name: '', last_name: '' })
    const [submitting, setSubmitting] = useState(false)

    // Customer Details Modal State
    const [selectedCustomer, setSelectedCustomer] = useState<Customer | null>(null)
    const [customerWallets, setCustomerWallets] = useState<Wallet[]>([])
    const [customerBalances, setCustomerBalances] = useState<any>(null)
    const [detailsLoading, setDetailsLoading] = useState(false)

    // Provisioning State
    const [selectedNetworks, setSelectedNetworks] = useState<string[]>(['Native', 'ERC20', 'BEP20'])
    const [provisioning, setProvisioning] = useState(false)

    // Sweep State
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

    const handleCreateCustomer = async (e: React.FormEvent) => {
        e.preventDefault()
        if (!newCustomer.external_id) {
            showToast('External ID is required', 'error')
            return;
        }

        try {
            setSubmitting(true)
            await customerAPI.create(newCustomer)
            showToast('Customer created successfully', 'success')
            setIsCreateModalOpen(false)
            setNewCustomer({ external_id: '', email: '', first_name: '', last_name: '' })
            fetchCustomers()
        } catch (error: any) {
            showToast(error.response?.data?.error || error.message || 'Failed to create customer', 'error')
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
            const balRes = await customerAPI.getBalances(customer.external_id)
            setCustomerBalances(balRes.data?.balances)
        } catch (error: any) {
            showToast(error.response?.data?.error || error.message || 'Failed to load customer details', 'error')
        } finally {
            setDetailsLoading(false)
        }
    }

    const handleProvisionWallets = async (auto: boolean = false) => {
        if (!selectedCustomer) return;

        const networksToProvision = auto ? [] : selectedNetworks;

        if (!auto && networksToProvision.length === 0) {
            showToast('Select at least one network', 'error')
            return;
        }

        try {
            setProvisioning(true)
            const res = await customerAPI.provisionWallets(selectedCustomer.external_id, networksToProvision)
            setCustomerWallets(res.data?.wallets || [])
            showToast(`Provisioned ${res.data?.wallets?.length || 0} wallets successfully`, 'success')

            // Refresh balances
            const balRes = await customerAPI.getBalances(selectedCustomer.external_id)
            setCustomerBalances(balRes.data?.balances)
        } catch (error: any) {
            showToast(error.response?.data?.error || error.message || 'Failed to provision wallets', 'error')
        } finally {
            setProvisioning(false)
        }
    }

    const handleDeactivate = async () => {
        if (!selectedCustomer) return;
        if (!window.confirm(`Are you sure you want to deactivate ${selectedCustomer.external_id}? This cannot be undone.`)) {
            return;
        }

        try {
            setDetailsLoading(true)
            await customerAPI.deactivate(selectedCustomer.external_id)
            showToast('Customer deactivated successfully', 'success')

            setCustomers(prev => prev.map(c =>
                c.external_id === selectedCustomer.external_id ? { ...c, is_active: false } : c
            ))
            setSelectedCustomer({ ...selectedCustomer, is_active: false })

        } catch (error: any) {
            showToast(error.response?.data?.error || error.message || 'Failed to deactivate customer', 'error')
        } finally {
            setDetailsLoading(false)
        }
    }

    const handleSweep = async (e: React.FormEvent) => {
        e.preventDefault()
        if (!selectedCustomer) return;
        if (!sweepCryptoType || !sweepAmount) return;

        try {
            setSweeping(true)
            await customerAPI.sweep(selectedCustomer.external_id, {
                crypto_type: sweepCryptoType,
                amount: sweepAmount
            })
            showToast(`Successfully swept ${sweepAmount} ${sweepCryptoType} to Master Balance`, 'success')
            setSweepAmount('')

            // Refresh balances
            const balRes = await customerAPI.getBalances(selectedCustomer.external_id)
            setCustomerBalances(balRes.data?.balances)
        } catch (error: any) {
            showToast(error.response?.data?.error || error.message || 'Failed to sweep funds', 'error')
        } finally {
            setSweeping(false)
        }
    }

    const toggleNetwork = (net: string) => {
        setSelectedNetworks(prev =>
            prev.includes(net) ? prev.filter(n => n !== net) : [...prev, net]
        )
    }

    return (
        <div className={styles.page}>
            <div className={styles.header}>
                <div className={styles.headerInfo}>
                    <h1>Merchant Customers</h1>
                    <p>Manage sub-accounts and their dedicated crypto deposit wallets</p>
                </div>
                <button className={styles.addBtn} onClick={() => setIsCreateModalOpen(true)}>
                    <i className="fas fa-plus"></i>
                    Add Customer
                </button>
            </div>

            <div className={styles.card}>
                <div className={styles.cardHeader}>
                    <div className={styles.cardTitle}>
                        <i className="fas fa-users"></i>
                        Customers List
                    </div>
                </div>

                {loading ? (
                    <div className={styles.loading}>
                        <i className="fas fa-spinner fa-spin"></i>
                    </div>
                ) : customers.length === 0 ? (
                    <div className={styles.emptyState}>
                        <i className="fas fa-user-slash"></i>
                        <p>No customers found.</p>
                        <button className={styles.addBtn} onClick={() => setIsCreateModalOpen(true)}>
                            Register First Customer
                        </button>
                    </div>
                ) : (
                    <div className={styles.tableContainer}>
                        <table className={styles.table}>
                            <thead>
                                <tr>
                                    <th>External ID</th>
                                    <th>Name</th>
                                    <th>Email</th>
                                    <th>Status</th>
                                    <th>Created At</th>
                                    <th>Action</th>
                                </tr>
                            </thead>
                            <tbody>
                                {customers.map((c) => (
                                    <tr
                                        key={c.id}
                                        className={`${styles.clickableRow} ${!c.is_active ? styles.inactiveRow : ''}`}
                                        onClick={() => openCustomerDetails(c)}
                                    >
                                        <td className={styles.idCell}>{c.external_id}</td>
                                        <td>{c.first_name || c.last_name ? `${c.first_name || ''} ${c.last_name || ''}`.trim() : '-'}</td>
                                        <td>{c.email || '-'}</td>
                                        <td>
                                            <span className={`${styles.statusBadge} ${c.is_active ? styles.statusActive : styles.statusInactive}`}>
                                                {c.is_active ? 'Active' : 'Inactive'}
                                            </span>
                                        </td>
                                        <td>{new Date(c.created_at).toLocaleDateString()}</td>
                                        <td>
                                            <button className={styles.actionBtn}>
                                                Manage <i className="fas fa-chevron-right ml-1"></i>
                                            </button>
                                        </td>
                                    </tr>
                                ))}
                            </tbody>
                        </table>
                    </div>
                )}
            </div>

            {/* Create Customer Modal */}
            {isCreateModalOpen && (
                <div className={styles.modalOverlay}>
                    <div className={styles.modal}>
                        <div className={styles.modalHeader}>
                            <h2>Register New Customer</h2>
                            <button className={styles.closeBtn} onClick={() => setIsCreateModalOpen(false)}>
                                <i className="fas fa-times"></i>
                            </button>
                        </div>
                        <div className={styles.modalBody}>
                            <form onSubmit={handleCreateCustomer}>
                                <div className={styles.formGroup}>
                                    <label>External ID (Required)*</label>
                                    <input
                                        type="text"
                                        required
                                        placeholder="e.g. user_12345"
                                        value={newCustomer.external_id}
                                        onChange={(e) => setNewCustomer({ ...newCustomer, external_id: e.target.value })}
                                        className={styles.input}
                                    />
                                    <small style={{ color: '#6b7280', fontSize: '0.75rem', marginTop: '0.25rem', display: 'block' }}>
                                        A unique identifier for this user in your own system.
                                    </small>
                                </div>
                                <div className={styles.formGroup}>
                                    <label>Email (Optional)</label>
                                    <input
                                        type="email"
                                        placeholder="customer@example.com"
                                        value={newCustomer.email}
                                        onChange={(e) => setNewCustomer({ ...newCustomer, email: e.target.value })}
                                        className={styles.input}
                                    />
                                </div>
                                <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '1rem' }}>
                                    <div className={styles.formGroup}>
                                        <label>First Name (Optional)</label>
                                        <input
                                            type="text"
                                            placeholder="John"
                                            value={newCustomer.first_name}
                                            onChange={(e) => setNewCustomer({ ...newCustomer, first_name: e.target.value })}
                                            className={styles.input}
                                        />
                                    </div>
                                    <div className={styles.formGroup}>
                                        <label>Last Name (Optional)</label>
                                        <input
                                            type="text"
                                            placeholder="Doe"
                                            value={newCustomer.last_name}
                                            onChange={(e) => setNewCustomer({ ...newCustomer, last_name: e.target.value })}
                                            className={styles.input}
                                        />
                                    </div>
                                </div>
                                <button type="submit" className={styles.submitBtn} disabled={submitting}>
                                    {submitting ? <><i className="fas fa-spinner fa-spin"></i> Registering...</> : 'Register Customer'}
                                </button>
                            </form>
                        </div>
                    </div>
                </div>
            )}

            {/* Customer Details & Wallets Modal */}
            {selectedCustomer && (
                <div className={styles.modalOverlay}>
                    <div className={`${styles.modal} ${styles.modalLarge}`}>
                        <div className={styles.modalHeader}>
                            <div style={{ display: 'flex', alignItems: 'center', gap: '1rem' }}>
                                <h2>Customer: <span className={styles.idCell}>{selectedCustomer.external_id}</span></h2>
                                <span className={`${styles.statusBadge} ${selectedCustomer.is_active ? styles.statusActive : styles.statusInactive}`}>
                                    {selectedCustomer.is_active ? 'Active' : 'Inactive'}
                                </span>
                            </div>
                            <button className={styles.closeBtn} onClick={() => setSelectedCustomer(null)}>
                                <i className="fas fa-times"></i>
                            </button>
                        </div>

                        <div className={styles.modalBody}>
                            {detailsLoading ? (
                                <div className={styles.loading}><i className="fas fa-spinner fa-spin"></i></div>
                            ) : (
                                <div className={styles.detailsGrid}>
                                    <div>
                                        <h3 className={styles.sectionTitle}>
                                            <span>Provision Deposit Wallets</span>
                                        </h3>
                                        <p style={{ fontSize: '0.85rem', color: '#6b7280', marginBottom: '1rem' }}>
                                            Select the networks on which this customer can deposit funds. Dedicated addresses will be generated.
                                        </p>

                                        <div className={styles.checkboxList} style={{ marginBottom: '1.25rem' }}>
                                            {NETWORKS.map(net => (
                                                <label key={net} className={styles.checkboxItem}>
                                                    <input
                                                        type="checkbox"
                                                        checked={selectedNetworks.includes(net)}
                                                        onChange={() => toggleNetwork(net)}
                                                    />
                                                    {net}
                                                </label>
                                            ))}
                                        </div>

                                        <div style={{ display: 'flex', gap: '1rem', marginBottom: '1.5rem' }}>
                                            <button
                                                className={styles.submitBtn}
                                                onClick={() => handleProvisionWallets(true)}
                                                disabled={provisioning || !selectedCustomer.is_active}
                                                style={{ background: '#059669' }}
                                            >
                                                {provisioning ? <i className="fas fa-spinner fa-spin"></i> : <i className="fas fa-magic"></i>}
                                                Provision All Supported (Auto)
                                            </button>

                                            <button
                                                className={styles.submitBtn}
                                                onClick={() => handleProvisionWallets(false)}
                                                disabled={provisioning || selectedNetworks.length === 0 || !selectedCustomer.is_active}
                                            >
                                                {provisioning ? <i className="fas fa-spinner fa-spin"></i> : <i className="fas fa-wallet"></i>}
                                                Provision Selected
                                            </button>
                                        </div>

                                        {customerWallets.length > 0 && (
                                            <div className={styles.walletList} style={{ marginTop: '1.5rem' }}>
                                                <h4 style={{ fontSize: '0.9rem', marginBottom: '0.5rem' }}>Newly Provisioned Addresses:</h4>
                                                {customerWallets.map((w, i) => (
                                                    <div key={i} className={styles.walletItem}>
                                                        <div className={styles.walletItemHeader}>
                                                            <span className={styles.walletType}>{w.crypto_type}</span>
                                                            <span className={styles.walletNetwork}>{w.network}</span>
                                                        </div>
                                                        <div className={styles.walletAddress}>
                                                            {w.address}
                                                            <button
                                                                className={styles.actionBtn}
                                                                style={{ padding: '0.1rem 0.3rem', marginLeft: '0.5rem', border: 'none' }}
                                                                onClick={() => navigator.clipboard.writeText(w.address)}
                                                            >
                                                                <i className="far fa-copy"></i>
                                                            </button>
                                                        </div>
                                                    </div>
                                                ))}
                                            </div>
                                        )}
                                    </div>

                                    {/* Balances & Sweeping Region */}
                                    <div style={{ marginTop: '1.5rem', borderTop: '1px solid #e5e7eb', paddingTop: '1.5rem' }}>
                                        <h3 className={styles.sectionTitle}>Sweep Funds to Master Balance</h3>
                                        <p style={{ fontSize: '0.85rem', color: '#6b7280', marginBottom: '1rem' }}>
                                            Move funds from this customer's dedicated wallets into your main merchant master balance.
                                        </p>

                                        {customerBalances && (
                                            <div style={{ marginBottom: '1rem', padding: '1rem', background: '#f3f4f6', borderRadius: '0.5rem' }}>
                                                <h4 style={{ fontSize: '0.85rem', margin: '0 0 0.5rem 0' }}>Current Sub-Account Balances:</h4>
                                                <pre style={{ fontSize: '0.8rem', margin: 0, fontFamily: 'monospace', whiteSpace: 'pre-wrap' }}>
                                                    {JSON.stringify(customerBalances, null, 2)}
                                                </pre>
                                            </div>
                                        )}

                                        <form onSubmit={handleSweep} style={{ display: 'flex', gap: '1rem', alignItems: 'flex-end' }}>
                                            <div className={styles.formGroup} style={{ flex: 1, marginBottom: 0 }}>
                                                <label>Asset (e.g. USDT)</label>
                                                <input
                                                    type="text"
                                                    required
                                                    value={sweepCryptoType}
                                                    onChange={(e) => setSweepCryptoType(e.target.value.toUpperCase())}
                                                    className={styles.input}
                                                />
                                            </div>
                                            <div className={styles.formGroup} style={{ flex: 1, marginBottom: 0 }}>
                                                <label>Amount</label>
                                                <input
                                                    type="number"
                                                    step="any"
                                                    required
                                                    value={sweepAmount}
                                                    onChange={(e) => setSweepAmount(e.target.value)}
                                                    className={styles.input}
                                                    placeholder="0.00"
                                                />
                                            </div>
                                            <button
                                                type="submit"
                                                className={styles.submitBtn}
                                                style={{ width: 'auto', padding: '0.75rem 1.5rem' }}
                                                disabled={sweeping || !selectedCustomer.is_active}
                                            >
                                                {sweeping ? <i className="fas fa-spinner fa-spin"></i> : 'Sweep'}
                                            </button>
                                        </form>

                                        <div style={{ marginTop: '2rem', borderTop: '1px solid #fee2e2', paddingTop: '1.5rem' }}>
                                            <h3 className={`${styles.sectionTitle} ${styles.dangerText}`}>Danger Zone</h3>
                                            <p style={{ fontSize: '0.85rem', color: '#6b7280', marginBottom: '1rem' }}>
                                                Deactivating a customer will prevent any further wallet provisioning or transactions.
                                                This action is permanent for this external ID.
                                            </p>
                                            <button
                                                className={styles.submitBtn}
                                                style={{ background: '#ef4444', color: 'white' }}
                                                onClick={handleDeactivate}
                                                disabled={!selectedCustomer.is_active || detailsLoading}
                                            >
                                                {detailsLoading ? <i className="fas fa-spinner fa-spin"></i> : <i className="fas fa-user-slash"></i>}
                                                {selectedCustomer.is_active ? 'Deactivate Customer' : 'Customer Deactivated'}
                                            </button>
                                        </div>
                                    </div>
                                </div>
                            )}
                        </div>
                    </div>
                </div>
            )}
        </div>
    );
};

export default MerchantCustomersPage;
