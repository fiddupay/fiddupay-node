import React from 'react';
import styles from '@/styles/pages/StatusPage.module.css';

interface UptimeBarChartProps {
  days?: number;
  data?: Array<{ date: string; status: 'operational' | 'degraded' | 'outage' | 'maintenance' }>;
}

const UptimeBarChart: React.FC<UptimeBarChartProps> = ({ days = 14, data }) => {
  // If no data provided, generate mock data for the enterprise look
  const bars = data || Array.from({ length: days }).map((_, i) => {
    // Generate mostly green, some yellow/orange for realism
    const rand = Math.random();
    let status: 'operational' | 'degraded' | 'outage' | 'maintenance' = 'operational';
    if (rand > 0.98) status = 'outage';
    else if (rand > 0.95) status = 'degraded';
    else if (rand > 0.94) status = 'maintenance';
    
    return {
      date: new Date(Date.now() - (days - i) * 86400000).toLocaleDateString(),
      status
    };
  });

  const operationalCount = bars.filter(b => b.status === 'operational').length;
  const uptimeRatio = bars.length > 0 ? (operationalCount / bars.length) * 100 : 100;
  const uptimeDisplay = `${uptimeRatio.toFixed(2).replace(/\.00$/, '')}% uptime`;

  return (
    <div className={styles.uptimeBarChart}>
      <div className={styles.barsContainer}>
        {bars.map((bar, i) => (
          <div 
            key={i} 
            className={`${styles.bar} ${styles[bar.status]}`}
            title={`${bar.date}: ${bar.status.charAt(0).toUpperCase() + bar.status.slice(1)}`}
          />
        ))}
      </div>
      <div className={styles.chartFooter}>
        <span>{days} days ago</span>
        <div className={styles.divider} />
        <span>{uptimeDisplay}</span>
        <div className={styles.divider} />
        <span>Today</span>
      </div>
    </div>
  );
};

export default UptimeBarChart;
