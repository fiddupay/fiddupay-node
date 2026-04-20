import React from 'react';
import Skeleton from '../ui/Skeleton';
import dashboardStyles from '@/styles/pages/DashboardPage.module.css';
import balanceStyles from '@/styles/pages/BalancePage.module.css';
import paymentsStyles from '@/styles/pages/PaymentsPage.module.css';
import walletStyles from '@/styles/pages/WalletsPage.module.css';

export const DashboardSkeleton: React.FC = () => {
    return (
        <div className={dashboardStyles.page}>
            <div className={dashboardStyles.header}>
                <div style={{ width: '300px' }}>
                    <Skeleton height={32} width="80%" borderRadius={8} />
                    <Skeleton height={16} width="60%" borderRadius={4} style={{ marginTop: '8px' }} />
                </div>
            </div>

            <div className={dashboardStyles.statsGrid}>
                {[1, 2, 3, 4].map((i) => (
                    <div key={i} className={dashboardStyles.statCard} style={{ background: 'rgba(255,255,255,0.03)', border: '1px solid var(--border)' }}>
                        <Skeleton variant="circle" width={48} height={48} />
                        <div style={{ flex: 1 }}>
                            <Skeleton height={14} width="40%" />
                            <Skeleton height={24} width="70%" style={{ marginTop: '8px' }} />
                        </div>
                    </div>
                ))}
            </div>

            <div style={{ display: 'grid', gridTemplateColumns: '1fr 340px', gap: '24px' }}>
                <div className={dashboardStyles.feedSection}>
                    <Skeleton height={400} borderRadius={24} />
                </div>
                <div className={dashboardStyles.securitySection}>
                    <Skeleton height={400} borderRadius={24} />
                </div>
            </div>
        </div>
    );
};

export const BalanceSkeleton: React.FC = () => {
    return (
        <div className={balanceStyles.page}>
            <div className={balanceStyles.header}>
                <div style={{ width: '400px' }}>
                    <Skeleton height={36} width="70%" borderRadius={10} />
                    <Skeleton height={18} width="50%" borderRadius={5} style={{ marginTop: '10px' }} />
                </div>
            </div>

            <div className={balanceStyles.statsGrid}>
                {[1, 2, 3].map((i) => (
                    <div key={i} className={balanceStyles.glassStatCard} style={{ border: 'none' }}>
                        <Skeleton variant="circle" width={64} height={64} />
                        <div style={{ flex: 1 }}>
                            <Skeleton height={14} width="50%" />
                            <Skeleton height={32} width="80%" style={{ marginTop: '10px' }} />
                        </div>
                    </div>
                ))}
            </div>

            <div className={balanceStyles.chartsGrid}>
                <Skeleton height={400} borderRadius={32} />
                <Skeleton height={400} borderRadius={32} />
            </div>

            <div className={balanceStyles.assetsGrid} style={{ marginTop: '2rem' }}>
                {[1, 2, 3, 4, 5, 6].map((i) => (
                    <div key={i} className={balanceStyles.assetGlassCard} style={{ border: 'none' }}>
                        <div style={{ display: 'flex', justifyContent: 'space-between' }}>
                            <Skeleton variant="circle" width={52} height={52} />
                            <Skeleton width={60} height={20} borderRadius={8} />
                        </div>
                        <Skeleton height={20} width="40%" style={{ marginTop: '20px' }} />
                        <Skeleton height={30} width="80%" style={{ marginTop: '10px' }} />
                        <div style={{ marginTop: '20px', paddingTop: '16px', borderTop: '1px solid var(--border)' }}>
                            <Skeleton height={14} width="60%" />
                        </div>
                    </div>
                ))}
            </div>
        </div>
    );
};

export const TableSkeleton: React.FC<{ rows?: number; columns?: number }> = ({ rows = 5 }) => {
    return (
        <div className={paymentsStyles.table}>
            {[...Array(rows)].map((_, i) => (
                <div key={i} className={paymentsStyles.tableRow} style={{ borderBottom: '1px solid var(--border)' }}>
                    <div className={paymentsStyles.tableCell}><Skeleton height={20} width="80%" /></div>
                    <div className={paymentsStyles.tableCell}><Skeleton height={20} width="60%" /></div>
                    <div className={paymentsStyles.tableCell}><Skeleton height={20} width="40%" /></div>
                    <div className={paymentsStyles.tableCell}><Skeleton height={20} width="70%" /></div>
                    <div className={paymentsStyles.tableCell}><Skeleton height={20} width="90%" /></div>
                    <div className={paymentsStyles.tableCell}><Skeleton height={20} width="30%" /></div>
                </div>
            ))}
        </div>
    );
};

export const StatCardSkeletons: React.FC = () => {
    return (
        <div className={paymentsStyles.stats}>
            {[1, 2, 3].map((i) => (
                <div key={i} className={paymentsStyles.statCard} style={{ border: 'none' }}>
                    <Skeleton variant="circle" width={48} height={48} />
                    <div style={{ flex: 1 }}>
                        <Skeleton height={14} width="40%" />
                        <Skeleton height={24} width="60%" style={{ marginTop: '8px' }} />
                    </div>
                </div>
            ))}
        </div>
    );
};

