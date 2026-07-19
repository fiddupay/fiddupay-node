import React, { useState } from 'react';
import { 
    ShieldAlert, 
    ShieldCheck, 
    Activity, 
    Ban, 
    Lock,
    RefreshCcw,
    Search
} from 'lucide-react';
import clsx from 'clsx';

const SecurityPage: React.FC = () => {
    const [activeTab, setActiveTab] = useState<'alerts' | 'ip_blocks' | 'audit'>('alerts');
    const [searchQuery, setSearchQuery] = useState('');

    const stats = [
        { label: 'Total Alerts', value: '0', icon: ShieldAlert, color: 'text-rose-400', bg: 'bg-[#151c2c] border-white/5' },
        { label: 'Blocked IPs', value: '2', icon: Ban, color: 'text-amber-400', bg: 'bg-[#151c2c] border-white/5' },
        { label: 'Security Score', value: '98/100', icon: ShieldCheck, color: 'text-emerald-400', bg: 'bg-[#151c2c] border-white/5' },
        { label: 'Active Sessions', value: '1', icon: Activity, color: 'text-primary-400', bg: 'bg-[#151c2c] border-white/5' },
    ];

    return (
        <div className="space-y-6 animate-in fade-in duration-500">
            <header className="flex justify-between items-end">
                <div>
                    <h2 className="text-2xl font-bold tracking-tight text-slate-100">Security Command Center</h2>
                    <p className="text-slate-400 text-sm mt-1">Monitor system-wide security events and manage global threat protections.</p>
                </div>
                <div className="flex gap-2">
                    <button className="flex items-center gap-2 px-4 py-2 border border-white/5 rounded-xl hover:bg-white/5 transition-colors text-sm font-semibold text-slate-300">
                        <RefreshCcw size={16} /> Refresh
                    </button>
                    <button className="flex items-center gap-2 px-4 py-2 bg-rose-600/20 border border-rose-500/30 text-rose-400 rounded-xl hover:bg-rose-600/30 transition-colors text-sm font-bold shadow-glow">
                        <Lock size={16} /> Global Lockdown
                    </button>
                </div>
            </header>

            {/* Stats Grid */}
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
                {stats.map((stat) => (
                    <div key={stat.label} className="bg-[#151c2c] p-6 rounded-2xl border border-white/5 shadow-sm">
                        <div className="flex items-center justify-between mb-4">
                            <div className={clsx('p-2.5 rounded-xl bg-white/5 border border-white/5')}>
                                <stat.icon className={clsx('w-5 h-5', stat.color)} />
                            </div>
                        </div>
                        <div className="space-y-1">
                            <p className="text-sm font-medium text-slate-400">{stat.label}</p>
                            <p className="text-2xl font-bold text-slate-100">{stat.value}</p>
                        </div>
                    </div>
                ))}
            </div>

            {/* Main Content */}
            <div className="bg-[#151c2c] rounded-2xl border border-white/5 shadow-sm overflow-hidden">
                <div className="border-b border-white/5 bg-[#1a2336]/40">
                    <div className="flex flex-col md:flex-row md:items-center justify-between px-6 py-4 gap-4">
                        <nav className="flex gap-1 p-1 bg-[#0b0f19] rounded-xl w-fit border border-white/5">
                            <button 
                                onClick={() => setActiveTab('alerts')}
                                className={clsx(
                                    'px-4 py-1.5 rounded-lg text-sm font-semibold transition-all',
                                    activeTab === 'alerts' ? 'bg-primary-600 text-white shadow-glow' : 'text-slate-400 hover:text-slate-200'
                                )}
                            >
                                Security Alerts
                            </button>
                            <button 
                                onClick={() => setActiveTab('ip_blocks')}
                                className={clsx(
                                    'px-4 py-1.5 rounded-lg text-sm font-semibold transition-all',
                                    activeTab === 'ip_blocks' ? 'bg-primary-600 text-white shadow-glow' : 'text-slate-400 hover:text-slate-200'
                                )}
                            >
                                IP Blacklist
                            </button>
                            <button 
                                onClick={() => setActiveTab('audit')}
                                className={clsx(
                                    'px-4 py-1.5 rounded-lg text-sm font-semibold transition-all',
                                    activeTab === 'audit' ? 'bg-primary-600 text-white shadow-glow' : 'text-slate-400 hover:text-slate-200'
                                )}
                            >
                                System Audit Log
                            </button>
                        </nav>
                        <div className="relative">
                            <Search className="absolute left-3 top-1/2 -translate-y-1/2 text-slate-500" size={16} />
                            <input 
                                type="text"
                                placeholder="Search logs..."
                                className="pl-10 pr-4 py-2 bg-[#0b0f19] border border-white/5 rounded-xl text-sm w-full md:w-64 focus:outline-none focus:border-primary-500 transition-all font-medium text-slate-200"
                                value={searchQuery}
                                onChange={(e) => setSearchQuery(e.target.value)}
                            />
                        </div>
                    </div>
                </div>

                <div className="p-0">
                    <div className="overflow-x-auto">
                        <table className="w-full text-left text-sm whitespace-nowrap">
                            <thead>
                                <tr className="border-b border-white/5 text-slate-400 font-bold bg-[#1a2336]/20">
                                    <th className="px-6 py-3">Severity</th>
                                    <th className="px-6 py-3">Event Type</th>
                                    <th className="px-6 py-3">Source</th>
                                    <th className="px-6 py-3">Impacted User</th>
                                    <th className="px-6 py-3">Timestamp</th>
                                    <th className="px-6 py-3 text-right">Actions</th>
                                </tr>
                            </thead>
                            <tbody className="divide-y divide-white/5">
                                {/* Example Alert 1 */}
                                <tr className="hover:bg-white/5 transition-colors">
                                    <td className="px-6 py-4">
                                        <span className="inline-flex items-center gap-1.5 px-3 py-1 rounded-full bg-rose-500/10 text-rose-400 font-bold text-xs border border-rose-500/20 uppercase tracking-wider">
                                            Critical
                                        </span>
                                    </td>
                                    <td className="px-6 py-4">
                                        <div className="flex flex-col">
                                            <span className="font-bold text-slate-200">Multiple Failed Logins</span>
                                            <span className="text-xs text-slate-500">Possible brute-force attack detected</span>
                                        </div>
                                    </td>
                                    <td className="px-6 py-4">
                                        <code className="text-xs bg-[#0b0f19] px-2 py-1 border border-white/5 text-slate-300 rounded font-mono">185.122.45.90</code>
                                    </td>
                                    <td className="px-6 py-4 font-semibold italic text-slate-300">merchant_user_882</td>
                                    <td className="px-6 py-4 text-slate-400 font-semibold">2026-03-31 15:12:45</td>
                                    <td className="px-6 py-4 text-right">
                                        <button className="text-primary-400 font-bold hover:text-primary-100 transition-colors">Investigate</button>
                                    </td>
                                </tr>
                                {/* Example Alert 2 */}
                                <tr className="hover:bg-white/5 transition-colors">
                                    <td className="px-6 py-4">
                                        <span className="inline-flex items-center gap-1.5 px-3 py-1 rounded-full bg-amber-500/10 text-amber-400 font-bold text-xs border border-amber-500/20 uppercase tracking-wider">
                                            High
                                        </span>
                                    </td>
                                    <td className="px-6 py-4">
                                        <div className="flex flex-col">
                                            <span className="font-bold text-slate-200">Large Withdrawal Request</span>
                                            <span className="text-xs text-slate-500">Pending manual approval for $50k+</span>
                                        </div>
                                    </td>
                                    <td className="px-6 py-4">
                                        <span className="text-xs bg-[#0b0f19] px-2 py-1 border border-white/5 text-slate-400 rounded font-medium">Internal System</span>
                                    </td>
                                    <td className="px-6 py-4 font-semibold italic text-slate-300">Global Holdings LLC</td>
                                    <td className="px-6 py-4 text-slate-400 font-semibold">2026-03-31 14:55:20</td>
                                    <td className="px-6 py-4 text-right">
                                        <button className="text-primary-400 font-bold hover:text-primary-100 transition-colors">Review</button>
                                    </td>
                                </tr>
                            </tbody>
                        </table>
                    </div>
                </div>
            </div>
        </div>
    );
};

export default SecurityPage;
