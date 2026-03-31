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
        { label: 'Total Alerts', value: '0', icon: ShieldAlert, color: 'text-red-600', bg: 'bg-red-50' },
        { label: 'Blocked IPs', value: '2', icon: Ban, iconColor: 'text-orange-600', bg: 'bg-orange-50' },
        { label: 'Security Score', value: '98/100', icon: ShieldCheck, color: 'text-green-600', bg: 'bg-green-50' },
        { label: 'Active Sessions', value: '1', icon: Activity, color: 'text-blue-600', bg: 'bg-blue-50' },
    ];

    return (
        <div className="space-y-6">
            <header className="flex justify-between items-end">
                <div>
                    <h2 className="text-2xl font-bold tracking-tight">Security Command Center</h2>
                    <p className="text-muted-foreground">Monitor system-wide security events and manage global threat protections.</p>
                </div>
                <div className="flex gap-2">
                    <button className="flex items-center gap-2 px-4 py-2 border rounded-md hover:bg-slate-50 transition-colors text-sm font-medium">
                        <RefreshCcw size={16} /> Refresh
                    </button>
                    <button className="flex items-center gap-2 px-4 py-2 bg-primary-600 text-white rounded-md hover:bg-primary-700 transition-colors text-sm font-medium">
                        <Lock size={16} /> Global Lockdown
                    </button>
                </div>
            </header>

            {/* Stats Grid */}
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
                {stats.map((stat) => (
                    <div key={stat.label} className="bg-white p-6 rounded-xl border border-slate-200 shadow-sm">
                        <div className="flex items-center justify-between mb-4">
                            <div className={clsx('p-2.5 rounded-lg', stat.bg)}>
                                <stat.icon className={clsx('w-5 h-5', stat.color || stat.iconColor)} />
                            </div>
                        </div>
                        <div className="space-y-1">
                            <p className="text-sm font-medium text-slate-500">{stat.label}</p>
                            <p className="text-2xl font-bold">{stat.value}</p>
                        </div>
                    </div>
                ))}
            </div>

            {/* Main Content */}
            <div className="bg-white rounded-xl border border-slate-200 shadow-sm overflow-hidden">
                <div className="border-b border-slate-200 bg-slate-50/50">
                    <div className="flex flex-col md:flex-row md:items-center justify-between px-6 py-4 gap-4">
                        <nav className="flex gap-1 p-1 bg-slate-200/50 rounded-lg w-fit">
                            <button 
                                onClick={() => setActiveTab('alerts')}
                                className={clsx(
                                    'px-4 py-1.5 rounded-md text-sm font-medium transition-all',
                                    activeTab === 'alerts' ? 'bg-white shadow-sm text-slate-900' : 'text-slate-500 hover:text-slate-700'
                                )}
                            >
                                Security Alerts
                            </button>
                            <button 
                                onClick={() => setActiveTab('ip_blocks')}
                                className={clsx(
                                    'px-4 py-1.5 rounded-md text-sm font-medium transition-all',
                                    activeTab === 'ip_blocks' ? 'bg-white shadow-sm text-slate-900' : 'text-slate-500 hover:text-slate-700'
                                )}
                            >
                                IP Blacklist
                            </button>
                            <button 
                                onClick={() => setActiveTab('audit')}
                                className={clsx(
                                    'px-4 py-1.5 rounded-md text-sm font-medium transition-all',
                                    activeTab === 'audit' ? 'bg-white shadow-sm text-slate-900' : 'text-slate-500 hover:text-slate-700'
                                )}
                            >
                                System Audit Log
                            </button>
                        </nav>
                        <div className="relative">
                            <Search className="absolute left-3 top-1/2 -translate-y-1/2 text-slate-400" size={16} />
                            <input 
                                type="text"
                                placeholder="Search logs..."
                                className="pl-10 pr-4 py-2 bg-white border border-slate-200 rounded-lg text-sm w-full md:w-64 focus:outline-none focus:ring-2 focus:ring-primary-500/20 transition-all font-medium"
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
                                <tr className="border-b border-slate-100 text-slate-500 font-medium bg-slate-50/30">
                                    <th className="px-6 py-3">Severity</th>
                                    <th className="px-6 py-3">Event Type</th>
                                    <th className="px-6 py-3">Source</th>
                                    <th className="px-6 py-3">Impacted User</th>
                                    <th className="px-6 py-3">Timestamp</th>
                                    <th className="px-6 py-3 text-right">Actions</th>
                                </tr>
                            </thead>
                            <tbody className="divide-y divide-slate-100">
                                {/* Example Alert 1 */}
                                <tr className="hover:bg-slate-50/50 transition-colors">
                                    <td className="px-6 py-4">
                                        <span className="inline-flex items-center gap-1.5 px-2 py-0.5 rounded-full bg-red-100 text-red-700 font-semibold text-xs border border-red-200 uppercase tracking-wider">
                                            Critical
                                        </span>
                                    </td>
                                    <td className="px-6 py-4">
                                        <div className="flex flex-col">
                                            <span className="font-bold text-slate-900">Multiple Failed Logins</span>
                                            <span className="text-xs text-slate-500">Possible brute-force attack detected</span>
                                        </div>
                                    </td>
                                    <td className="px-6 py-4">
                                        <code className="text-xs bg-slate-100 px-1.5 py-0.5 rounded font-mono">185.122.45.90</code>
                                    </td>
                                    <td className="px-6 py-4 font-medium italic overflow-hidden text-ellipsis whitespace-nowrap">merchant_user_882</td>
                                    <td className="px-6 py-4 text-slate-500">2026-03-31 15:12:45</td>
                                    <td className="px-6 py-4 text-right">
                                        <button className="text-primary-600 font-bold hover:underline">Investigate</button>
                                    </td>
                                </tr>
                                {/* Example Alert 2 */}
                                <tr className="hover:bg-slate-50/50 transition-colors">
                                    <td className="px-6 py-4">
                                        <span className="inline-flex items-center gap-1.5 px-2 py-0.5 rounded-full bg-orange-100 text-orange-700 font-semibold text-xs border border-orange-200 uppercase tracking-wider">
                                            High
                                        </span>
                                    </td>
                                    <td className="px-6 py-4">
                                        <div className="flex flex-col">
                                            <span className="font-bold text-slate-900">Large Withdrawal Request</span>
                                            <span className="text-xs text-slate-500">Pending manual approval for $50k+</span>
                                        </div>
                                    </td>
                                    <td className="px-6 py-4">
                                        <span className="text-xs bg-slate-100 px-1.5 py-0.5 rounded font-medium">Internal System</span>
                                    </td>
                                    <td className="px-6 py-4 font-medium italic overflow-hidden text-ellipsis whitespace-nowrap">Global Holdings LLC</td>
                                    <td className="px-6 py-4 text-slate-500">2026-03-31 14:55:20</td>
                                    <td className="px-6 py-4 text-right">
                                        <button className="text-primary-600 font-bold hover:underline">Review</button>
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
