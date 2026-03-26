import React, { useEffect, useState, useMemo } from 'react'
import { merchantAPI } from '@/services/apiService'
import { Balance, BalanceHistory } from '@/types'
import { useToast } from '@/contexts/ToastContext'
import { useAuthStore } from '@/stores/authStore'
import styles from '@/styles/pages/BalancePage.module.css'

// Safe parseFloat that never returns NaN
const safeFloat = (val: any): number => {
    const n = parseFloat(val)
    return isNaN(n) ? 0 : n
}
import {
    AreaChart,
    Area,
    XAxis,
    YAxis,
    CartesianGrid,
    Tooltip,
    ResponsiveContainer,
    PieChart,
    Pie,
    Cell,
    Tooltip as PieTooltip
} from 'recharts'

const PRIORITY_COLORS = [
    '#2563eb', '#10b981', '#f59e0b', '#8b5cf6', '#ec4899',
    '#06b6d4', '#f97316', '#3b82f6', '#14b8a6', '#6366f1'
];

const NETWORK_LABELS: Record<string, { name: string, sandbox: string }> = {
    SOL: { name: 'Solana', sandbox: 'Solana Devnet' },
    ETH: { name: 'Ethereum', sandbox: 'Ethereum Sepolia' },
    BNB: { name: 'BSC', sandbox: 'BSC Testnet' },
    MATIC: { name: 'Polygon', sandbox: 'Polygon Mumbai' },
    ARB: { name: 'Arbitrum', sandbox: 'Arbitrum Sepolia' },
    USDT_SPL: { name: 'Solana SPL', sandbox: 'Solana Devnet SPL' },
    WSOL: { name: 'Wrapped SOL', sandbox: 'Devnet WSOL' },
    USDT_ETH: { name: 'USDT (ERC20)', sandbox: 'Sepolia ERC20' },
    USDT_BEP20: { name: 'USDT (BINANCE)', sandbox: 'BINANCE Testnet (USDT)' },
    BUSD_BEP20: { name: 'BUSD (BINANCE)', sandbox: 'BINANCE Testnet (BUSD)' },
    USDT_POLYGON: { name: 'USDT (Polygon)', sandbox: 'Mumbai' },
    USDT_ARBITRUM: { name: 'USDT (Arbitrum)', sandbox: 'Arbitrum Sepolia' },
    BTC: { name: 'Bitcoin', sandbox: 'BTC Testnet' },
};

const getNetworkLabel = (cryptoType: string, isSandbox: boolean): string => {
    const entry = NETWORK_LABELS[cryptoType]
    if (entry) return isSandbox ? entry.sandbox : entry.name
    return isSandbox ? 'Testnet' : 'Mainnet'
};

