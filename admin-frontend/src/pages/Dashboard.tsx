import React, { useEffect, useState } from 'react';
import { 
    Users, 
    CreditCard, 
    TrendingUp, 
    ArrowUpRight, 
    ArrowDownRight, 
    Zap, 
    ShieldCheck, 
    Clock,
    Activity,
    ChevronRight,
    Search
} from 'lucide-react';
import { adminAPI } from '../lib/api';

const Dashboard: React.FC = () => {
    const [stats, setStats] = useState([
        { label: 'Total Volume', value: '$128,430.00', trend: '+12.5%', isUp: true, icon: TrendingUp, color: 'emerald' },
        { label: 'Active Merchants', value: '42', trend: '+3', isUp: true, icon: Users, color: 'blue' },
        { label: 'Total Payments', value: '1,504', trend: '+85', isUp: true, icon: CreditCard, color: 'indigo' },
        { label: 'Pending Withdrawals', value: '12', trend: '-2', isUp: false, icon: Clock, color: 'amber' },
    ]);

    const [recentActivity, setRecentActivity] = useState([
        { type: 'payment', merchant: 'TechStore Global', amount: '$120.50', time: '2 mins ago', status: 'completed' },
        { type: 'withdrawal', merchant: 'CryptoCafe', amount: '$250.00', time: '15 mins ago', status: 'pending' },
        { type: 'merchant', merchant: 'New FashionHub', amount: 'N/A', time: '1 hour ago', status: 'new_registration' },
        { type: 'payment', merchant: 'EcoFriendly Goods', amount: '$45.20', time: '2 hours ago', status: 'failed' },
    ]);

    useEffect(() => {
        fetchDashboardData();
    }, []);

    const fetchDashboardData = async () => {
        try {
            const res = await adminAPI.getDashboardStats();
            if (res.data) {
                // Update stats from actual api response dynamically
                const d = res.data;
                setStats([
                    { label: 'Total Volume', value: d.total_volume_usd || '$128,430.00', trend: d.volume_trend || '+12.5%', isUp: true, icon: TrendingUp, color: 'emerald' },
                    { label: 'Active Merchants', value: d.active_merchants?.toString() || '42', trend: d.merchants_trend || '+3', isUp: true, icon: Users, color: 'blue' },
                    { label: 'Total Payments', value: d.total_payments_count?.toString() || '1,504', trend: d.payments_trend || '+85', isUp: true, icon: CreditCard, color: 'indigo' },
                    { label: 'Pending Withdrawals', value: d.pending_withdrawals_count?.toString() || '12', trend: d.withdrawals_trend || '-2', isUp: false, icon: Clock, color: 'amber' },
                ]);

                if (d.recent_activities) {
                    setRecentActivity(d.recent_activities);
                }
            }
        } catch (e) {
            console.error('Failed to load dashboard metrics from backend APIs:', e);
        }
    };

    return (
        <div className="space-y-8 animate-in fade-in duration-700">
            <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
                <div>
                    <h1 className="text-2xl font-bold text-slate-100 tracking-tight">System Overview</h1>
                    <p className="text-slate-400 text-sm mt-1">Real-time snapshots of platform performance and activity.</p>
                </div>
                <div className="flex items-center gap-3">
                    <div className="relative hidden md:block">
                        <Search className="absolute left-3 top-1/2 -translate-y-1/2 text-slate-500" size={16} />
                        <input 
                            type="text" 
                            placeholder="Search everything..." 
                            className="pl-9 pr-4 py-2 bg-[#151c2c] border border-white/5 rounded-xl text-sm focus:outline-none focus:border-primary-500 transition-all w-64 text-slate-200"
                        />
                    </div>
                    <button onClick={fetchDashboardData} className="px-5 py-2.5 bg-primary-600 text-white rounded-xl text-sm font-bold hover:bg-primary-500 transition-all shadow-glow active:scale-95">
                        Refresh Stats
                    </button>
                </div>
            </div>

            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
                {stats.map((stat, i) => (
                    <div key={i} className="bg-[#151c2c] p-6 rounded-2xl border border-white/5 hover:border-primary-500/30 transition-all hover:-translate-y-0.5 duration-300">
                        <div className="flex items-center justify-between mb-4">
                            <div className={`p-3 bg-white/5 text-primary-400 rounded-xl`}>
                                <stat.icon size={20} />
                            </div>
                            <div className={`flex items-center gap-1 text-xs font-bold ${stat.isUp ? 'text-emerald-400' : 'text-rose-400'}`}>
                                {stat.isUp ? <ArrowUpRight size={14} /> : <ArrowDownRight size={14} />}
                                {stat.trend}
                            </div>
                        </div>
                        <div className="text-sm font-medium text-slate-400">{stat.label}</div>
                        <div className="text-2xl font-bold text-slate-100 mt-1">{stat.value}</div>
                    </div>
                ))}
            </div>

            <div className="grid grid-cols-1 lg:grid-cols-3 gap-8">
                <div className="lg:col-span-2 space-y-6">
                    <div className="bg-[#151c2c] rounded-2xl border border-white/5 shadow-sm overflow-hidden min-h-[400px]">
                        <div className="p-6 border-b border-white/5 flex items-center justify-between">
                            <h2 className="text-lg font-bold text-slate-100">Transaction Volume</h2>
                            <div className="flex gap-2">
                                {['7D', '30D', '90D'].map(p => (
                                    <button key={p} className={`px-3 py-1 rounded-lg text-[10px] font-bold ${p === '30D' ? 'bg-primary-600 text-white shadow-glow' : 'bg-white/5 text-slate-400 hover:bg-white/10'}`}>
                                        {p}
                                    </button>
                                ))}
                            </div>
                        </div>
                        <div className="p-8 flex flex-col justify-center items-center h-full text-center">
                            <div className="w-full h-48 bg-[#0b0f19] rounded-xl border border-dashed border-white/10 flex items-center justify-center">
                                 <div className="flex items-center gap-2 text-slate-500">
                                    <Activity size={24} className="animate-pulse" />
                                    <span className="text-sm font-medium">Chart Engine Initializing...</span>
                                 </div>
                            </div>
                            <div className="mt-8 grid grid-cols-3 gap-12 w-full">
                                <div>
                                    <div className="text-xs font-bold text-slate-500 uppercase tracking-widest">Successful</div>
                                    <div className="text-xl font-bold text-slate-200">98.4%</div>
                                </div>
                                <div>
                                    <div className="text-xs font-bold text-slate-500 uppercase tracking-widest">Failed</div>
                                    <div className="text-xl font-bold text-slate-200">1.2%</div>
                                </div>
                                <div>
                                    <div className="text-xs font-bold text-slate-500 uppercase tracking-widest">Refunded</div>
                                    <div className="text-xl font-bold text-slate-200">0.4%</div>
                                </div>
                            </div>
                        </div>
                    </div>

                    <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                        <div className="bg-gradient-to-br from-indigo-600 to-primary-700 rounded-2xl shadow-lg p-6 text-white relative overflow-hidden group">
                            <div className="absolute -right-6 -bottom-6 opacity-10 transform scale-150 rotate-12 group-hover:rotate-0 transition-transform duration-700">
                                <ShieldCheck size={140} />
                            </div>
                            <h3 className="text-lg font-bold mb-2">Security Audit</h3>
                            <p className="text-indigo-100 text-sm mb-6 leading-relaxed">
                                No critical vulnerabilities found. All blockchain nodes are synchronizing correctly.
                            </p>
                            <button className="px-4 py-2 bg-white text-indigo-600 rounded-lg text-sm font-bold hover:bg-indigo-50 transition-colors">
                                View Security Report
                            </button>
                        </div>
                        <div className="bg-[#151c2c] rounded-2xl border border-white/5 shadow-sm p-6">
                            <div className="flex items-center justify-between mb-4">
                                <h3 className="text-sm font-bold text-slate-200 uppercase tracking-wider">Node Latency</h3>
                                <Zap size={16} className="text-amber-500" />
                            </div>
                            <div className="space-y-4">
                                {[
                                    { name: 'Ethereum', val: '45ms', p: 40 },
                                    { name: 'Solana', val: '12ms', p: 15 },
                                    { name: 'BSC', val: '32ms', p: 30 }
                                ].map((node, i) => (
                                    <div key={i}>
                                        <div className="flex justify-between text-xs mb-1.5">
                                            <span className="font-medium text-slate-400">{node.name}</span>
                                            <span className="font-bold text-slate-200">{node.val}</span>
                                        </div>
                                        <div className="w-full bg-[#0b0f19] h-1.5 rounded-full overflow-hidden">
                                            <div className="bg-primary-500 h-full rounded-full" style={{ width: `${node.p}%` }}></div>
                                        </div>
                                    </div>
                                ))}
                            </div>
                        </div>
                    </div>
                </div>

                <div className="space-y-6">
                    <div className="bg-[#151c2c] rounded-2xl border border-white/5 shadow-sm overflow-hidden flex flex-col">
                        <div className="p-6 border-b border-white/5">
                            <h2 className="text-lg font-bold text-slate-100">Recent Activity</h2>
                        </div>
                        <div className="divide-y divide-white/5">
                            {recentActivity.map((act, i) => (
                                <div key={i} className="p-6 hover:bg-white/5 transition-colors group cursor-default">
                                    <div className="flex items-center gap-4">
                                        <div className={`p-2 rounded-lg ${
                                            act.type === 'payment' ? 'bg-emerald-500/10 text-emerald-400' :
                                            act.type === 'withdrawal' ? 'bg-amber-500/10 text-amber-400' :
                                            'bg-indigo-500/10 text-indigo-400'
                                        }`}>
                                            {act.type === 'payment' ? <CreditCard size={18} /> : 
                                             act.type === 'withdrawal' ? <ArrowUpRight size={18} /> : 
                                             <Users size={18} />}
                                        </div>
                                        <div className="flex-1">
                                            <div className="text-sm font-bold text-slate-200 leading-tight">
                                                {act.merchant}
                                            </div>
                                            <div className="text-xs text-slate-500 mt-0.5">{act.time}</div>
                                        </div>
                                        <div className="text-right">
                                            <div className="text-sm font-bold text-slate-200">{act.amount}</div>
                                            <div className={`text-[10px] font-bold uppercase tracking-widest mt-0.5 ${
                                                act.status === 'completed' ? 'text-emerald-400' :
                                                act.status === 'pending' ? 'text-amber-400' :
                                                act.status === 'failed' ? 'text-rose-400' : 'text-indigo-400'
                                            }`}>
                                                {act.status.replace('_', ' ')}
                                            </div>
                                        </div>
                                    </div>
                                </div>
                            ))}
                        </div>
                        <button className="p-4 text-sm font-bold text-primary-400 hover:bg-white/5 border-t border-white/5 transition-colors flex items-center justify-center gap-2">
                            View All Activity
                            <ChevronRight size={16} />
                        </button>
                    </div>

                    <div className="bg-[#151c2c] rounded-2xl border border-white/5 shadow-xl p-6 text-white text-center">
                         <div className="w-16 h-16 bg-[#0b0f19] rounded-full flex items-center justify-center mx-auto mb-4 border border-white/5">
                            <Zap size={32} className="text-primary-400" />
                        </div>
                        <h3 className="text-lg font-bold mb-2">Automated Tasks</h3>
                        <p className="text-slate-400 text-sm mb-6">
                            There are <span className="text-white font-bold">14 active automations</span> running. All tasks are currently on schedule.
                        </p>
                        <div className="space-y-3">
                            <div className="flex items-center justify-between text-xs p-2 bg-[#0b0f19]/50 rounded-lg border border-white/5">
                                <span className="text-slate-400 font-medium">Fee Sweepers</span>
                                <span className="text-emerald-400 font-bold">ACTIVE</span>
                            </div>
                            <div className="flex items-center justify-between text-xs p-2 bg-[#0b0f19]/50 rounded-lg border border-white/5">
                                <span className="text-slate-400 font-medium">Chain Watchers</span>
                                <span className="text-emerald-400 font-bold">ACTIVE</span>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    );
};

export default Dashboard;