export const WalletGridSkeleton: React.FC = () => {
    return (
        <div className={walletStyles.walletGrid}>
            {[1, 2, 3, 4, 5, 6].map((i) => (
                <div key={i} className={walletStyles.walletCard} style={{ border: 'none', minHeight: '340px' }}>
                    <div className={walletStyles.walletHeader}>
                        <div className={walletStyles.coinInfo}>
                            <Skeleton variant="circle" width={48} height={48} />
                            <div className={walletStyles.coinDetails}>
                                <Skeleton height={20} width="80px" />
                                <Skeleton height={14} width="40px" style={{ marginTop: '4px' }} />
                            </div>
                        </div>
                        <Skeleton height={24} width="40px" borderRadius={12} />
                    </div>
                    <div className={walletStyles.walletContent} style={{ marginTop: '24px' }}>
                        <Skeleton height={14} width="40%" />
                        <Skeleton height={40} width="100%" style={{ marginTop: '12px' }} borderRadius={8} />
                        <div style={{ marginTop: '24px' }}>
                            <Skeleton height={16} width="60%" />
                        </div>
                    </div>
                </div>
            ))}
        </div>
    );
};
export const WithdrawalFormSkeleton: React.FC = () => {
    return (
        <div className={balanceStyles.page}>
            <div className={balanceStyles.header}>
                <div style={{ width: '400px' }}>
                    <Skeleton height={36} width="70%" borderRadius={10} />
                    <Skeleton height={18} width="50%" borderRadius={5} style={{ marginTop: '10px' }} />
                </div>
            </div>
            
            <div style={{ display: 'grid', gridTemplateColumns: '1fr 1.2fr', gap: '2rem', marginTop: '2rem' }}>
                <div style={{ background: 'var(--surface)', padding: '1.5rem', borderRadius: '1rem', border: '1px solid var(--border)' }}>
                    <Skeleton height={24} width="40%" style={{ marginBottom: '24px' }} />
                    {[1, 2, 3, 4].map(i => (
                        <div key={i} style={{ marginBottom: '20px' }}>
                            <Skeleton height={14} width="30%" style={{ marginBottom: '8px' }} />
                            <Skeleton height={44} width="100%" borderRadius={8} />
                        </div>
                    ))}
                    <Skeleton height={48} width="100%" borderRadius={8} style={{ marginTop: '32px' }} />
                </div>
                
                <div style={{ background: 'var(--surface)', padding: '1.5rem', borderRadius: '1rem', border: '1px solid var(--border)' }}>
                    <Skeleton height={24} width="40%" style={{ marginBottom: '24px' }} />
                    {[1, 2, 3, 4, 5].map(i => (
                        <div key={i} style={{ display: 'flex', justifyContent: 'space-between', padding: '16px 0', borderBottom: '1px solid var(--border)' }}>
                            <div style={{ flex: 1 }}>
                                <Skeleton height={16} width="40%" />
                                <Skeleton height={12} width="60%" style={{ marginTop: '8px' }} />
                            </div>
                            <Skeleton height={24} width="80px" borderRadius={12} />
                        </div>
                    ))}
                </div>
            </div>
        </div>
    );
};

export const AnalyticsSkeleton: React.FC = () => {
    return (
        <div className={dashboardStyles.page}>
            <div className={dashboardStyles.header}>
                <div style={{ width: '400px' }}>
                    <Skeleton height={32} width="60%" borderRadius={8} />
                    <Skeleton height={16} width="40%" borderRadius={4} style={{ marginTop: '8px' }} />
                </div>
                <div style={{ display: 'flex', gap: '12px' }}>
                    <Skeleton height={36} width="120px" borderRadius={6} />
                    <Skeleton height={36} width="120px" borderRadius={6} />
                </div>
            </div>

            <div className={dashboardStyles.statsGrid}>
                {[1, 2, 3, 4].map((i) => (
                    <div key={i} className={dashboardStyles.statCard} style={{ border: 'none' }}>
                        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start' }}>
                            <Skeleton height={14} width="50%" />
                            <Skeleton variant="circle" width={24} height={24} />
                        </div>
                        <Skeleton height={32} width="70%" style={{ marginTop: '12px' }} />
                        <Skeleton height={12} width="80%" style={{ marginTop: '12px' }} />
                    </div>
                ))}
            </div>

            <div style={{ background: 'var(--surface)', padding: '32px', borderRadius: '16px', border: '1px solid var(--border)', marginBottom: '32px' }}>
                <Skeleton height={24} width="20%" style={{ marginBottom: '8px' }} />
                <Skeleton height={14} width="15%" style={{ marginBottom: '24px' }} />
                <Skeleton height={350} width="100%" borderRadius={8} />
            </div>

            <div style={{ display: 'grid', gridTemplateColumns: '2fr 1fr', gap: '32px' }}>
                <div style={{ background: 'var(--surface)', padding: '32px', borderRadius: '16px', border: '1px solid var(--border)' }}>
                    <Skeleton height={24} width="30%" style={{ marginBottom: '24px' }} />
                    {[1, 2, 3, 4].map(i => (
                        <div key={i} style={{ marginBottom: '24px' }}>
                            <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '8px' }}>
                                <Skeleton height={16} width="40%" />
                                <Skeleton height={16} width="20%" />
                            </div>
                            <Skeleton height={8} width="100%" borderRadius={4} />
                        </div>
                    ))}
                </div>
                <div style={{ background: 'var(--surface)', padding: '32px', borderRadius: '16px', border: '1px solid var(--border)' }}>
                    <Skeleton height={24} width="60%" style={{ marginBottom: '24px' }} />
                    {[1, 2, 3].map(i => (
                        <div key={i} style={{ display: 'flex', justifyContent: 'space-between', padding: '16px', background: 'rgba(255,255,255,0.02)', borderRadius: '8px', marginBottom: '16px' }}>
                            <Skeleton height={14} width="50%" />
                            <Skeleton height={14} width="30%" />
                        </div>
                    ))}
                </div>
            </div>
        </div>
    );
};

