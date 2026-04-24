import React from 'react';
import { Badge } from './badge';
import { 
  MdShield, 
  MdWarning, 
  MdCheckCircle, 
  MdOutlineCircle 
} from 'react-icons/md';
import { User } from '@/types';
import styles from '@/styles/components/TrustScoreWidget.module.css';

interface TrustScoreWidgetProps {
  user: User | null;
  className?: string;
}

export const TrustScoreWidget: React.FC<TrustScoreWidgetProps> = ({ user, className = "" }) => {
  if (!user || !user.trust_score) {
    return (
      <div className={className} style={{ position: 'relative', overflow: 'hidden', height: '100%', display: 'flex', flexDirection: 'column', justifyContent: 'center', alignItems: 'center', padding: '20px', textAlign: 'center' }}>
        <MdShield style={{ color: 'var(--primary)', width: '48px', height: '48px', marginBottom: '16px', opacity: 0.2 }} className="animate-pulse" />
        <div style={{ fontSize: '12px', fontWeight: '800', color: 'var(--text-muted)', textTransform: 'uppercase', letterSpacing: '0.05em' }}>
          Syncing Trust Intelligence...
        </div>
        <p style={{ fontSize: '10px', color: 'var(--text-muted)', marginTop: '8px' }}>Feeding the Swarm signals</p>
      </div>
    );
  }

  const { score, tier, identity_verified, social_verified, business_verified } = user.trust_score;

  const getTierGradient = (t: string) => {
    switch (t) {
      case 'Gold': return 'linear-gradient(135deg, #fbbf24, #d97706)';
      case 'Silver': return 'linear-gradient(135deg, #cbd5e1, #64748b)';
      default: return 'linear-gradient(135deg, #fb923c, #c2410c)';
    }
  };

  const getScoreColor = (s: number) => {
    if (s >= 80) return '#22c55e';
    if (s >= 60) return '#3b82f6';
    if (s >= 40) return '#f59e0b';
    return '#ef4444';
  };

  const checklistItems = [
    { label: 'Identity Agent (NIN/BVN)', status: identity_verified, points: 60 },
    { label: 'Social Signal Agent (Handles)', status: social_verified, points: 20 },
    { label: 'Reputation Agent (History)', status: business_verified, points: 20 },
  ];

  return (
    <div className={`${className} ${styles.widgetRoot}`}>
      <div style={{ height: '4px', width: '100%', background: getTierGradient(tier) }} />
      
      <div style={{ padding: '20px' }}>
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '16px' }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: '8px', fontSize: '10px', fontWeight: '800', color: 'var(--text-muted)', textTransform: 'uppercase', letterSpacing: '0.05em' }}>
            <MdShield style={{ color: 'var(--primary)', width: '16px', height: '16px' }} />
            Trust Intelligence
          </div>
          <Badge style={{ background: getTierGradient(tier), color: 'white', border: 'none', padding: '4px 10px', fontWeight: '800', fontSize: '10px' }}>
            {tier} TIER
          </Badge>
        </div>

        <div className={styles.widgetBody}>
          {/* Left: Score + Checklist */}
          <div className={styles.widgetLeft}>
            <div className={styles.healthSection}>
              <div className={styles.score} style={{ color: getScoreColor(score) }}>
                {score}<span className={styles.percent}>%</span>
              </div>
              <div className={styles.healthMeta}>
                <div className={styles.healthLabel}>Network Reputation</div>
                <div className={styles.healthSub}>Agent Consensus</div>
              </div>
            </div>

            <div className={styles.checklist}>
              {checklistItems.map((item, idx) => (
                <div key={idx} className={styles.checkItem}>
                  <div className={`${styles.checkLabel} ${item.status ? styles.checkLabelActive : ''}`}>
                    {item.status ? (
                      <MdCheckCircle style={{ color: '#22c55e', width: '20px', height: '20px' }} />
                    ) : (
                      <MdOutlineCircle style={{ color: 'rgba(255,255,255,0.1)', width: '20px', height: '20px' }} />
                    )}
                    <span>{item.label}</span>
                  </div>
                  <span className={styles.points} style={{ color: item.status ? '#22c55e' : 'inherit' }}>
                    +{item.points}
                  </span>
                </div>
              ))}
            </div>
          </div>

          {/* Center: Pulse */}
          <div className={styles.widgetCenter}>
            <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: '9px', fontWeight: '900', textTransform: 'uppercase', letterSpacing: '0.1em', color: 'var(--text-muted)' }}>
              <span>Swarm Pulse</span>
              <span style={{ color: 'var(--primary)' }}>LIVE SIGNAL</span>
            </div>
            <div className={styles.pulseBar}>
              {[...Array(12)].map((_, i) => (
                <div 
                  key={i} 
                  className={`${styles.pulseDot} ${i < (score / 8) ? styles.pulseDotActive : ''} ${i === Math.floor(score / 8) ? styles.pulseDotCurrent : ''}`} 
                />
              ))}
            </div>
          </div>

          {/* Right: Suggestion */}
          {score < 100 && (
            <div className={styles.widgetRight}>
              <div style={{ padding: '12px', borderRadius: '12px', background: 'rgba(99, 102, 241, 0.05)', border: '1px solid rgba(99, 102, 241, 0.1)', display: 'flex', gap: '12px' }}>
                <MdWarning style={{ color: 'var(--primary)', width: '16px', height: '16px', flexShrink: 0 }} />
                <p style={{ fontSize: '10px', color: 'var(--text-muted)', lineHeight: '1.4', margin: 0 }}>
                  Unlock 0-fee interoperability by feeding the Swarm more signals. 
                  {score < 60 ? " Start with Identity verification." : " Link Social Signals for Gold status."}
                </p>
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};
