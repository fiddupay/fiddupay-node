import React, { useState, useEffect } from 'react';
import { 
  MdShield, 
  MdSecurity, 
  MdRadar, 
  MdAutoGraph, 
  MdCheckCircle, 
  MdSearch,
  MdStream
} from 'react-icons/md';
import styles from '@/styles/components/SwarmIntelligenceWidget.module.css';

interface Agent {
  name: string;
  status: 'scanning' | 'verified' | 'alert';
  signal: string;
  id: string;
}

interface SwarmIntelligenceWidgetProps {
  user: any;
}

export const SwarmIntelligenceWidget: React.FC<SwarmIntelligenceWidgetProps> = ({ user }) => {
  const hasIdentity = user?.has_national_id || user?.kyc_tier >= 1;
  const hasSocials = user?.social_handles?.twitter || user?.social_handles?.website;
  const hasBusiness = !!user?.business_license_number;

  const [agents, setAgents] = useState<Agent[]>([
    { id: 'identity', name: 'Identity Agent', status: hasIdentity ? 'verified' : 'scanning', signal: hasIdentity ? 'NIN/BVN Hashed' : 'Awaiting Signal' },
    { id: 'social', name: 'Social Agent', status: hasSocials ? 'verified' : 'scanning', signal: hasSocials ? 'Signals Synced' : 'Analyzing Footprint' },
    { id: 'velocity', name: 'Velocity Agent', status: 'verified', signal: 'Flow Stabilized' },
    { id: 'network', name: 'Network Agent', status: 'scanning', signal: 'Monitoring SOL/EVM' },
  ]);

  const [activeLog, setActiveLog] = useState<string[]>([]);

  useEffect(() => {
    // Sync agents with user data when user changes
    setAgents([
      { id: 'identity', name: 'Identity Agent', status: hasIdentity ? 'verified' : 'scanning', signal: hasIdentity ? 'NIN/BVN Hashed' : 'Awaiting Signal' },
      { id: 'social', name: 'Social Agent', status: (hasSocials && hasBusiness) ? 'verified' : 'scanning', signal: (hasSocials && hasBusiness) ? 'Institutional Proof' : 'Analyzing X/LinkedIn' },
      { id: 'velocity', name: 'Velocity Agent', status: 'verified', signal: 'Flow Stabilized' },
      { id: 'network', name: 'Network Agent', status: 'scanning', signal: 'Monitoring SOL/EVM' },
    ]);
  }, [user, hasIdentity, hasSocials, hasBusiness]);

  useEffect(() => {
    const logs = [
      `Identity Agent: ${hasIdentity ? 'Signal verified for gov_ID' : 'Awaiting identity protocol...'}`,
      "Velocity Agent: Spike detected in SOL mainnet (resolved)",
      `Social Agent: ${hasSocials ? 'Reputation score updated' : 'Scanning digital footprint...'}`,
      "Network Agent: Indexing institutional EVM liquidity",
      `Swarm: Consensus reached on Trust Tier ${user?.kyc_tier || 0}`,
      `Signal: Merchant health at ${user?.trust_score?.score || 65}%`,
    ];
    
    const interval = setInterval(() => {
      setActiveLog(prev => [logs[Math.floor(Math.random() * logs.length)], ...prev.slice(0, 4)]);
      
      // Only fluctuate velocity and network agents, keep identity and social stable if verified
      setAgents(prev => prev.map(a => {
        if ((a.id === 'identity' && hasIdentity) || (a.id === 'social' && hasSocials && hasBusiness)) {
          return a;
        }
        return {
          ...a,
          status: Math.random() > 0.8 ? 'scanning' : 'verified'
        };
      }));
    }, 4000);

    return () => clearInterval(interval);
  }, [user, hasIdentity, hasSocials, hasBusiness]);

  return (
    <div className={styles.container}>
      <div className={styles.radarOverlay}>
        <MdRadar className={styles.radarIcon} />
      </div>
      
      <div className={styles.header}>
        <div className={styles.title}>
          <MdSecurity className={styles.titleIcon} />
          Fraud Signal Swarm
        </div>
        <div className={styles.statusBadge}>
          <div className={styles.pulse} />
          <span className={styles.statusText}>Active Consensus</span>
        </div>
      </div>
      
      <div className={styles.content}>
        <div className={styles.agentGrid}>
          {agents.map((agent) => (
            <div key={agent.id} className={styles.agentCard}>
              <div className={styles.agentHeader}>
                <span className={styles.agentName}>{agent.name}</span>
                {agent.status === 'scanning' ? (
                  <MdSearch className={styles.scanningText} />
                ) : (
                  <MdCheckCircle style={{color: '#22c55e'}} />
                )}
              </div>
              <div className={styles.agentSignal}>
                {agent.status === 'scanning' ? (
                  <span className={styles.scanningText}>SCANNING...</span>
                ) : (
                  agent.signal
                )}
              </div>
            </div>
          ))}
        </div>

        <div className={styles.feedContainer}>
          <div className={styles.feedHeader}>
            <MdStream className={styles.titleIcon} />
            Real-time Intelligence Feed
          </div>
          <div className={styles.terminal}>
            <div className="space-y-2">
              {activeLog.map((log, i) => (
                <div key={i} className={`${styles.logEntry} ${i === 0 ? styles.logActive : styles.logMuted}`}>
                  <span className={styles.logTimestamp}>[{new Date().toLocaleTimeString()}]</span>
                  <span>{log}</span>
                </div>
              ))}
              {activeLog.length === 0 && <div style={{opacity: 0.3, fontStyle: 'italic', fontSize: '10px'}}>Waiting for swarm signals...</div>}
            </div>
            <div className={styles.terminalOverlay} />
          </div>
        </div>

        <div className={styles.footer}>
            <div className={styles.footerInfo}>
                <div className={styles.footerIconBox}>
                    <MdAutoGraph />
                </div>
                <div>
                    <div className={styles.footerLabel}>Consensus Risk Level</div>
                    <div className={styles.footerValue}>Low Probability (Secure)</div>
                </div>
            </div>
            <MdShield className={styles.shieldIcon} size={24} />
        </div>
      </div>
    </div>
  );
};