export const SettingsSkeleton: React.FC = () => {
    return (
        <div style={{ padding: '24px' }}>
            <div style={{ marginBottom: '32px' }}>
                <Skeleton height={36} width="200px" borderRadius={8} />
                <Skeleton height={18} width="350px" borderRadius={4} style={{ marginTop: '12px' }} />
            </div>

            <div style={{ display: 'flex', gap: '24px', borderBottom: '1px solid var(--border)', marginBottom: '32px', paddingBottom: '2px' }}>
                {[1, 2, 3, 4, 5, 6].map(i => (
                    <Skeleton key={i} height={32} width={120} borderRadius={6} />
                ))}
            </div>

            <div style={{ background: 'var(--surface)', borderRadius: '12px', border: '1px solid var(--border)', padding: '32px' }}>
                <Skeleton height={24} width="30%" style={{ marginBottom: '12px' }} />
                <Skeleton height={14} width="60%" style={{ marginBottom: '32px' }} />
                
                <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(280px, 1fr))', gap: '16px' }}>
                    {[1, 2].map(i => (
                        <div key={i} style={{ padding: '24px', border: '2px solid var(--border)', borderRadius: '12px' }}>
                            <Skeleton variant="circle" width={48} height={48} />
                            <Skeleton height={20} width="60%" style={{ marginTop: '16px' }} />
                            <Skeleton height={14} width="90%" style={{ marginTop: '12px' }} />
                        </div>
                    ))}
                </div>
            </div>
        </div>
    );
};

export const ActivityListSkeleton: React.FC = () => {
    return (
        <div style={{ padding: '16px' }}>
            {[1, 2, 3, 4, 5].map((i) => (
                <div key={i} style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', padding: '12px 0', borderBottom: i < 5 ? '1px solid var(--border)' : 'none' }}>
                    <div style={{ display: 'flex', alignItems: 'center', gap: '12px', flex: 1 }}>
                        <Skeleton variant="circle" width={32} height={32} />
                        <div style={{ flex: 1 }}>
                            <Skeleton height={14} width="40%" />
                            <Skeleton height={10} width="30%" style={{ marginTop: '6px' }} />
                        </div>
                    </div>
                    <div style={{ textAlign: 'right' }}>
                        <Skeleton height={14} width="60px" />
                        <Skeleton height={10} width="40px" style={{ marginTop: '6px' }} />
                    </div>
                </div>
            ))}
        </div>
    );
};

export const SecurityHubSkeleton: React.FC = () => {
    return (
        <div style={{ padding: '24px' }}>
            <div style={{ marginBottom: '32px' }}>
                <Skeleton height={32} width="240px" borderRadius={8} />
                <Skeleton height={16} width="400px" borderRadius={4} style={{ marginTop: '12px' }} />
            </div>

            <div style={{ display: 'flex', gap: '8px', marginBottom: '24px', background: 'var(--surface)', padding: '6px', borderRadius: '10px', width: 'fit-content', border: '1px solid var(--border)' }}>
                {[1, 2, 3].map(i => (
                    <Skeleton key={i} height={36} width={140} borderRadius={6} />
                ))}
            </div>

            <div style={{ background: 'var(--surface)', borderRadius: '12px', border: '1px solid var(--border)', overflow: 'hidden' }}>
                {[1, 2, 3, 4, 5].map(i => (
                    <div key={i} style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', padding: '24px', borderBottom: '1px solid var(--border)' }}>
                        <div style={{ display: 'flex', gap: '16px', alignItems: 'flex-start', flex: 1 }}>
                            <Skeleton variant="circle" width={40} height={40} />
                            <div style={{ flex: 1 }}>
                                <Skeleton height={18} width="30%" />
                                <Skeleton height={14} width="70%" style={{ marginTop: '8px' }} />
                                <Skeleton height={10} width="100px" style={{ marginTop: '12px' }} />
                            </div>
                        </div>
                        <Skeleton height={36} width="100px" borderRadius={6} style={{ marginLeft: '24px' }} />
                    </div>
                ))}
            </div>
        </div>
    );
};
