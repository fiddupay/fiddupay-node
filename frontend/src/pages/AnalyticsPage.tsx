import React, { useEffect, useState } from 'react'
import {
    XAxis,
    YAxis,
    CartesianGrid,
    Tooltip,
    ResponsiveContainer,
    AreaChart,
    Area
} from 'recharts'
import { merchantAPI } from '@/services/apiService'
import { useToast } from '@/contexts/ToastContext'
import styles from '@/styles/pages/AnalyticsPage.module.css'

interface TimeSeriesPoint {
    date: string;
    volume_usd: string;
    count: number;
}

interface AnalyticsData {
    total_volume_usd: string;
    successful_payments: number;
    failed_payments: number;
    total_fees_paid: string;
    average_transaction_value: string;
    by_blockchain: Record<string, {
        volume_usd: string;
        payment_count: number;
        average_value: string;
    }>;
    payment_trends: TimeSeriesPoint[];
}

const AnalyticsPage: React.FC = () => {
    const [analytics, setAnalytics] = useState<AnalyticsData | null>(null)
    const [loading, setLoading] = useState(true)
    const [dateRange, setDateRange] = useState({
        from_date: new Date(Date.now() - 30 * 24 * 60 * 60 * 1000).toISOString().split('T')[0],
        to_date: new Date().toISOString().split('T')[0]
    })
    const { showToast } = useToast()

    useEffect(() => {
        loadAnalytics()
    }, [dateRange])

    const loadAnalytics = async () => {
        try {
            setLoading(true)
            const response = await merchantAPI.getAnalytics({
                from_date: new Date(dateRange.from_date).toISOString(),
                to_date: new Date(dateRange.to_date + 'T23:59:59Z').toISOString()
            })
            if (response.data) {
                setAnalytics(response.data)
            }
        } catch (error) {
            console.error('Failed to load analytics:', error)
            showToast('Failed to load analytics data', 'error')
        } finally {
            setLoading(false)
        }
    }

    const totalPayments = (analytics?.successful_payments || 0) + (analytics?.failed_payments || 0)
    const successRate = totalPayments > 0
        ? ((analytics?.successful_payments || 0) / totalPayments * 100).toFixed(1)
        : '0'

    // Format chart data
    const chartData = analytics?.payment_trends?.map(point => ({
        ...point,
        volume: parseFloat(point.volume_usd),
        displayDate: new Date(point.date).toLocaleDateString(undefined, { month: 'short', day: 'numeric' })
    })) || []

    return (
        <div className={styles.page}>
            <div className={styles.header}>
                <div>
                    <h1><i className="fas fa-chart-line"></i> Analytics</h1>
                    <p>Insights into your business performance and revenue</p>
                </div>
                <div className={styles.filters}>
                    <div className={styles.dateInput}>
                        <label>From</label>
                        <input
                            type="date"
                            value={dateRange.from_date}
                            onChange={(e) => setDateRange(prev => ({ ...prev, from_date: e.target.value }))}
                        />
                    </div>
                    <div className={styles.dateInput}>
                        <label>To</label>
                        <input
                            type="date"
                            value={dateRange.to_date}
                            onChange={(e) => setDateRange(prev => ({ ...prev, to_date: e.target.value }))}
                        />
                    </div>
                    <button className={styles.refreshBtn} onClick={loadAnalytics} disabled={loading}>
                        <i className={`fas fa-sync-alt ${loading ? 'fa-spin' : ''}`}></i>
                    </button>
                </div>
            </div>

            {loading && !analytics ? (
                <div className={styles.loadingState}>
                    <i className="fas fa-spinner fa-spin"></i>
                    <p>Calculating your insights...</p>
                </div>
            ) : (
                <div className={styles.content}>
                    <div className={styles.statsGrid}>
                        <div className={styles.statCard}>
                            <div className={styles.statHeader}>
                                <span className={styles.statLabel}>Total Volume</span>
                                <i className="fas fa-money-bill-wave text-blue-500"></i>
                            </div>
                            <div className={styles.statValue}>${parseFloat(analytics?.total_volume_usd || '0').toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 })}</div>
                            <div className={styles.statFooter}>Total revenue processed</div>
                        </div>

                        <div className={styles.statCard}>
                            <div className={styles.statHeader}>
                                <span className={styles.statLabel}>Success Rate</span>
                                <i className="fas fa-check-circle text-green-500"></i>
                            </div>
                            <div className={styles.statValue}>{successRate}%</div>
                            <div className={styles.statFooter}>{analytics?.successful_payments || 0} successful / {totalPayments} total</div>
                        </div>

                        <div className={styles.statCard}>
                            <div className={styles.statHeader}>
                                <span className={styles.statLabel}>Avg. Transaction</span>
                                <i className="fas fa-percentage text-purple-500"></i>
                            </div>
                            <div className={styles.statValue}>${parseFloat(analytics?.average_transaction_value || '0').toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 })}</div>
                            <div className={styles.statFooter}>Average value per payment</div>
                        </div>

                        <div className={styles.statCard}>
                            <div className={styles.statHeader}>
                                <span className={styles.statLabel}>Total Fees</span>
                                <i className="fas fa-hand-holding-usd text-orange-500"></i>
                            </div>
                            <div className={styles.statValue}>${parseFloat(analytics?.total_fees_paid || '0').toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 })}</div>
                            <div className={styles.statFooter}>Networking and processing fees</div>
                        </div>
                    </div>

                    <div className={styles.chartContainer}>
                        <div className={styles.sectionHeader}>
                            <h2>Revenue Trend</h2>
                            <p>Daily processing volume in USD</p>
                        </div>
                        <div className={styles.chartWrapper}>
                            <ResponsiveContainer width="100%" height={350}>
                                <AreaChart data={chartData}>
                                    <defs>
                                        <linearGradient id="colorVolume" x1="0" y1="0" x2="0" y2="1">
                                            <stop offset="5%" stopColor="var(--fiddu-brand-primary)" stopOpacity={0.3} />
                                            <stop offset="95%" stopColor="var(--fiddu-brand-primary)" stopOpacity={0} />
                                        </linearGradient>
                                    </defs>
                                    <CartesianGrid strokeDasharray="3 3" vertical={false} stroke="#f0f0f0" />
                                    <XAxis
                                        dataKey="displayDate"
                                        axisLine={false}
                                        tickLine={false}
                                        tick={{ fill: '#94a3b8', fontSize: 12 }}
                                        dy={10}
                                    />
                                    <YAxis
                                        axisLine={false}
                                        tickLine={false}
                                        tick={{ fill: '#94a3b8', fontSize: 12 }}
                                        tickFormatter={(value: number | string) => `$${value}`}
                                    />
                                    <Tooltip
                                        contentStyle={{
                                            borderRadius: '12px',
                                            border: 'none',
                                            boxShadow: '0 10px 15px -3px rgba(0,0,0,0.1)',
                                            padding: '12px'
                                        }}
                                        formatter={(value: any) => [`$${parseFloat(value).toLocaleString()}`, 'Volume']}
                                    />
                                    <Area
                                        type="monotone"
                                        dataKey="volume"
                                        stroke="var(--fiddu-brand-primary)"
                                        strokeWidth={3}
                                        fillOpacity={1}
                                        fill="url(#colorVolume)"
                                        animationDuration={1500}
                                    />
                                </AreaChart>
                            </ResponsiveContainer>
                        </div>
                    </div>

                    <div className={styles.mainGrid}>
                        <div className={styles.chartSection}>
                            <div className={styles.sectionHeader}>
                                <h2>Network Breakdown</h2>
                                <p>Volume distribution across blockchain networks</p>
                            </div>
                            <div className={styles.networkList}>
                                {analytics?.by_blockchain && Object.keys(analytics.by_blockchain).length > 0 ? (
                                    Object.entries(analytics.by_blockchain).sort((a, b) => parseFloat(b[1].volume_usd) - parseFloat(a[1].volume_usd)).map(([network, stats]) => (
                                        <div key={network} className={styles.networkItem}>
                                            <div className={styles.networkInfo}>
                                                <span className={styles.networkName}>{network}</span>
                                                <span className={styles.networkCount}>{stats.payment_count} payments</span>
                                            </div>
                                            <div className={styles.networkValue}>
                                                <span className={styles.networkUsd}>${parseFloat(stats.volume_usd).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 })}</span>
                                                <div className={styles.progressBar}>
                                                    <div
                                                        className={styles.progressFill}
                                                        style={{
                                                            width: `${(parseFloat(stats.volume_usd) / (parseFloat(analytics.total_volume_usd) || 1) * 100)}%`
                                                        }}
                                                    ></div>
                                                </div>
                                            </div>
                                        </div>
                                    ))
                                ) : (
                                    <div className={styles.emptyState}>
                                        <i className="fas fa-chart-line"></i>
                                        <p>No network data available for this period.</p>
                                    </div>
                                )}
                            </div>
                        </div>

                        <div className={styles.recentActivity}>
                            <div className={styles.sectionHeader}>
                                <h2>Performance Overview</h2>
                            </div>
                            <div className={styles.summaryBox}>
                                <div className={styles.summaryItem}>
                                    <span className={styles.label}>Successful Payments</span>
                                    <span className={styles.value + ' ' + styles.positive}>{analytics?.successful_payments || 0}</span>
                                </div>
                                <div className={styles.summaryItem}>
                                    <span className={styles.label}>Failed/Expired</span>
                                    <span className={styles.value + ' ' + styles.negative}>{analytics?.failed_payments || 0}</span>
                                </div>
                                <div className={styles.summaryItem}>
                                    <span className={styles.label}>Gross Processing</span>
                                    <span className={styles.value}>${parseFloat(analytics?.total_volume_usd || '0').toLocaleString()}</span>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            )}
        </div>
    )
}

export default AnalyticsPage
