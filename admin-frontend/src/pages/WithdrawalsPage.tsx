import React, { useState } from 'react';
import { 
    Search, 
    Filter, 
    MoreHorizontal, 
    CheckCircle2, 
    XCircle, 
    Clock, 
    AlertTriangle,
    Download
} from 'lucide-react';

interface WithdrawalRequest {
    id: string;
    merchant: string;
    amount: string;
    crypto: string;
    status: 'pending' | 'approved' | 'rejected' | 'failed' | 'processing';
    timestamp: string;
    destination: string;
}

const WithdrawalsPage: React.FC = () => {
    const [searchQuery, setSearchQuery] = useState('');

    const withdrawals: WithdrawalRequest[] = [
        { id: 'wd_4k2v9k', merchant: 'TechStore Global', amount: '$1,500.00', crypto: '0.45 ETH', status: 'pending', timestamp: '2024-03-27 12:45', destination: '0x123...456' },
        { id: 'wd_1m8n5p', merchant: 'CryptoCafe', amount: '$250.00', crypto: '5.25 SOL', status: 'approved', timestamp: '2024-03-27 11:12', destination: 'ABC...XYZ' },
        { id: 'wd_6l4q1r', merchant: 'FashionHub', amount: '$540.00', crypto: '540.00 USDC', status: 'failed', timestamp: '2024-03-26 15:05', destination: '0x3b1...c4d' },
    ];

    const getStatusStyles = (status: string) => {
        switch (status) {
            case 'approved': return 'bg-emerald-50 text-emerald-600 border-emerald-100';
            case 'pending': return 'bg-amber-50 text-amber-600 border-amber-100';
            case 'processing': return 'bg-blue-50 text-blue-600 border-blue-100';
            case 'rejected': return 'bg-rose-50 text-rose-600 border-rose-100';
            case 'failed': return 'bg-rose-50 text-rose-600 border-rose-100';
            default: return 'bg-slate-50 text-slate-600 border-slate-100';
        }
    };

    const getStatusIcon = (status: string) => {
        switch (status) {
            case 'approved': return <CheckCircle2 size={14} />;
            case 'pending': return <Clock size={14} />;
            case 'processing': return <Clock size={14} className="animate-spin" />;
            case 'rejected': return <XCircle size={14} />;
            case 'failed': return <AlertTriangle size={14} />;
            default: return null;
        }
    };

    return (
        <div className="space-y-6 animate-in fade-in duration-500">
            <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
                <div>
                    <h1 className="text-2xl font-bold text-slate-900 tracking-tight">Withdrawals</h1>
                    <p className="text-slate-500 text-sm mt-1">Review and approve merchant withdrawal requests.</p>
                </div>
                <div className="flex items-center gap-3">
                    <button className="flex items-center gap-2 px-4 py-2 bg-white border border-slate-200 rounded-lg text-sm font-medium text-slate-700 hover:bg-slate-50 transition-colors shadow-sm">
                        <Download size={16} />
                        Export
                    </button>
                    <button className="flex items-center gap-2 px-4 py-2 bg-white border border-slate-200 rounded-lg text-sm font-medium text-slate-700 hover:bg-slate-50 transition-colors shadow-sm">
                        <Filter size={16} />
                        Filters
                    </button>
                    <button className="flex items-center gap-2 px-4 py-2 bg-primary-600 rounded-lg text-sm font-medium text-white hover:bg-primary-700 transition-colors shadow-sm">
                        Process Batch
                    </button>
                </div>
            </div>

            <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
                {[
                    { label: 'Pending Approval', value: '12', color: 'amber' },
                    { label: 'Processing', value: '5', color: 'blue' },
                    { label: 'Avg Process Time', value: '4h 12m', color: 'emerald' }
                ].map((stat, i) => (
                    <div key={i} className="bg-white p-6 rounded-xl border border-slate-200 shadow-sm">
                        <div className="text-sm font-medium text-slate-500">{stat.label}</div>
                        <div className={`text-2xl font-bold text-${stat.color}-600 mt-1`}>{stat.value}</div>
                    </div>
                ))}
            </div>

            <div className="bg-white rounded-xl border border-slate-200 shadow-sm overflow-hidden">
                <div className="p-4 border-b border-slate-100 bg-slate-50/50 flex flex-wrap gap-4">
                    <div className="relative flex-1 min-w-[300px]">
                        <Search className="absolute left-3 top-1/2 -translate-y-1/2 text-slate-400" size={18} />
                        <input 
                            type="text" 
                            placeholder="Search by ID, Merchant or Destination..." 
                            className="w-full pl-10 pr-4 py-2 bg-white border border-slate-200 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-primary-500/20 focus:border-primary-500 transition-all"
                            value={searchQuery}
                            onChange={(e) => setSearchQuery(e.target.value)}
                        />
                    </div>
                </div>

                <div className="overflow-x-auto">
                    <table className="w-full text-left border-collapse">
                        <thead>
                            <tr className="bg-slate-50/50 text-slate-500 text-xs font-bold uppercase tracking-wider">
                                <th className="px-6 py-4">ID</th>
                                <th className="px-6 py-4">Merchant</th>
                                <th className="px-6 py-4 text-right">Amount</th>
                                <th className="px-6 py-4 text-center">Status</th>
                                <th className="px-6 py-4">Requested At</th>
                                <th className="px-6 py-4 text-right">Actions</th>
                            </tr>
                        </thead>
                        <tbody className="divide-y divide-slate-100">
                            {withdrawals.map((wd) => (
                                <tr key={wd.id} className="hover:bg-slate-50/50 transition-colors">
                                    <td className="px-6 py-4 font-mono text-xs text-slate-500">{wd.id}</td>
                                    <td className="px-6 py-4">
                                        <div className="text-sm font-medium text-slate-900">{wd.merchant}</div>
                                        <div className="text-[10px] font-mono text-slate-400 truncate max-w-[120px]">{wd.destination}</div>
                                    </td>
                                    <td className="px-6 py-4 text-right">
                                        <div className="text-sm font-bold text-slate-900">{wd.amount}</div>
                                        <div className="text-xs text-slate-500">{wd.crypto}</div>
                                    </td>
                                    <td className="px-6 py-4 text-center">
                                        <span className={`inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full text-xs font-bold border capitalize ${getStatusStyles(wd.status)}`}>
                                            {getStatusIcon(wd.status)}
                                            {wd.status}
                                        </span>
                                    </td>
                                    <td className="px-6 py-4 text-xs text-slate-500">{wd.timestamp}</td>
                                    <td className="px-6 py-4 text-right">
                                        <div className="flex items-center justify-end gap-2 text-right">
                                            <button className="px-3 py-1 bg-emerald-600 text-white text-xs font-bold rounded-lg hover:bg-emerald-700 transition-colors">Approve</button>
                                            <button className="p-2 text-slate-400 hover:text-slate-600 hover:bg-slate-100 rounded-lg transition-all">
                                                <MoreHorizontal size={16} />
                                            </button>
                                        </div>
                                    </td>
                                </tr>
                            ))}
                        </tbody>
                    </table>
                </div>
            </div>
        </div>
    );
};

export default WithdrawalsPage;
