import React from 'react';
import Skeleton from '../ui/Skeleton';
import dashboardStyles from '@/styles/pages/DashboardPage.module.css';
import balanceStyles from '@/styles/pages/BalancePage.module.css';

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
                    <div key={i} className={dashboardStyles.statCard} style={{ background: 'rgba(255,255,255,0.5)', border: 'none' }}>
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
                        <div style={{ marginTop: '20px', paddingTop: '16px', borderTop: '1px solid #f1f5f9' }}>
                            <Skeleton height={14} width="60%" />
                        </div>
                    </div>
                ))}
            </div>
        </div>
    );
};
