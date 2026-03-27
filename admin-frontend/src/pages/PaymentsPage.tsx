import React, { useState } from 'react';
import { 
    Search, 
    Filter, 
    MoreHorizontal, 
    CheckCircle2, 
    XCircle, 
    Clock, 
    RotateCcw,
    Zap,
    Download
} from 'lucide-react';

interface Transaction {
    id: string;
    merchant: string;
    amount: string;
    crypto: string;
    status: 'completed' | 'pending' | 'failed' | 'processing';
    timestamp: string;
    hash: string;
}

const PaymentsPage: React.FC = () => {
    const [searchQuery, setSearchQuery] = useState('');

    const transactions: Transaction[] = [
        { id: 'pay_7x2v9k', merchant: 'TechStore Global', amount: '$120.50', crypto: '0.045 ETH', status: 'completed', timestamp: '2024-03-27 10:45', hash: '0x7a2...f8e' },
        { id: 'pay_3m8n5p', merchant: 'CryptoCafe', amount: '$15.00', crypto: '0.25 SOL', status: 'pending', timestamp: '2024-03-27 11:12', hash: '5k9...w2r' },
        { id: 'pay_9l4q1r', merchant: 'FashionHub', amount: '$85.00', crypto: '85.00 USDC', status: 'processing', timestamp: '2024-03-27 11:05', hash: '0x3b1...c4d' },
        { id: 'pay_2w5s8t', merchant: 'EcoFriendly Goods', amount: '$45.20', crypto: '0.0012 BTC', status: 'failed', timestamp: '2024-03-27 09:30', hash: 'bc1...p9q' },
    ];

    const getStatusStyles = (status: string) => {
        switch (status) {
            case 'completed': return 'bg-emerald-50 text-emerald-600 border-emerald-100';
            case 'pending': return 'bg-amber-50 text-amber-600 border-amber-100';
            case 'processing': return 'bg-blue-50 text-blue-600 border-blue-100';
            case 'failed': return 'bg-rose-50 text-rose-600 border-rose-100';
            default: return 'bg-slate-50 text-slate-600 border-slate-100';
        }
    };

    const getStatusIcon = (status: string) => {
        switch (status) {
            case 'completed': return <CheckCircle2 size={14} />;
            case 'pending': return <Clock size={14} />;
            case 'processing': return <Zap size={14} className="animate-pulse" />;
            case 'failed': return <XCircle size={14} />;
            default: return null;
        }
    };

    return (
        <div className="space-y-6 animate-in fade-in duration-500">
            <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
                <div>
                    <h1 className="text-2xl font-bold text-slate-900 tracking-tight">Payments</h1>
                    <p className="text-slate-500 text-sm mt-1">Monitor all incoming crypto transactions across the platform.</p>
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
                </div>
            </div>

            <div className="bg-white rounded-xl border border-slate-200 shadow-sm overflow-hidden">
                <div className="p-4 border-b border-slate-100 bg-slate-50/50 flex flex-wrap gap-4">
                    <div className="relative flex-1 min-w-[300px]">
                        <Search className="absolute left-3 top-1/2 -translate-y-1/2 text-slate-400" size={18} />
                        <input 
                            type="text" 
                            placeholder="Search by ID, Merchant or TX Hash..." 
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
                                <th className="px-6 py-4">Transaction ID</th>
                                <th className="px-6 py-4">Merchant</th>
                                <th className="px-6 py-4 text-right">Amount</th>
                                <th className="px-6 py-4 text-right">Crypto</th>
                                <th className="px-6 py-4 text-center">Status</th>
                                <th className="px-6 py-4">Date & Time</th>
                                <th className="px-6 py-4">Actions</th>
                            </tr>
                        </thead>
                        <tbody className="divide-y divide-slate-100">
                            {transactions.map((tx) => (
                                <tr key={tx.id} className="hover:bg-slate-50/50 transition-colors">
                                    <td className="px-6 py-4">
                                        <div className="font-mono text-xs text-slate-900 bg-slate-100 px-2 py-1 rounded inline-block">
                                            {tx.id}
                                        </div>
                                    </td>
                                    <td className="px-6 py-4 text-sm font-medium text-slate-700">{tx.merchant}</td>
                                    <td className="px-6 py-4 text-sm font-bold text-slate-900 text-right">{tx.amount}</td>
                                    <td className="px-6 py-4 text-sm text-slate-600 text-right">{tx.crypto}</td>
                                    <td className="px-6 py-4 text-center">
                                        <span className={`inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full text-xs font-bold border capitalize ${getStatusStyles(tx.status)}`}>
                                            {getStatusIcon(tx.status)}
                                            {tx.status}
                                        </span>
                                    </td>
                                    <td className="px-6 py-4 text-xs text-slate-500">{tx.timestamp}</td>
                                    <td className="px-6 py-4">
                                        <div className="flex items-center gap-2">
                                            <button title="Re-verify" className="p-2 text-slate-400 hover:text-blue-600 hover:bg-blue-50 rounded-lg transition-all">
                                                <RotateCcw size={16} />
                                            </button>
                                            <button title="Options" className="p-2 text-slate-400 hover:text-slate-600 hover:bg-slate-100 rounded-lg transition-all">
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

export default PaymentsPage;
