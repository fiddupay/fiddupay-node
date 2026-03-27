import React, { useState } from 'react';
import { 
    Activity, 
    Database, 
    Cpu, 
    HardDrive, 
    Terminal,
    History,
    ShieldCheck,
    CheckCircle,
    Clock,
    Search,
    RefreshCw
} from 'lucide-react';

const SystemPage: React.FC = () => {
    const [activeTab, setActiveTab] = useState<'health' | 'logs' | 'audit'>('health');

    const nodeStatus = [
        { name: 'Ethereum Mainnet', status: 'connected', latency: '45ms', blocks: '19,245,120' },
        { name: 'Solana Mainnet', status: 'connected', latency: '12ms', blocks: '254,120,450' },
        { name: 'BSC Mainnet', status: 'connected', latency: '32ms', blocks: '36,450,120' },
        { name: 'Bitcoin Mainnet', status: 'connected', latency: '120ms', blocks: '835,120' },
    ];

    const logs = [
        { level: 'info', message: 'Payment successfully processed for Merchant ID: 12', time: '14:05:22', module: 'PAYMENT_PROC' },
        { level: 'warn', message: 'Rate limit approaching for IP: 192.168.1.45', time: '14:02:10', module: 'API_GATEWAY' },
        { level: 'error', message: 'Failed to broadcast transaction on SOL network', time: '13:58:45', module: 'WATCHER_SOL' },
        { level: 'info', message: 'System health check: All services operational', time: '13:55:00', module: 'MONITOR' },
    ];

    return (
        <div className="space-y-6 animate-in fade-in duration-500">
            <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
                <div>
                    <h1 className="text-2xl font-bold text-slate-900 tracking-tight">System Administration</h1>
                    <p className="text-slate-500 text-sm mt-1">Monitor system health, node connections, and operational logs.</p>
                </div>
                <div className="flex bg-slate-100 p-1 rounded-lg border border-slate-200 shadow-sm">
                    {['health', 'logs', 'audit'].map((tab) => (
                        <button
                            key={tab}
                            onClick={() => setActiveTab(tab as any)}
                            className={`px-4 py-1.5 rounded-md text-xs font-bold transition-all ${
                                activeTab === tab 
                                ? 'bg-white text-primary-600 shadow-sm' 
                                : 'text-slate-500 hover:text-slate-700'
                            }`}
                        >
                            {tab.charAt(0).toUpperCase() + tab.slice(1)}
                        </button>
                    ))}
                </div>
            </div>

            {activeTab === 'health' && (
                <div className="space-y-6 animate-in slide-in-from-bottom-2 duration-500">
                    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
                        {[
                            { label: 'CPU Usage', value: '12%', icon: Cpu, color: 'emerald' },
                            { label: 'RAM Usage', value: '42%', icon: Activity, color: 'blue' },
                            { label: 'Disk Space', value: '68%', icon: HardDrive, color: 'amber' },
                            { label: 'Active Tasks', value: '142', icon: Clock, color: 'indigo' }
                        ].map((stat, i) => (
                            <div key={i} className="bg-white p-6 rounded-xl border border-slate-200 shadow-sm">
                                <div className="flex items-center justify-between mb-4">
                                    <div className={`p-2 bg-${stat.color}-50 text-${stat.color}-600 rounded-lg`}>
                                        <stat.icon size={20} />
                                    </div>
                                    <span className={`text-${stat.color}-600 text-xs font-bold`}>Normal</span>
                                </div>
                                <div className="text-sm font-medium text-slate-500">{stat.label}</div>
                                <div className="text-2xl font-bold text-slate-900 mt-1">{stat.value}</div>
                                <div className="w-full bg-slate-100 h-1.5 rounded-full mt-4 overflow-hidden">
                                     <div 
                                        className={`bg-${stat.color}-500 h-full rounded-full transition-all duration-1000`} 
                                        style={{ width: stat.value }}
                                    ></div>
                                </div>
                            </div>
                        ))}
                    </div>

                    <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
                        <div className="bg-white rounded-xl border border-slate-200 shadow-sm overflow-hidden">
                            <div className="p-4 border-b border-slate-100 bg-slate-50/50 flex items-center justify-between">
                                <h2 className="text-sm font-bold text-slate-900 flex items-center gap-2">
                                    <Database size={18} className="text-blue-500" />
                                    Blockchain Nodes
                                </h2>
                                <button className="p-2 text-slate-400 hover:text-primary-600 hover:bg-primary-50 rounded-lg transition-all">
                                    <RefreshCw size={16} />
                                </button>
                            </div>
                            <div className="divide-y divide-slate-100">
                                {nodeStatus.map((node, i) => (
                                    <div key={i} className="px-6 py-4 flex items-center justify-between hover:bg-slate-50 transition-colors">
                                        <div>
                                            <div className="text-sm font-bold text-slate-900">{node.name}</div>
                                            <div className="text-[10px] text-slate-500 font-mono">Height: {node.blocks}</div>
                                        </div>
                                        <div className="text-right">
                                            <div className="flex items-center gap-2 justify-end">
                                                <span className="text-[10px] font-bold text-emerald-600 uppercase tracking-widest">{node.status}</span>
                                                <div className="w-2 h-2 rounded-full bg-emerald-500 animate-pulse"></div>
                                            </div>
                                            <div className="text-[10px] text-slate-400">Latency: {node.latency}</div>
                                        </div>
                                    </div>
                                ))}
                            </div>
                        </div>

                        <div className="bg-white rounded-xl border border-slate-200 shadow-sm overflow-hidden flex flex-col">
                            <div className="p-4 border-b border-slate-100 bg-slate-50/50 flex items-center justify-between">
                                <h2 className="text-sm font-bold text-slate-900 flex items-center gap-2">
                                    <ShieldCheck size={18} className="text-emerald-500" />
                                    Security Status
                                </h2>
                            </div>
                            <div className="p-6 flex-1 flex flex-col justify-center items-center text-center space-y-4">
                                <div className="w-20 h-20 rounded-full bg-emerald-100 flex items-center justify-center text-emerald-600">
                                    <CheckCircle size={40} />
                                </div>
                                <div>
                                    <h3 className="font-bold text-slate-900">All Systems Secure</h3>
                                    <p className="text-sm text-slate-500 mt-1 max-w-xs">No active security alerts or suspicious activities detected in the last 24 hours.</p>
                                </div>
                                <button className="px-6 py-2 bg-slate-900 text-white text-sm font-bold rounded-lg hover:bg-slate-800 transition-colors shadow-sm">
                                    Run Security Audit
                                </button>
                            </div>
                        </div>
                    </div>
                </div>
            )}

            {activeTab === 'logs' && (
                <div className="bg-slate-900 rounded-xl border border-slate-800 shadow-2xl overflow-hidden animate-in zoom-in-95 duration-300">
                    <div className="p-4 border-b border-slate-800 flex items-center justify-between bg-slate-950">
                        <div className="flex items-center gap-2 text-slate-400 font-mono text-xs">
                            <Terminal size={14} />
                            system_logs.sh
                        </div>
                        <div className="flex gap-2">
                            <div className="w-3 h-3 rounded-full bg-rose-500/50 group-hover:bg-rose-500 transition-colors"></div>
                            <div className="w-3 h-3 rounded-full bg-amber-500/50 group-hover:bg-amber-500 transition-colors"></div>
                            <div className="w-3 h-3 rounded-full bg-emerald-500/50 group-hover:bg-emerald-500 transition-colors"></div>
                        </div>
                    </div>
                    <div className="p-4 font-mono text-[13px] leading-relaxed max-h-[500px] overflow-y-auto">
                        <div className="text-slate-500 flex items-center gap-2 mb-4 border-b border-slate-800 pb-2">
                             <Search size={14} />
                             <input type="text" placeholder="Filter logs..." className="bg-transparent border-none outline-none w-full text-slate-300" />
                        </div>
                        {logs.map((log, i) => (
                            <div key={i} className="group hover:bg-slate-800/50 -mx-4 px-4 py-1.5 transition-colors">
                                <span className="text-slate-500 mr-3">[{log.time}]</span>
                                <span className={`mr-3 uppercase font-bold text-[10px] px-1.5 py-0.5 rounded ${
                                    log.level === 'error' ? 'bg-rose-500/20 text-rose-400' :
                                    log.level === 'warn' ? 'bg-amber-500/20 text-amber-400' :
                                    'bg-emerald-500/20 text-emerald-400'
                                }`}>
                                    {log.level}
                                </span>
                                <span className="text-slate-400 mr-3">[{log.module}]</span>
                                <span className={log.level === 'error' ? 'text-rose-300' : 'text-slate-300'}>{log.message}</span>
                            </div>
                        ))}
                        <div className="mt-4 flex items-center gap-2 text-emerald-500 animate-pulse">
                            <span className="text-slate-500">$</span>
                             tail -f system/service.log
                             <span className="w-2 h-4 bg-emerald-500 ml-1"></span>
                        </div>
                    </div>
                </div>
            )}

            {activeTab === 'audit' && (
                <div className="bg-white rounded-xl border border-slate-200 shadow-sm overflow-hidden min-h-[400px] flex flex-col justify-center items-center text-slate-400 space-y-3">
                     <History size={48} className="opacity-20" />
                     <div className="text-sm font-medium">Audit logs are being indexed...</div>
                     <p className="text-xs max-w-xs text-center">Historical audit data is currently being migrated to the new schema. Please check back in a few minutes.</p>
                </div>
            )}
        </div>
    );
};

export default SystemPage;
