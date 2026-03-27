import React, { useState } from 'react';
import { 
    Search, 
    Filter, 
    MoreHorizontal, 
    ExternalLink, 
    Shield, 
    ShieldAlert, 
    CheckCircle2
} from 'lucide-react';

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

    // Mock data for initial UI
    const merchants: Merchant[] = [
        { id: 1, name: 'TechStore Global', email: 'admin@techstore.com', status: 'active', joinedDate: '2024-01-15', totalVolume: '$12,450.00', transactions: 156 },
        { id: 2, name: 'EcoFriendly Goods', email: 'contact@ecofriendly.org', status: 'active', joinedDate: '2024-02-01', totalVolume: '$3,200.00', transactions: 42 },
        { id: 3, name: 'CryptoCafe', email: 'hello@cryptocafe.io', status: 'suspended', joinedDate: '2023-11-20', totalVolume: '$850.00', transactions: 12 },
        { id: 4, name: 'FashionHub', email: 'billing@fashionhub.net', status: 'pending', joinedDate: '2024-03-10', totalVolume: '$0.00', transactions: 0 },
    ];

    const getStatusColor = (status: string) => {
        switch (status) {
            case 'active': return 'bg-emerald-100 text-emerald-700 border-emerald-200';
            case 'suspended': return 'bg-rose-100 text-rose-700 border-rose-200';
            case 'pending': return 'bg-amber-100 text-amber-700 border-amber-200';
            default: return 'bg-slate-100 text-slate-700 border-slate-200';
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
                    <h1 className="text-2xl font-bold text-slate-900 tracking-tight">Merchants</h1>
                    <p className="text-slate-500 text-sm mt-1">Manage all registered merchants and their platform access.</p>
                </div>
                <div className="flex items-center gap-3">
                    <button className="flex items-center gap-2 px-4 py-2 bg-white border border-slate-200 rounded-lg text-sm font-medium text-slate-700 hover:bg-slate-50 transition-colors shadow-sm">
                        <Filter size={16} />
                        Filters
                    </button>
                    <button className="flex items-center gap-2 px-4 py-2 bg-primary-600 rounded-lg text-sm font-medium text-white hover:bg-primary-700 transition-colors shadow-sm">
                        Export Data
                    </button>
                </div>
            </div>

            <div className="bg-white rounded-xl border border-slate-200 shadow-sm overflow-hidden">
                <div className="p-4 border-b border-slate-100 bg-slate-50/50">
                    <div className="relative max-w-md">
                        <Search className="absolute left-3 top-1/2 -translate-y-1/2 text-slate-400" size={18} />
                        <input 
                            type="text" 
                            placeholder="Search by name, email or ID..." 
                            className="w-full pl-10 pr-4 py-2 bg-white border border-slate-200 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-primary-500/20 focus:border-primary-500 transition-all"
                            value={searchQuery}
                            onChange={(e) => setSearchQuery(e.target.value)}
                        />
                    </div>
                </div>

                <div className="overflow-x-auto">
                    <table className="w-full text-left border-collapse">
                        <thead>
                            <tr className="bg-slate-50/50 text-slate-500 text-xs font-semibold uppercase tracking-wider">
                                <th className="px-6 py-4">Merchant</th>
                                <th className="px-6 py-4">Status</th>
                                <th className="px-6 py-4">Joined Date</th>
                                <th className="px-6 py-4 text-right">Volume</th>
                                <th className="px-6 py-4 text-center">Actions</th>
                            </tr>
                        </thead>
                        <tbody className="divide-y divide-slate-100">
                            {merchants.map((merchant) => (
                                <tr key={merchant.id} className="hover:bg-slate-50/50 transition-colors group">
                                    <td className="px-6 py-4">
                                        <div className="flex items-center gap-3">
                                            <div className="w-10 h-10 rounded-full bg-slate-100 flex items-center justify-center text-primary-600 font-bold text-sm">
                                                {merchant.name.charAt(0)}
                                            </div>
                                            <div>
                                                <div className="text-sm font-semibold text-slate-900">{merchant.name}</div>
                                                <div className="text-xs text-slate-500">{merchant.email}</div>
                                            </div>
                                        </div>
                                    </td>
                                    <td className="px-6 py-4">
                                        <span className={`inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full text-xs font-medium border ${getStatusColor(merchant.status)}`}>
                                            {getStatusIcon(merchant.status)}
                                            {merchant.status.charAt(0).toUpperCase() + merchant.status.slice(1)}
                                        </span>
                                    </td>
                                    <td className="px-6 py-4 text-sm text-slate-600">
                                        {merchant.joinedDate}
                                    </td>
                                    <td className="px-6 py-4 text-sm font-medium text-slate-900 text-right">
                                        {merchant.totalVolume}
                                    </td>
                                    <td className="px-6 py-4 text-center">
                                        <div className="flex items-center justify-center gap-2">
                                            <button title="View Details" className="p-2 text-slate-400 hover:text-primary-600 hover:bg-primary-50 rounded-lg transition-all">
                                                <ExternalLink size={18} />
                                            </button>
                                            <button title="More Actions" className="p-2 text-slate-400 hover:text-slate-600 hover:bg-slate-100 rounded-lg transition-all">
                                                <MoreHorizontal size={18} />
                                            </button>
                                        </div>
                                    </td>
                                </tr>
                            ))}
                        </tbody>
                    </table>
                </div>

                <div className="p-4 border-t border-slate-100 bg-slate-50/30 flex items-center justify-between text-sm text-slate-500">
                    <div>Showing 4 of 42 merchants</div>
                    <div className="flex items-center gap-2">
                        <button className="px-3 py-1 border border-slate-200 rounded bg-white hover:bg-slate-50 disabled:opacity-50" disabled>Previous</button>
                        <button className="px-3 py-1 border border-slate-200 rounded bg-white hover:bg-slate-50">Next</button>
                    </div>
                </div>
            </div>
        </div>
    );
};

export default MerchantsPage;