const BalancePage: React.FC = () => {
    const [balance, setBalance] = useState<Balance | null>(null)
    const [history, setHistory] = useState<BalanceHistory | null>(null)
    const [loading, setLoading] = useState(true)
    const [selectedAsset, setSelectedAsset] = useState<string | null>(null) // null means 'Total'
    const { showToast } = useToast()
    const { user } = useAuthStore()

    useEffect(() => {
        loadData()
    }, [user?.sandbox_mode])

    const loadData = async () => {
        try {
            setLoading(true)
            const balRes = await merchantAPI.getBalance()
            if (balRes.data) setBalance(balRes.data)

            // Balance history is non-critical — don't let it block the page
            try {
                const histRes = await merchantAPI.getBalanceHistory({ limit: 30 })
                if (histRes.data) setHistory(histRes.data)
            } catch (histErr) {
                console.warn('Balance history unavailable:', histErr)
            }
        } catch (error) {
            console.error('Failed to load balance data:', error)
            showToast('Failed to load balance data', 'error')
        } finally {
            setLoading(false)
        }
    }

    const pieData = useMemo(() => {
        if (!balance?.balances) return []
        return balance.balances
            .filter(b => safeFloat(b.balance_usd) > 0)
            .map(b => ({
                name: b.crypto_type.split('_')[0],
                value: safeFloat(b.balance_usd)
            }))
            .sort((a, b) => b.value - a.value)
    }, [balance])

    const chartData = useMemo(() => {
        if (!history?.points) return []
        return history.points.map(p => ({
            date: new Date(p.date).toLocaleDateString(undefined, { month: 'short', day: 'numeric' }),
            total: safeFloat(p.total_usd),
            [selectedAsset || 'total']: selectedAsset
                ? safeFloat(p.balances[selectedAsset] || '0')
                : safeFloat(p.total_usd)
        }))
    }, [history, selectedAsset])

    return (
        <div className={styles.page}>
            <div className={styles.header}>
                <div>
                    <h1><i className="fas fa-wallet"></i> Balance Overview</h1>
                    <p>Visual breakdown and historical growth of your assets</p>
                </div>
                <button
                    className={styles.refreshBtn}
                    onClick={loadData}
                    disabled={loading}
                >
                    <i className={`fas fa-sync-alt ${loading ? 'fa-spin' : ''}`}></i>
                    Refresh Data
                </button>
            </div>

            {loading && !balance ? (
                <div className={styles.loadingState}>
                    <i className="fas fa-spinner fa-spin"></i>
                    <p>Analyzing your wealth...</p>
                </div>
            ) : (
                <div className={styles.content}>
                    {/* Stats Summary Cards */}
                    <div className={styles.statsGrid}>
                        <div className={styles.statCard}>
                            <div className={styles.statIcon}><i className="fas fa-vault"></i></div>
                            <div className={styles.statInfo}>
                                <p className={styles.statLabel}>Total Assets (USD)</p>
                                <p className={styles.statValue}>${safeFloat(balance?.total_usd || '0').toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 })}</p>
                            </div>
                        </div>
                        <div className={styles.statCard}>
                            <div className={styles.statIcon} style={{ background: 'rgba(16, 185, 129, 0.1)', color: '#10b981' }}><i className="fas fa-unlock"></i></div>
                            <div className={styles.statInfo}>
                                <p className={styles.statLabel}>Available Now</p>
                                <p className={styles.statValue}>${safeFloat(balance?.available_usd || '0').toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 })}</p>
                            </div>
                        </div>
                        <div className={styles.statCard}>
                            <div className={styles.statIcon} style={{ background: 'rgba(245, 158, 11, 0.1)', color: '#f59e0b' }}><i className="fas fa-hourglass-half"></i></div>
                            <div className={styles.statInfo}>
                                <p className={styles.statLabel}>Settling / Reserved</p>
                                <p className={styles.statValue}>${safeFloat(balance?.reserved_usd || '0').toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 })}</p>
                            </div>
                        </div>
                    </div>

                    {/* Charts Section */}
                    <div className={styles.chartsGrid}>
                        {/* Line Chart Component */}
                        <div className={styles.premiumCard}>
                            <div className={styles.cardHeader}>
                                <h3>
                                    <i className="fas fa-chart-line"></i>
                                    {selectedAsset ? `${selectedAsset.split('_')[0]} Balance Trend` : 'Total Balance Growth'}
                                </h3>
                                {selectedAsset && (
                                    <button
                                        onClick={() => setSelectedAsset(null)}
                                        style={{ fontSize: '0.75rem', color: '#2563eb', background: 'none', border: 'none', cursor: 'pointer', fontWeight: 600 }}
                                    >
                                        Show Total
                                    </button>
                                )}
                            </div>
                            <div className={styles.chartContainer}>
                                {chartData.length > 0 ? (
                                    <ResponsiveContainer width="100%" height={300}>
                                        <AreaChart data={chartData}>
                                            <defs>
                                                <linearGradient id="colorValue" x1="0" y1="0" x2="0" y2="1">
                                                    <stop offset="5%" stopColor="#3b82f6" stopOpacity={0.1} />
                                                    <stop offset="95%" stopColor="#3b82f6" stopOpacity={0} />
                                                </linearGradient>
                                            </defs>
                                            <CartesianGrid strokeDasharray="3 3" vertical={false} stroke="#f1f5f9" />
                                            <XAxis dataKey="date" axisLine={false} tickLine={false} tick={{ fontSize: 12, fill: '#64748b' }} dy={10} />
                                            <YAxis axisLine={false} tickLine={false} tick={{ fontSize: 12, fill: '#64748b' }} tickFormatter={(val) => `$${val}`} />
                                            <Tooltip
                                                contentStyle={{ borderRadius: '12px', border: 'none', boxShadow: '0 10px 15px -3px rgba(0,0,0,0.1)' }}
                                                formatter={(value: any) => [`$${parseFloat(value).toLocaleString()}`, selectedAsset ? 'Amount' : 'Total USD']}
                                            />
                                            <Area
                                                type="monotone"
                                                dataKey={selectedAsset || 'total'}
                                                stroke="#2563eb"
                                                strokeWidth={3}
                                                fillOpacity={1}
                                                fill="url(#colorValue)"
                                                animationDuration={1500}
                                            />
                                        </AreaChart>
                                    </ResponsiveContainer>
                                ) : (
                                    <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', height: '300px', color: '#94a3b8', flexDirection: 'column', gap: '8px' }}>
                                        <i className="fas fa-chart-area" style={{ fontSize: '2rem' }}></i>
                                        <p style={{ margin: 0, fontSize: '0.85rem' }}>No balance history data yet</p>
                                    </div>
                                )}
                            </div>
                        </div>

                        {/* Pie Chart Component */}
                        <div className={styles.premiumCard}>
                            <div className={styles.cardHeader}>
                                <h3><i className="fas fa-chart-pie"></i> Distribution</h3>
                            </div>
                            <div className={styles.miniPieContainer}>
                                {pieData.length > 0 ? (
                                    <ResponsiveContainer width="100%" height={200}>
                                        <PieChart>
                                            <Pie
                                                data={pieData}
                                                cx="50%"
                                                cy="50%"
                                                innerRadius={60}
                                                outerRadius={80}
                                                paddingAngle={5}
                                                dataKey="value"
                                                animationBegin={200}
                                                animationDuration={1000}
                                            >
                                                {pieData.map((_entry, index) => (
                                                    <Cell key={`cell-${index}`} fill={PRIORITY_COLORS[index % PRIORITY_COLORS.length]} />
                                                ))}
                                            </Pie>
                                            <PieTooltip
                                                contentStyle={{ borderRadius: '8px', border: 'none', fontSize: '12px' }}
                                                formatter={(val: any) => [`$${parseFloat(val).toLocaleString()}`, 'Value']}
                                            />
                                        </PieChart>
                                    </ResponsiveContainer>
                                ) : (
                                    <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', height: '200px', color: '#94a3b8', fontSize: '0.85rem' }}>
                                        No assets to display
                                    </div>
                                )}
                                <div style={{ fontSize: '0.8rem', color: '#64748b', textAlign: 'center', marginTop: '1rem' }}>
                                    {pieData.length} active assets
                                </div>
                            </div>
                        </div>
                    </div>

                    {/* Assets List Section */}
                    <div className={styles.balanceListSection}>
                        <div className={styles.sectionHeader}>
                            <h2>Assets Breakdown</h2>
                        </div>

                        <div className={styles.assetsList}>
                            {balance?.balances && balance.balances.length > 0 ? (
                                balance.balances.map((asset) => (
                                    <div
                                        key={asset.crypto_type}
                                        className={`${styles.assetRow} ${selectedAsset === asset.crypto_type ? styles.active : ''}`}
                                        onClick={() => setSelectedAsset(selectedAsset === asset.crypto_type ? null : asset.crypto_type)}
                                    >
                                        <div className={styles.assetMain}>
                                            <div className={styles.assetIconBox}>
                                                {(asset.crypto_type.includes('SOL') || asset.crypto_type.includes('BUSD')) ? (
                                                    <img 
                                                        src={asset.crypto_type.includes('SOL') ? '/solana-sol-logo.png' : '/binance-usd-busd-logo.png'} 
                                                        alt={asset.crypto_type}
                                                        className={styles.assetIconImage}
                                                        style={{ width: '100%', height: '100%', borderRadius: '50%' }}
                                                    />
                                                ) : (
                                                    <i className={getIconForCrypto(asset.crypto_type)}></i>
                                                )}
                                            </div>
                                            <div className={styles.assetMeta}>
                                                <h3>{asset.crypto_type.split('_')[0]}</h3>
                                                <span>{getNetworkLabel(asset.crypto_type, !!user?.sandbox_mode)}</span>
                                            </div>
                                        </div>
                                        <div className={styles.assetValues}>
                                            <div className={styles.cryptoValue}>
                                                {safeFloat(asset.total_balance).toFixed(6)} {asset.crypto_type.split('_')[0]}
                                            </div>
                                            <div className={styles.usdValue}>
                                                ${safeFloat(asset.balance_usd).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 })}
                                            </div>
                                        </div>
                                    </div>
                                ))
                            ) : (
                                <div className={styles.emptyAssets}>
                                    <i className="fas fa-coins"></i>
                                    <p>No crypto assets found in your account yet.</p>
                                </div>
                            )}
                        </div>
                    </div>
                </div>
            )}
        </div>
    )
}

// Helper to get font-awesome icons for crypto
function getIconForCrypto(type: string): string {
    const t = type.toLowerCase()
    if (t.includes('eth')) return 'fab fa-ethereum'
    if (t.includes('btc')) return 'fab fa-bitcoin'
    if (t.includes('usdt') || t.includes('usdc')) return 'fas fa-dollar-sign'
    if (t.includes('sol')) return 'fas fa-bolt'
    if (t.includes('bnb')) return 'fab fa-bitcoin'
    if (t.includes('arb')) return 'fas fa-layer-group'
    return 'fas fa-coins'
}

export default BalancePage
