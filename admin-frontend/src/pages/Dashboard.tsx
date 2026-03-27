import React from 'react';
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

const Dashboard: React.FC = () => {
    const stats = [
        { label: 'Total Volume', value: '$128,430.00', trend: '+12.5%', isUp: true, icon: TrendingUp, color: 'emerald' },
        { label: 'Active Merchants', value: '42', trend: '+3', isUp: true, icon: Users, color: 'blue' },
        { label: 'Total Payments', value: '1,504', trend: '+85', isUp: true, icon: CreditCard, color: 'indigo' },
        { label: 'Pending Withdrawals', value: '12', trend: '-2', isUp: false, icon: Clock, color: 'amber' },
    ];

    const recentActivity = [
        { type: 'payment', merchant: 'TechStore Global', amount: '$120.50', time: '2 mins ago', status: 'completed' },
        { type: 'withdrawal', merchant: 'CryptoCafe', amount: '$250.00', time: '15 mins ago', status: 'pending' },
        { type: 'merchant', merchant: 'New FashionHub', amount: 'N/A', time: '1 hour ago', status: 'new_registration' },
        { type: 'payment', merchant: 'EcoFriendly Goods', amount: '$45.20', time: '2 hours ago', status: 'failed' },
    ];

    return (
        <div className="space-y-8 animate-in fade-in duration-700">
            <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
                <div>
                    <h1 className="text-2xl font-bold text-slate-900 tracking-tight">System Overview</h1>
                    <p className="text-slate-500 text-sm mt-1">Real-time snapshots of platform performance and activity.</p>
                </div>
                <div className="flex items-center gap-3">
                    <div className="relative hidden md:block">
                        <Search className="absolute left-3 top-1/2 -translate-y-1/2 text-slate-400" size={16} />
                        <input 
                            type="text" 
                            placeholder="Search everything..." 
                            className="pl-9 pr-4 py-2 bg-slate-100 border-transparent rounded-lg text-sm focus:bg-white focus:ring-2 focus:ring-primary-500/20 focus:border-primary-500 transition-all w-64"
                        />
                    </div>
                    <button className="px-4 py-2 bg-slate-900 text-white rounded-lg text-sm font-bold hover:bg-slate-800 transition-colors shadow-sm">
                        Generate Report
                    </button>
                </div>
            </div>

            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
                {stats.map((stat, i) => (
                    <div key={i} className="bg-white p-6 rounded-2xl border border-slate-200 shadow-sm hover:shadow-md transition-shadow group">
                        <div className="flex items-center justify-between mb-4">
                            <div className={`p-3 bg-${stat.color}-50 text-${stat.color}-600 rounded-xl group-hover:scale-110 transition-transform`}>
                                <stat.icon size={20} />
                            </div>
                            <div className={`flex items-center gap-1 text-xs font-bold ${stat.isUp ? 'text-emerald-600' : 'text-rose-600'}`}>
                                {stat.isUp ? <ArrowUpRight size={14} /> : <ArrowDownRight size={14} />}
                                {stat.trend}
                            </div>
                        </div>
                        <div className="text-sm font-medium text-slate-500">{stat.label}</div>
                        <div className="text-2xl font-bold text-slate-900 mt-1">{stat.value}</div>
                    </div>
                ))}
            </div>

            <div className="grid grid-cols-1 lg:grid-cols-3 gap-8">
                <div className="lg:col-span-2 space-y-6">
                    <div className="bg-white rounded-2xl border border-slate-200 shadow-sm overflow-hidden min-h-[400px]">
                        <div className="p-6 border-b border-slate-100 flex items-center justify-between">
                            <h2 className="text-lg font-bold text-slate-900">Transaction Volume</h2>
                            <div className="flex gap-2">
                                {['7D', '30D', '90D'].map(p => (
                                    <button key={p} className={`px-3 py-1 rounded-md text-[10px] font-bold ${p === '30D' ? 'bg-slate-900 text-white' : 'bg-slate-100 text-slate-500 hover:bg-slate-200'}`}>
                                        {p}
                                    </button>
                                ))}
                            </div>
                        </div>
                        <div className="p-8 flex flex-col justify-center items-center h-full text-center">
                            <div className="w-full h-48 bg-slate-50 rounded-xl border border-dashed border-slate-200 flex items-center justify-center">
                                 <div className="flex items-center gap-2 text-slate-400">
                                    <Activity size={24} className="animate-pulse" />
                                    <span className="text-sm font-medium">Chart Engine Initializing...</span>
                                 </div>
                            </div>
                            <div className="mt-8 grid grid-cols-3 gap-12 w-full">
                                <div>
                                    <div className="text-xs font-bold text-slate-400 uppercase tracking-widest">Successful</div>
                                    <div className="text-xl font-bold text-slate-900">98.4%</div>
                                </div>
                                <div>
                                    <div className="text-xs font-bold text-slate-400 uppercase tracking-widest">Failed</div>
                                    <div className="text-xl font-bold text-slate-900">1.2%</div>
                                </div>
                                <div>
                                    <div className="text-xs font-bold text-slate-400 uppercase tracking-widest">Refunded</div>
                                    <div className="text-xl font-bold text-slate-900">0.4%</div>
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
                        <div className="bg-white rounded-2xl border border-slate-200 shadow-sm p-6">
                            <div className="flex items-center justify-between mb-4">
                                <h3 className="text-sm font-bold text-slate-900 uppercase tracking-wider">Node Latency</h3>
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
                                            <span className="font-medium text-slate-600">{node.name}</span>
                                            <span className="font-bold text-slate-900">{node.val}</span>
                                        </div>
                                        <div className="w-full bg-slate-100 h-1.5 rounded-full overflow-hidden">
                                            <div className="bg-primary-500 h-full rounded-full" style={{ width: `${node.p}%` }}></div>
                                        </div>
                                    </div>
                                ))}
                            </div>
                        </div>
                    </div>
                </div>

                <div className="space-y-6">
                    <div className="bg-white rounded-2xl border border-slate-200 shadow-sm overflow-hidden flex flex-col">
                        <div className="p-6 border-b border-slate-100">
                            <h2 className="text-lg font-bold text-slate-900">Recent Activity</h2>
                        </div>
                        <div className="divide-y divide-slate-50">
                            {recentActivity.map((act, i) => (
                                <div key={i} className="p-6 hover:bg-slate-50 transition-colors group cursor-default">
                                    <div className="flex items-center gap-4">
                                        <div className={`p-2 rounded-lg ${
                                            act.type === 'payment' ? 'bg-emerald-50 text-emerald-600' :
                                            act.type === 'withdrawal' ? 'bg-amber-50 text-amber-600' :
                                            'bg-indigo-50 text-indigo-600'
                                        }`}>
                                            {act.type === 'payment' ? <CreditCard size={18} /> : 
                                             act.type === 'withdrawal' ? <ArrowUpRight size={18} /> : 
                                             <Users size={18} />}
                                        </div>
                                        <div className="flex-1">
                                            <div className="text-sm font-bold text-slate-900 leading-tight">
                                                {act.merchant}
                                            </div>
                                            <div className="text-xs text-slate-500 mt-0.5">{act.time}</div>
                                        </div>
                                        <div className="text-right">
                                            <div className="text-sm font-bold text-slate-900">{act.amount}</div>
                                            <div className={`text-[10px] font-bold uppercase tracking-widest mt-0.5 ${
                                                act.status === 'completed' ? 'text-emerald-500' :
                                                act.status === 'pending' ? 'text-amber-500' :
                                                act.status === 'failed' ? 'text-rose-500' : 'text-indigo-500'
                                            }`}>
                                                {act.status.replace('_', ' ')}
                                            </div>
                                        </div>
                                    </div>
                                </div>
                            ))}
                        </div>
                        <button className="p-4 text-sm font-bold text-primary-600 hover:bg-primary-50 border-t border-slate-100 transition-colors flex items-center justify-center gap-2">
                            View All Activity
                            <ChevronRight size={16} />
                        </button>
                    </div>

                    <div className="bg-slate-900 rounded-2xl shadow-xl p-6 text-white text-center">
                         <div className="w-16 h-16 bg-slate-800 rounded-full flex items-center justify-center mx-auto mb-4">
                            <Zap size={32} className="text-primary-400" />
                        </div>
                        <h3 className="text-lg font-bold mb-2">Automated Tasks</h3>
                        <p className="text-slate-400 text-sm mb-6">
                            There are <span className="text-white font-bold">14 active automations</span> running. All tasks are currently on schedule.
                        </p>
                        <div className="space-y-3">
                            <div className="flex items-center justify-between text-xs p-2 bg-slate-800/50 rounded-lg">
                                <span className="text-slate-400 font-medium">Fee Sweepers</span>
                                <span className="text-emerald-400 font-bold">ACTIVE</span>
                            </div>
                            <div className="flex items-center justify-between text-xs p-2 bg-slate-800/50 rounded-lg">
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
