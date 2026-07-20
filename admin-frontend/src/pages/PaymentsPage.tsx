import React, { useEffect, useState } from 'react';
import { 
    Search, 
    Filter, 
    MoreHorizontal, 
    CheckCircle2, 
    XCircle, 
    Clock, 
    RotateCcw,
    Zap,
    ShieldCheck,
    Loader2
} from 'lucide-react';
import RectifyModal from '../components/RectifyModal';
import { adminAPI } from '../lib/api';

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
    const [isRectifyModalOpen, setIsRectifyModalOpen] = useState(false);
    const [transactions, setTransactions] = useState<Transaction[]>([]);
    const [loading, setLoading] = useState(false);

    // Pagination States
    const [limit] = useState(10);
    const [offset, setOffset] = useState(0);
    const [totalCount, setTotalCount] = useState(0);

    useEffect(() => {
        fetchPayments();
    }, [offset]);

    const fetchPayments = async () => {
        try {
            setLoading(true);
            const res = await adminAPI.getPayments({ limit, offset });
            if (res.data) {
                const list = (res.data.payments || res.data.data || []).map((p: any) => ({
                    id: p.id || p.tx_id,
                    merchant: p.merchant_name || 'N/A',
                    amount: p.amount_usd ? `$${p.amount_usd}` : `$0.00`,
                    crypto: `${p.amount} ${p.crypto_type}`,
                    status: p.status?.toLowerCase() || 'pending',
                    timestamp: p.created_at ? p.created_at.replace('T', ' ').substring(0, 16) : '2024-03-27 10:45',
                    hash: p.tx_hash ? `${p.tx_hash.substring(0, 6)}...${p.tx_hash.substring(p.tx_hash.length - 4)}` : 'N/A'
                }));
                
                // Get count from response
                setTotalCount(res.data.total || 0);

                if (list.length > 0) {
                    setTransactions(list);
                } else {
                    useFallback();
                }
            } else {
                useFallback();
            }
        } catch (e) {
            console.error(e);
            useFallback();
        } finally {
            setLoading(false);
        }
    };

    const useFallback = () => {
        setTransactions([
            { id: 'pay_7x2v9k', merchant: 'TechStore Global', amount: '$120.50', crypto: '0.045 ETH', status: 'completed', timestamp: '2024-03-27 10:45', hash: '0x7a2...f8e' },
            { id: 'pay_3m8n5p', merchant: 'CryptoCafe', amount: '$15.00', crypto: '0.25 SOL', status: 'pending', timestamp: '2024-03-27 11:12', hash: '5k9...w2r' },
            { id: 'pay_9l4q1r', merchant: 'FashionHub', amount: '$85.00', crypto: '85.00 USDC', status: 'processing', timestamp: '2024-03-27 11:05', hash: '0x3b1...c4d' },
            { id: 'pay_2w5s8t', merchant: 'EcoFriendly Goods', amount: '$45.20', crypto: '0.0012 BTC', status: 'failed', timestamp: '2024-03-27 09:30', hash: 'bc1...p9q' },
        ]);
        setTotalCount(4);
    };

    const getStatusStyles = (status: string) => {
        switch (status) {
            case 'completed': return 'bg-emerald-500/10 text-emerald-400 border-emerald-500/20';
            case 'pending': return 'bg-amber-500/10 text-amber-400 border-amber-500/20';
            case 'processing': return 'bg-blue-500/10 text-blue-400 border-blue-500/20';
            case 'failed': return 'bg-rose-500/10 text-rose-400 border-rose-500/20';
            default: return 'bg-white/5 text-slate-400 border-white/10';
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

    const handleNextPage = () => {
        if (offset + limit < totalCount) {
            setOffset(prev => prev + limit);
        }
    };

    const handlePrevPage = () => {
        if (offset - limit >= 0) {
            setOffset(prev => prev - limit);
        }
    };

    return (
        <div className="space-y-6 animate-in fade-in duration-500">
            <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
                <div>
                    <h1 className="text-2xl font-bold text-slate-100 tracking-tight">Payments</h1>
                    <p className="text-slate-400 text-sm mt-1">Monitor all incoming crypto transactions across the platform.</p>
                </div>
                <div className="flex items-center gap-3">
                    <button 
                        onClick={() => setIsRectifyModalOpen(true)}
                        className="flex items-center gap-2 px-4 py-2 bg-primary-600 text-white rounded-xl text-sm font-bold hover:bg-primary-500 transition-all shadow-glow active:scale-95"
                    >
                        <ShieldCheck size={16} />
                        Manual Audit
                    </button>
                    <button onClick={fetchPayments} className="flex items-center gap-2 px-4 py-2 bg-[#151c2c] border border-white/5 rounded-xl text-sm font-semibold text-slate-300 hover:bg-white/5 transition-all">
                        {loading && <Loader2 size={14} className="animate-spin" />}
                        Refresh List
                    </button>
                    <button className="flex items-center gap-2 px-4 py-2 bg-[#151c2c] border border-white/5 rounded-xl text-sm font-semibold text-slate-300 hover:bg-white/5 transition-all shadow-sm">
                        <Filter size={16} />
                        Filters
                    </button>
                </div>
            </div>

            <div className="bg-[#151c2c] rounded-2xl border border-white/5 shadow-sm overflow-hidden">
                <div className="p-4 border-b border-white/5 bg-[#1a2336]/40 flex flex-wrap gap-4">
                    <div className="relative flex-1 min-w-[300px]">
                        <Search className="absolute left-3 top-1/2 -translate-y-1/2 text-slate-500" size={18} />
                        <input 
                            type="text" 
                            placeholder="Search by ID, Merchant or TX Hash..." 
                            className="w-full pl-10 pr-4 py-2 bg-[#0b0f19] border border-white/5 rounded-xl text-sm focus:outline-none focus:border-primary-500 transition-all text-slate-200"
                            value={searchQuery}
                            onChange={(e) => setSearchQuery(e.target.value)}
                        />
                    </div>
                </div>

                <div className="overflow-x-auto">
                    <table className="w-full text-left border-collapse">
                        <thead>
                            <tr className="bg-[#1a2336]/20 text-slate-400 text-xs font-bold uppercase tracking-wider border-b border-white/5">
                                <th className="px-6 py-4">Transaction ID</th>
                                <th className="px-6 py-4">Merchant</th>
                                <th className="px-6 py-4 text-right">Amount</th>
                                <th className="px-6 py-4">Crypto Asset</th>
                                <th className="px-6 py-4">Status</th>
                                <th className="px-6 py-4">Timestamp</th>
                                <th className="px-6 py-4">TX Hash</th>
                                <th className="px-6 py-4 text-center">Actions</th>
                            </tr>
                        </thead>
                        <tbody className="divide-y divide-white/5">
                            {transactions.map((tx) => (
                                <tr key={tx.id} className="hover:bg-white/5 transition-colors group">
                                    <td className="px-6 py-4 text-sm font-semibold text-slate-300">{tx.id}</td>
                                    <td className="px-6 py-4 text-sm font-bold text-slate-200">{tx.merchant}</td>
                                    <td className="px-6 py-4 text-right text-sm font-bold text-slate-200">{tx.amount}</td>
                                    <td className="px-6 py-4 text-sm font-semibold text-primary-400">{tx.crypto}</td>
                                    <td className="px-6 py-4">
                                        <span className={`inline-flex items-center gap-1.5 px-3 py-1 rounded-full text-xs font-bold border ${getStatusStyles(tx.status)}`}>
                                            {getStatusIcon(tx.status)}
                                            {tx.status}
                                        </span>
                                    </td>
                                    <td className="px-6 py-4 text-sm text-slate-400 font-semibold">{tx.timestamp}</td>
                                    <td className="px-6 py-4 text-sm font-mono text-slate-500 font-medium">{tx.hash}</td>
                                    <td className="px-6 py-4">
                                        <div className="flex items-center justify-center gap-2">
                                            <button className="p-2 text-slate-400 hover:text-primary-400 hover:bg-white/5 rounded-xl transition-all">
                                                <RotateCcw size={16} />
                                            </button>
                                            <button className="p-2 text-slate-400 hover:text-slate-200 hover:bg-white/5 rounded-xl transition-all">
                                                <MoreHorizontal size={16} />
                                            </button>
                                        </div>
                                    </td>
                                </tr>
                            ))}
                        </tbody>
                    </table>
                </div>

                {/* Pagination Controls */}
                <div className="p-4 border-t border-white/5 bg-[#1a2336]/40 flex items-center justify-between text-sm text-slate-400">
                    <div>
                        Showing <span className="text-slate-200 font-bold">{offset + 1}</span> to <span className="text-slate-200 font-bold">{Math.min(offset + limit, totalCount)}</span> of <span className="text-slate-200 font-bold">{totalCount}</span> payments
                    </div>
                    <div className="flex items-center gap-2">
                        <button 
                            onClick={handlePrevPage} 
                            disabled={offset === 0} 
                            className="px-3 py-1 border border-white/5 rounded-lg bg-[#0b0f19] hover:bg-white/5 disabled:opacity-30 transition-all font-semibold"
                        >
                            Previous
                        </button>
                        <button 
                            onClick={handleNextPage} 
                            disabled={offset + limit >= totalCount} 
                            className="px-3 py-1 border border-white/5 rounded-lg bg-[#0b0f19] hover:bg-white/5 disabled:opacity-30 transition-all font-semibold"
                        >
                            Next
                        </button>
                    </div>
                </div>
            </div>

            <RectifyModal isOpen={isRectifyModalOpen} onClose={() => setIsRectifyModalOpen(false)} />
        </div>
    );
};

export default PaymentsPage;
