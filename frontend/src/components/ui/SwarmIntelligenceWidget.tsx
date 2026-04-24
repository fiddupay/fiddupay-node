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
import { Card, CardHeader, CardTitle, CardContent } from './card';

interface Agent {
  name: string;
  status: 'scanning' | 'verified' | 'alert';
  signal: string;
  id: string;
}

export const SwarmIntelligenceWidget: React.FC = () => {
  const [agents, setAgents] = useState<Agent[]>([
    { id: 'identity', name: 'Identity Agent', status: 'verified', signal: 'NIN/BVN Hashed' },
    { id: 'social', name: 'Social Agent', status: 'scanning', signal: 'Analyzing X/LinkedIn' },
    { id: 'velocity', name: 'Velocity Agent', status: 'verified', signal: 'Flow Stabilized' },
    { id: 'network', name: 'Network Agent', status: 'scanning', signal: 'Monitoring SOL/EVM' },
  ]);

  const [activeLog, setActiveLog] = useState<string[]>([]);

  useEffect(() => {
    const logs = [
      "Identity Agent: Validating gov_signal_X92...",
      "Velocity Agent: Spike detected in SOL mainnet (resolved)",
      "Social Agent: Reputation score updated for @merchant",
      "Network Agent: Indexing institutional EVM liquidity",
      "Swarm: Consensus reached on Trust Tier 1",
      "Signal: Merchant health at 85%",
    ];
    
    const interval = setInterval(() => {
      setActiveLog(prev => [logs[Math.floor(Math.random() * logs.length)], ...prev.slice(0, 4)]);
      
      // Randomly change status to scanning
      setAgents(prev => prev.map(a => ({
        ...a,
        status: Math.random() > 0.7 ? 'scanning' : 'verified'
      })));
    }, 3000);

    return () => clearInterval(interval);
  }, []);

  return (
    <Card className="bg-[#0f172a]/80 border-primary/20 backdrop-blur-xl overflow-hidden">
      <div className="absolute top-0 right-0 p-4 opacity-10">
        <MdRadar className="w-24 h-24 animate-spin-slow" />
      </div>
      
      <CardHeader className="border-b border-white/5 pb-4">
        <div className="flex items-center justify-between">
          <CardTitle className="text-white flex items-center gap-2">
            <MdSecurity className="text-primary" />
            Fraud Signal Swarm
          </CardTitle>
          <div className="flex items-center gap-2 bg-green-500/10 px-3 py-1 rounded-full">
            <div className="w-2 h-2 bg-green-500 rounded-full animate-pulse" />
            <span className="text-[10px] font-black text-green-500 uppercase tracking-widest">Active Consenus</span>
          </div>
        </div>
      </CardHeader>
      
      <CardContent className="pt-6">
        <div className="grid grid-cols-2 gap-4 mb-6">
          {agents.map((agent) => (
            <div key={agent.id} className="bg-white/5 p-3 rounded-xl border border-white/10 group hover:border-primary/50 transition-all">
              <div className="flex items-center justify-between mb-2">
                <span className="text-[10px] font-bold text-gray-500 uppercase">{agent.name}</span>
                {agent.status === 'scanning' ? (
                  <MdSearch className="text-primary animate-bounce" />
                ) : (
                  <MdCheckCircle className="text-green-500" />
                )}
              </div>
              <div className="text-xs font-mono text-gray-300 truncate">
                {agent.status === 'scanning' ? (
                  <span className="animate-pulse">SCANNING...</span>
                ) : (
                  agent.signal
                )}
              </div>
            </div>
          ))}
        </div>

        <div className="space-y-2">
          <div className="flex items-center gap-2 text-xs font-bold text-gray-500 uppercase tracking-widest mb-3">
            <MdStream className="text-primary" />
            Real-time Intelligence Feed
          </div>
          <div className="bg-black/40 rounded-xl p-4 font-mono text-[10px] h-32 overflow-hidden relative">
            <div className="space-y-2">
              {activeLog.map((log, i) => (
                <div key={i} className={`flex gap-2 ${i === 0 ? 'text-primary' : 'text-gray-500'}`}>
                  <span className="opacity-50">[{new Date().toLocaleTimeString()}]</span>
                  <span>{log}</span>
                </div>
              ))}
              {activeLog.length === 0 && <div className="text-gray-700 italic">Waiting for swarm signals...</div>}
            </div>
            <div className="absolute bottom-0 left-0 right-0 h-8 bg-gradient-to-t from-[#0a0f1c] to-transparent" />
          </div>
        </div>

        <div className="mt-6 p-4 bg-primary/5 border border-primary/20 rounded-xl flex items-center justify-between">
            <div className="flex items-center gap-3">
                <div className="w-10 h-10 rounded-lg bg-primary/20 flex items-center justify-center text-primary">
                    <MdAutoGraph />
                </div>
                <div>
                    <div className="text-[10px] text-gray-500 font-bold uppercase">Consensus Risk Level</div>
                    <div className="text-sm font-black text-white uppercase tracking-tight">Low Probability (Secure)</div>
                </div>
            </div>
            <MdShield className="text-primary w-6 h-6 opacity-50" />
        </div>
      </CardContent>
    </Card>
  );
};
