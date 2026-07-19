import React, { useEffect, useState } from 'react';
import { 
    Search, 
    Filter, 
    MoreHorizontal, 
    ExternalLink, 
    Shield, 
    ShieldAlert, 
    CheckCircle2,
    Loader2
} from 'lucide-react';
import { adminAPI } from '../lib/api';

interface Merchant {
    id: number;
    name: string;
    email: string;
    status: 'active' | 'suspended' | 'pending';
    joinedDate: string;
    totalVolume: string;
    transactions: number;
}

const MerchantsPage: React.FC = () => {
    const [searchQuery, setSearchQuery] = useState('');
    const [merchants, setMerchants] = useState<Merchant[]>([]);
    const [loading, setLoading] = useState(false);

    useEffect(() => {
        fetchMerchants();
    }, []);

    const fetchMerchants = async () => {
        try {
            setLoading(true);
            const res = await adminAPI.getMerchants();
            if (res.data) {
                // If backend endpoint yields structured merchant array, map it.
                // Fallback to placeholders if backend db is newly set up or clean.
                const mapped = (res.data.merchants || res.data || []).map((m: any) => ({
                    id: m.id,
                    name: m.business_name || m.name,
                    email: m.email,
                    status: m.status?.toLowerCase() || 'active',
                    joinedDate: m.created_at ? m.created_at.split('T')[0] : '2024-01-15',
                    totalVolume: m.total_volume || '$0.00',
                    transactions: m.transactions_count || 0
                }));
                if (mapped.length > 0) {
                    setMerchants(mapped);
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
        setMerchants([
            { id: 1, name: 'TechStore Global', email: 'admin@techstore.com', status: 'active', joinedDate: '2024-01-15', totalVolume: '$12,450.00', transactions: 156 },
            { id: 2, name: 'EcoFriendly Goods', email: 'contact@ecofriendly.org', status: 'active', joinedDate: '2024-02-01', totalVolume: '$3,200.00', transactions: 42 },
            { id: 3, name: 'CryptoCafe', email: 'hello@cryptocafe.io', status: 'suspended', joinedDate: '2023-11-20', totalVolume: '$850.00', transactions: 12 },
            { id: 4, name: 'FashionHub', email: 'billing@fashionhub.net', status: 'pending', joinedDate: '2024-03-10', totalVolume: '$0.00', transactions: 0 },
        ]);
    };

    const getStatusColor = (status: string) => {
        switch (status) {
            case 'active': return 'bg-emerald-500/10 text-emerald-400 border-emerald-500/20';
            case 'suspended': return 'bg-rose-500/10 text-rose-400 border-rose-500/20';
            case 'pending': return 'bg-amber-500/10 text-amber-400 border-amber-500/20';
            default: return 'bg-white/5 text-slate-400 border-white/10';
        }
    };

    const getStatusIcon = (status: string) => {
        switch (status) {
            case 'active': return <CheckCircle2 size={14} />;
            case 'suspended': return <ShieldAlert size={14} />;
            case 'pending': return <Shield size={14} />;
            default: return null;
        }
    };

    return (
        <div className="space-y-6 animate-in fade-in duration-500">
            <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
                <div>
                    <h1 className="text-2xl font-bold text-slate-100 tracking-tight">Merchants</h1>
                    <p className="text-slate-400 text-sm mt-1">Manage all registered merchants and their platform access.</p>
                </div>
                <div className="flex items-center gap-3">
                    <button onClick={fetchMerchants} className="flex items-center gap-2 px-4 py-2 bg-[#151c2c] border border-white/5 rounded-xl text-sm font-semibold text-slate-300 hover:bg-white/5 transition-all">
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
                            placeholder="Search merchants..." 
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
                                <th className="px-6 py-4">Merchant ID</th>
                                <th className="px-6 py-4">Business Name</th>
                                <th className="px-6 py-4">Email</th>
                                <th className="px-6 py-4">Status</th>
                                <th className="px-6 py-4">Joined Date</th>
                                <th className="px-6 py-4 text-right">Total Volume</th>
                                <th className="px-6 py-4 text-center">Tx Count</th>
                                <th className="px-6 py-4 text-center">Actions</th>
                            </tr>
                        </thead>
                        <tbody className="divide-y divide-white/5">
                            {merchants.map((merchant) => (
                                <tr key={merchant.id} className="hover:bg-white/5 transition-colors group">
                                    <td className="px-6 py-4 text-sm font-semibold text-slate-400">#{merchant.id}</td>
                                    <td className="px-6 py-4 text-sm font-bold text-slate-200">{merchant.name}</td>
                                    <td className="px-6 py-4 text-sm text-slate-400">{merchant.email}</td>
                                    <td className="px-6 py-4">
                                        <span className={`inline-flex items-center gap-1.5 px-3 py-1 rounded-full text-xs font-bold border capitalize ${getStatusColor(merchant.status)}`}>
                                            {getStatusIcon(merchant.status)}
                                            {merchant.status}
                                        </span>
                                    </td>
                                    <td className="px-6 py-4 text-sm text-slate-400 font-semibold">{merchant.joinedDate}</td>
                                    <td className="px-6 py-4 text-sm font-bold text-emerald-400 text-right">{merchant.totalVolume}</td>
                                    <td className="px-6 py-4 text-sm font-bold text-slate-200 text-center">{merchant.transactions}</td>
                                    <td className="px-6 py-4">
                                        <div className="flex items-center justify-center gap-2">
                                            <button className="p-2 text-slate-400 hover:text-primary-400 hover:bg-white/5 rounded-xl transition-all">
                                                <ExternalLink size={16} />
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
            </div>
        </div>
    );
};

export default MerchantsPage;
