import React, { useState } from 'react';
import { 
    ArrowRightLeft, 
    Zap, 
    ShieldCheck, 
    TrendingUp, 
    RefreshCcw,
    AlertCircle,
    Copy,
    ChevronRight
} from 'lucide-react';

interface WalletInfo {
    crypto: string;
    address: string;
    hotBalance: string;
    coldBalance: string;
    totalUsd: string;
    lastSweep: string;
    pendingFees: string;
}

const WalletsPage: React.FC = () => {
    const wallets: WalletInfo[] = [
        { crypto: 'ETH', address: '0x123...456', hotBalance: '12.45', coldBalance: '150.00', totalUsd: '$375,000.00', lastSweep: '2024-03-26 10:00', pendingFees: '0.12 ETH' },
        { crypto: 'SOL', address: 'ABC...XYZ', hotBalance: '450.00', coldBalance: '2,500.00', totalUsd: '$442,500.00', lastSweep: '2024-03-27 08:30', pendingFees: '5.4 SOL' },
        { crypto: 'BTC', address: 'bc1...p9q', hotBalance: '0.24', coldBalance: '5.00', totalUsd: '$360,000.00', lastSweep: '2024-03-25 14:20', pendingFees: '0.0015 BTC' },
    ];

    const [sweeping, setSweeping] = useState<string | null>(null);

    const handleSweep = (crypto: string) => {
        setSweeping(crypto);
        setTimeout(() => setSweeping(null), 2000);
    };

    return (
        <div className="space-y-6 animate-in fade-in duration-500">
            <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
                <div>
                    <h1 className="text-2xl font-bold text-slate-900 tracking-tight">Wallets & Fee Sweep</h1>
                    <p className="text-slate-500 text-sm mt-1">Manage platform hot/cold wallets and trigger manual fee collections.</p>
                </div>
                <div className="flex items-center gap-3">
                    <button className="flex items-center gap-2 px-4 py-2 bg-white border border-slate-200 rounded-lg text-sm font-medium text-slate-700 hover:bg-slate-50 transition-colors shadow-sm">
                        <ArrowRightLeft size={16} />
                        Internal Transfer
                    </button>
                    <button className="flex items-center gap-2 px-4 py-2 bg-primary-600 rounded-lg text-sm font-medium text-white hover:bg-primary-700 transition-colors shadow-sm">
                        <Zap size={16} />
                        Sweep All Eligible
                    </button>
                </div>
            </div>

            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
                {[
                    { label: 'Total Platform Value', value: '$1,177,500', icon: TrendingUp, color: 'emerald' },
                    { label: 'Hot Wallet Total', value: '$125,400', icon: Zap, color: 'amber' },
                    { label: 'Cold Storage Total', value: '$1,052,100', icon: ShieldCheck, color: 'blue' },
                    { label: 'Pending Collections', value: '$1,450', icon: AlertCircle, color: 'rose' }
                ].map((stat, i) => (
                    <div key={i} className="bg-white p-6 rounded-xl border border-slate-200 shadow-sm flex items-center justify-between">
                        <div>
                            <div className="text-xs font-semibold text-slate-500 uppercase tracking-wider">{stat.label}</div>
                            <div className="text-xl font-bold text-slate-900 mt-1">{stat.value}</div>
                        </div>
                        <div className={`p-3 bg-${stat.color}-50 text-${stat.color}-600 rounded-lg`}>
                            <stat.icon size={20} />
                        </div>
                    </div>
                ))}
            </div>

            <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
                <div className="lg:col-span-2 space-y-6">
                    {wallets.map((wallet) => (
                        <div key={wallet.crypto} className="bg-white rounded-xl border border-slate-200 shadow-sm overflow-hidden hover:border-primary-200 transition-colors group">
                            <div className="p-6">
                                <div className="flex items-center justify-between mb-6">
                                    <div className="flex items-center gap-4">
                                        <div className="w-12 h-12 rounded-xl bg-slate-900 text-white flex items-center justify-center font-bold text-lg">
                                            {wallet.crypto}
                                        </div>
                                        <div>
                                            <h3 className="text-lg font-bold text-slate-900">{wallet.crypto} Network Wallet</h3>
                                            <div className="flex items-center gap-2 text-xs text-slate-500 font-mono mt-0.5">
                                                {wallet.address}
                                                <button className="hover:text-primary-600"><Copy size={12} /></button>
                                            </div>
                                        </div>
                                    </div>
                                    <div className="text-right">
                                        <div className="text-xs font-semibold text-slate-400 uppercase">Total Value</div>
                                        <div className="text-xl font-bold text-emerald-600">{wallet.totalUsd}</div>
                                    </div>
                                </div>

                                <div className="grid grid-cols-2 lg:grid-cols-4 gap-4 p-4 bg-slate-50 rounded-lg border border-slate-100">
                                    <div>
                                        <div className="text-[10px] font-bold text-slate-400 uppercase">Hot Balance</div>
                                        <div className="text-sm font-bold text-slate-900">{wallet.hotBalance} {wallet.crypto}</div>
                                    </div>
                                    <div>
                                        <div className="text-[10px] font-bold text-slate-400 uppercase">Cold Balance</div>
                                        <div className="text-sm font-bold text-slate-900">{wallet.coldBalance} {wallet.crypto}</div>
                                    </div>
                                    <div>
                                        <div className="text-[10px] font-bold text-slate-400 uppercase">Last Sweep</div>
                                        <div className="text-xs font-medium text-slate-600">{wallet.lastSweep.split(' ')[0]}</div>
                                    </div>
                                    <div className="text-right">
                                        <div className="text-[10px] font-bold text-rose-400 uppercase">Uncollected Fees</div>
                                        <div className="text-sm font-bold text-rose-600">{wallet.pendingFees}</div>
                                    </div>
                                </div>
                            </div>
                            <div className="px-6 py-4 bg-slate-50/50 border-t border-slate-100 flex items-center justify-between">
                                <span className="text-xs text-slate-500 italic">Sweep threshold: 0.05 {wallet.crypto}</span>
                                <button 
                                    onClick={() => handleSweep(wallet.crypto)}
                                    disabled={sweeping === wallet.crypto}
                                    className={`flex items-center gap-2 px-4 py-1.5 rounded-lg text-xs font-bold transition-all shadow-sm ${
                                        sweeping === wallet.crypto 
                                        ? 'bg-slate-200 text-slate-500 cursor-not-allowed' 
                                        : 'bg-white border border-slate-200 text-slate-700 hover:bg-primary-50 hover:text-primary-600 hover:border-primary-200'
                                    }`}
                                >
                                    <RefreshCcw size={14} className={sweeping === wallet.crypto ? 'animate-spin' : ''} />
                                    {sweeping === wallet.crypto ? 'Sweeping...' : 'Trigger Manual Sweep'}
                                </button>
                            </div>
                        </div>
                    ))}
                </div>

                <div className="space-y-6">
                    <div className="bg-white rounded-xl border border-slate-200 shadow-sm overflow-hidden p-6">
                        <h2 className="text-lg font-bold text-slate-900 mb-4 flex items-center gap-2">
                            <TrendingUp className="text-emerald-500" size={20} />
                            Collection History
                        </h2>
                        <div className="space-y-4">
                            {[
                                { date: 'Today, 10:45', crypto: 'ETH', amount: '0.12 ETH', status: 'Success' },
                                { date: 'Yesterday, 15:20', crypto: 'SOL', amount: '8.45 SOL', status: 'Success' },
                                { date: '25 Mar, 09:10', crypto: 'USDC', amount: '1,200 USDC', status: 'Success' },
                                { date: '24 Mar, 11:30', crypto: 'ETH', amount: '0.08 ETH', status: 'Success' },
                            ].map((log, i) => (
                                <div key={i} className="flex items-center justify-between p-3 rounded-lg border border-slate-50 hover:bg-slate-50 transition-colors group cursor-default">
                                    <div>
                                        <div className="text-sm font-bold text-slate-800">{log.amount}</div>
                                        <div className="text-[10px] text-slate-500">{log.date}</div>
                                    </div>
                                    <div className="flex flex-col items-end">
                                        <div className="text-[10px] font-bold text-emerald-600 uppercase tracking-widest">{log.status}</div>
                                        <ChevronRight size={14} className="text-slate-300 group-hover:text-slate-500 transition-colors" />
                                    </div>
                                </div>
                            ))}
                        </div>
                        <button className="w-full mt-6 py-2 text-sm font-bold text-primary-600 hover:bg-primary-50 rounded-lg transition-colors border border-transparent hover:border-primary-100">
                            View Full History
                        </button>
                    </div>

                    <div className="bg-primary-600 rounded-xl shadow-lg p-6 text-white overflow-hidden relative group">
                         <div className="absolute -right-4 -bottom-4 opacity-10 transform scale-150 rotate-12 group-hover:rotate-0 transition-transform duration-700">
                            <ShieldCheck size={120} />
                        </div>
                        <h2 className="text-lg font-bold mb-2">Automated Rules</h2>
                        <p className="text-primary-100 text-sm mb-4 leading-relaxed">
                            Fee collection is automatically triggered when uncollected fees exceed $500 value or every 24 hours.
                        </p>
                        <button className="px-4 py-2 bg-white/10 hover:bg-white/20 rounded-lg text-sm font-bold transition-colors">
                            Configure Rules
                        </button>
                    </div>
                </div>
            </div>
        </div>
    );
};

export default WalletsPage;
