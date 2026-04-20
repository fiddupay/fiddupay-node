import React, { useEffect, useState, useMemo } from 'react'
import { merchantAPI } from '@/services/apiService'
import { BalanceHistory } from '@/types'
import { useToast } from '@/contexts/ToastContext'
import { useAuthStore } from '@/stores/authStore'
import styles from '@/styles/pages/BalancePage.module.css'
import { BalanceSkeleton } from '@/components/layout/PageSkeletons'
import SEO from '@/components/ui/SEO'

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

import { useBalanceStore } from '@/stores/balanceStore'

const BalancePage: React.FC = () => {
    const { balance, fetchBalance } = useBalanceStore()
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
            
            // Load balance through store
            await fetchBalance()

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
            .filter(b => safeFloat(b.total_usd) > 0)
            .map(b => ({
                name: b.crypto_type.split('_')[0],
                value: safeFloat(b.total_usd)
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
            <SEO 
                title="Portfolio Balance" 
                description="Monitor your cryptocurrency portfolio value, asset allocation, and historical growth."
            />
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
                <BalanceSkeleton />
            ) : (
                <div className={styles.content}>
                    {/* Stats Summary Cards */}
                    <div className={styles.statsGrid}>
                        <div className={styles.glassStatCard}>
                            <div className={styles.statIcon}><i className="fas fa-vault"></i></div>
                            <div className={styles.statInfo}>
                                <p className={styles.statLabel}>Total Portfolio Value</p>
                                <p className={`${styles.statValue} ${safeFloat(balance?.total_usd) < 0 ? styles.negativeValue : ''}`}>
                                    ${safeFloat(balance?.total_usd || '0').toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 })}
                                </p>
                            </div>
                        </div>
                        <div className={styles.glassStatCard}>
                            <div className={styles.statIcon} style={{ background: 'linear-gradient(135deg, #10b981, #059669)', color: 'white' }}><i className="fas fa-unlock"></i></div>
                            <div className={styles.statInfo}>
                                <p className={styles.statLabel}>Liquid Assets</p>
                                <p className={`${styles.statValue} ${safeFloat(balance?.available_usd) < 0 ? styles.negativeValue : ''}`}>
                                    ${safeFloat(balance?.available_usd || '0').toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 })}
                                </p>
                            </div>
                        </div>
                        <div className={styles.glassStatCard}>
                            <div className={styles.statIcon} style={{ background: 'linear-gradient(135deg, #f59e0b, #d97706)', color: 'white' }}><i className="fas fa-hourglass-half"></i></div>
                            <div className={styles.statInfo}>
                                <p className={styles.statLabel}>Pending / Reserved</p>
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
                                    {selectedAsset ? `${selectedAsset.split('_')[0]} Performance` : 'Wealth Growth'}
                                </h3>
                                {selectedAsset && (
                                    <button
                                        onClick={() => setSelectedAsset(null)}
                                        className={styles.resetAssetBtn}
                                    >
                                        <i className="fas fa-times"></i> Clear Selection
                                    </button>
                                )}
                            </div>
                            <div className={styles.chartContainer}>
                                {chartData.length > 0 ? (
                                    <ResponsiveContainer width="100%" height={300}>
                                        <AreaChart data={chartData}>
                                            <defs>
                                                <linearGradient id="colorValue" x1="0" y1="0" x2="0" y2="1">
                                                    <stop offset="5%" stopColor="#3b82f6" stopOpacity={0.2} />
                                                    <stop offset="95%" stopColor="#3b82f6" stopOpacity={0} />
                                                </linearGradient>
                                            </defs>
                                            <CartesianGrid strokeDasharray="3 3" vertical={false} stroke="rgba(203, 213, 225, 0.3)" />
                                            <XAxis dataKey="date" axisLine={false} tickLine={false} tick={{ fontSize: 12, fill: '#64748b' }} dy={10} />
                                            <YAxis axisLine={false} tickLine={false} tick={{ fontSize: 12, fill: '#64748b' }} tickFormatter={(val) => `$${val}`} />
                                            <Tooltip
                                                contentStyle={{ 
                                                    borderRadius: '16px', 
                                                    border: '1px solid rgba(255,255,255,0.2)', 
                                                    boxShadow: '0 25px 50px -12px rgba(0,0,0,0.25)',
                                                    backdropFilter: 'blur(10px)',
                                                    background: 'rgba(255, 255, 255, 0.9)'
                                                }}
                                                formatter={(value: any) => [`$${parseFloat(value).toLocaleString()}`, selectedAsset ? 'Current Value' : 'Portfolio Value']}
                                            />
                                            <Area
                                                type="monotone"
                                                dataKey={selectedAsset || 'total'}
                                                stroke="#2563eb"
                                                strokeWidth={4}
                                                fillOpacity={1}
                                                fill="url(#colorValue)"
                                                animationDuration={2000}
                                            />
                                        </AreaChart>
                                    </ResponsiveContainer>
                                ) : (
                                    <div className={styles.noHistory}>
                                        <div className={styles.emptyIcon}><i className="fas fa-chart-area"></i></div>
                                        <p>Chart data will appear once you have multi-day history</p>
                                    </div>
                                )}
                            </div>
                        </div>

                        {/* Pie Chart Component */}
                        <div className={styles.premiumCard}>
                            <div className={styles.cardHeader}>
                                <h3><i className="fas fa-pie-chart"></i> Asset Allocation</h3>
                            </div>
                            <div className={styles.pieContent}>
                                {pieData.length > 0 ? (
                                    <div className={styles.pieWrapper}>
                                        <ResponsiveContainer width="100%" height={240}>
                                            <PieChart>
                                                <Pie
                                                    data={pieData}
                                                    cx="50%"
                                                    cy="50%"
                                                    innerRadius={70}
                                                    outerRadius={95}
                                                    paddingAngle={8}
                                                    dataKey="value"
                                                    stroke="none"
                                                    animationBegin={0}
                                                    animationDuration={1500}
                                                >
                                                    {pieData.map((_entry, index) => (
                                                        <Cell 
                                                            key={`cell-${index}`} 
                                                            fill={PRIORITY_COLORS[index % PRIORITY_COLORS.length]} 
                                                            style={{ filter: 'drop-shadow(0 4px 6px rgba(0,0,0,0.1))' }}
                                                        />
                                                    ))}
                                                </Pie>
                                                <PieTooltip
                                                    contentStyle={{ 
                                                        borderRadius: '12px', 
                                                        border: 'none', 
                                                        background: 'rgba(0,0,0,0.8)', 
                                                        color: 'white',
                                                        fontSize: '12px' 
                                                    }}
                                                    itemStyle={{ color: 'white' }}
                                                    formatter={(val: any) => [`$${parseFloat(val).toLocaleString()}`, 'Portfolio Stake']}
                                                />
                                            </PieChart>
                                        </ResponsiveContainer>
                                        <div className={styles.pieCenterContent}>
                                            <span className={styles.pieCenterLabel}>Assets</span>
                                            <span className={styles.pieCenterValue}>{pieData.length}</span>
                                        </div>
                                    </div>
                                ) : (
                                    <div className={styles.noDataPlaceholder}>
                                        <i className="fas fa-coins"></i>
                                        <p>Awaiting your first deposit</p>
                                    </div>
                                )}
                            </div>
                        </div>
                    </div>

                    {/* Assets Grid Section */}
                    <div className={styles.balanceListSection}>
                        <div className={styles.sectionHeader}>
                            <h2>Your Crypto Assets</h2>
                            <span className={styles.assetCountBadge}>{balance?.balances?.length || 0} Total</span>
                        </div>

                        <div className={styles.assetsGrid}>
                            {balance?.balances && balance.balances.length > 0 ? (
                                balance.balances.map((asset, index) => (
                                    <div
                                        key={asset.crypto_type}
                                        className={`${styles.assetGlassCard} ${selectedAsset === asset.crypto_type ? styles.active : ''}`}
                                        style={{ '--index': index } as React.CSSProperties}
                                        onClick={() => setSelectedAsset(selectedAsset === asset.crypto_type ? null : asset.crypto_type)}
                                    >
                                        <div className={styles.assetHeader}>
                                            <div className={styles.assetIconWrapper}>
                                                {(asset.crypto_type.includes('SOL') || asset.crypto_type.includes('BUSD')) ? (
                                                    <img 
                                                        src={asset.crypto_type.includes('SOL') ? '/solana-sol-logo.png' : '/binance-usd-busd-logo.png'} 
                                                        alt={asset.crypto_type}
                                                        className={styles.assetIconImage}
                                                    />
                                                ) : (
                                                    <div className={styles.fallbackIcon}>
                                                        <i className={getIconForCrypto(asset.crypto_type)}></i>
                                                    </div>
                                                )}
                                            </div>
                                            <div className={styles.assetBadge}>
                                                {getNetworkLabel(asset.crypto_type, !!user?.sandbox_mode)}
                                            </div>
                                        </div>
                                        
                                        <div className={styles.assetMainInfo}>
                                            <h3>{asset.crypto_type.split('_')[0]}</h3>
                                            <div className={styles.assetHoldings}>
                                                <div className={styles.cryptoAmount}>
                                                    {safeFloat(asset.total_balance).toFixed(6)} <span>{asset.crypto_type.split('_')[0]}</span>
                                                </div>
                                                <div className={`${styles.usdEquivalent} ${safeFloat(asset.total_usd) < 0 ? styles.negativeValue : ''}`}>
                                                    ${safeFloat(asset.total_usd).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 })}
                                                </div>
                                            </div>
                                        </div>

                                            <div className={styles.availableLabel}>
                                                <span className={styles.pulseDot} style={{ background: safeFloat(asset.available_usd) < 0 ? '#f87171' : '#10b981' }}></span> 
                                                Available: <span className={safeFloat(asset.available_usd) < 0 ? styles.negativeValue : ''}>
                                                    ${safeFloat(asset.available_usd).toLocaleString()}
                                                </span>
                                            </div>
                                    </div>
                                ))
                            ) : (
                                <div className={styles.modernEmptyState}>
                                    <div className={styles.emptyIllustration}>
                                        <i className="fas fa-wallet"></i>
                                        <div className={styles.floatingCoin}><i className="fas fa-coins"></i></div>
                                    </div>
                                    <h3>Empty Wallet</h3>
                                    <p>Your received payments will appear here as assets once confirmed on the blockchain.</p>
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
    if (t.includes('usdt')) return 'fas fa-shield-halved'
    if (t.includes('usdc')) return 'fas fa-dollar-sign'
    if (t.includes('sol')) return 'fas fa-bolt-lightning'
    if (t.includes('bnb')) return 'fas fa-diamond'
    if (t.includes('arb')) return 'fas fa-layer-group'
    if (t.includes('matic')) return 'fas fa-hexagon-nodes'
    return 'fas fa-coins'
}

export default BalancePage
