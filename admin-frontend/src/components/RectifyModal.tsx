import React, { useState } from 'react';
import { 
    X, 
    Zap, 
    Search, 
    ShieldCheck, 
    ShieldAlert, 
    CheckCircle2, 
    AlertCircle,
    Loader2,
    Database,
    Globe
} from 'lucide-react';
import { adminAPI } from '../lib/api';

interface RectifyModalProps {
    isOpen: boolean;
    onClose: () => void;
    initialAddress?: string;
    initialCrypto?: string;
}

const RectifyModal: React.FC<RectifyModalProps> = ({ isOpen, onClose, initialAddress = '', initialCrypto = 'BNB' }) => {
    const [address, setAddress] = useState(initialAddress);
    const [cryptoType, setCryptoType] = useState(initialCrypto);
    const [dryRun, setDryRun] = useState(true);
    const [loading, setLoading] = useState(false);
    const [report, setReport] = useState<any>(null);
    const [error, setError] = useState<string | null>(null);

    const handleRectify = async () => {
        setLoading(true);
        setError(null);
        setReport(null);
        
        try {
            const response = await adminAPI.rectifyOnchain({
                address,
                crypto_type: cryptoType,
                dry_run: dryRun,
                signature_limit: 100,
                rectify_type: "BOTH"
            });
            
            if (response.data.success) {
                setReport(response.data.data);
            } else {
                setError(response.data.error || 'Failed to rectify balance');
            }
        } catch (err: any) {
            setError(err.response?.data?.error || err.message || 'An unexpected error occurred');
        } finally {
            setLoading(false);
        }
    };

    if (!isOpen) return null;

    return (
        <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-slate-900/60 backdrop-blur-sm animate-in fade-in duration-200">
            <div className="bg-white w-full max-w-2xl rounded-2xl shadow-2xl overflow-hidden flex flex-col max-h-[90vh] animate-in zoom-in-95 duration-200">
                {/* Header */}
                <div className="px-6 py-4 border-b border-slate-100 flex items-center justify-between bg-slate-50/50">
                    <div className="flex items-center gap-2">
                        <div className="p-2 bg-primary-100 text-primary-600 rounded-lg">
                            <ShieldCheck size={20} />
                        </div>
                        <div>
                            <h2 className="text-lg font-bold text-slate-900">On-Chain Rectification</h2>
                            <p className="text-xs text-slate-500">Audit and reconcile database balances with blockchain state.</p>
                        </div>
                    </div>
                    <button onClick={onClose} className="p-2 hover:bg-slate-200 rounded-full transition-colors">
                        <X size={20} className="text-slate-500" />
                    </button>
                </div>

                {/* Body */}
                <div className="p-6 overflow-y-auto flex-1 space-y-6">
                    {!report ? (
                        <div className="space-y-4">
                            <div className="grid grid-cols-2 gap-4">
                                <div className="space-y-1.5">
                                    <label className="text-xs font-bold text-slate-500 uppercase">Network / Asset</label>
                                    <select 
                                        value={cryptoType}
                                        onChange={(e) => setCryptoType(e.target.value)}
                                        className="w-full px-4 py-2 bg-white border border-slate-200 rounded-lg text-sm focus:ring-2 focus:ring-primary-500/20 focus:border-primary-500 outline-none"
                                    >
                                        <option value="BNB">Binance Smart Chain (BNB)</option>
                                        <option value="USDT_BEP20">USDT (BEP20)</option>
                                        <option value="SOL">Solana (SOL)</option>
                                        <option value="USDT_SPL">USDT (SPL)</option>
                                        <option value="ETH">Ethereum (ETH)</option>
                                        <option value="MATIC">Polygon (MATIC)</option>
                                    </select>
                                </div>
                                <div className="space-y-1.5">
                                    <label className="text-xs font-bold text-slate-500 uppercase">Audit Mode</label>
                                    <div className="flex p-1 bg-slate-100 rounded-lg">
                                        <button 
                                            onClick={() => setDryRun(true)}
                                            className={`flex-1 py-1.5 text-xs font-bold rounded-md transition-all ${dryRun ? 'bg-white text-slate-900 shadow-sm' : 'text-slate-500 hover:text-slate-700'}`}
                                        >
                                            Dry Run
                                        </button>
                                        <button 
                                            onClick={() => setDryRun(false)}
                                            className={`flex-1 py-1.5 text-xs font-bold rounded-md transition-all ${!dryRun ? 'bg-rose-500 text-white shadow-sm' : 'text-slate-500 hover:text-slate-700'}`}
                                        >
                                            Force Sync
                                        </button>
                                    </div>
                                </div>
                            </div>

                            <div className="space-y-1.5">
                                <label className="text-xs font-bold text-slate-500 uppercase">Wallet Address</label>
                                <div className="relative">
                                    <Search className="absolute left-3 top-1/2 -translate-y-1/2 text-slate-400" size={16} />
                                    <input 
                                        type="text" 
                                        value={address}
                                        onChange={(e) => setAddress(e.target.value)}
                                        placeholder="Enter 0x... or Solana address"
                                        className="w-full pl-10 pr-4 py-2 bg-white border border-slate-200 rounded-lg text-sm focus:ring-2 focus:ring-primary-500/20 focus:border-primary-500 outline-none font-mono"
                                    />
                                </div>
                            </div>

                            {error && (
                                <div className="p-3 bg-rose-50 border border-rose-100 rounded-lg flex gap-3 text-rose-700 text-sm">
                                    <AlertCircle size={18} className="shrink-0" />
                                    {error}
                                </div>
                            )}

                            <div className="p-4 bg-amber-50 border border-amber-100 rounded-xl flex gap-4">
                                <Zap className="text-amber-500 shrink-0" size={24} />
                                <div>
                                    <h4 className="text-sm font-bold text-amber-900">Platform Audit Rule</h4>
                                    <p className="text-xs text-amber-700 mt-1 leading-relaxed">
                                        Rectification will scan the last 100 on-chain transactions and cross-reference them with the ledger. 
                                        {dryRun ? ' This will generate a report without making any changes.' : ' This will VOID missing transactions and FORCE sync the balance.'}
                                    </p>
                                </div>
                            </div>
                        </div>
                    ) : (
                        <div className="space-y-6 animate-in slide-in-from-bottom-4 duration-300">
                            {/* Summary Cards */}
                            <div className="grid grid-cols-3 gap-4">
                                <div className="p-4 bg-slate-50 rounded-xl border border-slate-100">
                                    <div className="flex items-center gap-2 text-slate-500 mb-1">
                                        <Globe size={14} />
                                        <span className="text-[10px] font-bold uppercase">RPC Balance</span>
                                    </div>
                                    <div className="text-lg font-bold text-slate-900">
                                        {report.wallet_reconciliation.actual_rpc_balance} {report.blockchain}
                                    </div>
                                </div>
                                <div className="p-4 bg-slate-50 rounded-xl border border-slate-100">
                                    <div className="flex items-center gap-2 text-slate-500 mb-1">
                                        <Database size={14} />
                                        <span className="text-[10px] font-bold uppercase">Expected</span>
                                    </div>
                                    <div className="text-lg font-bold text-slate-900">
                                        {report.wallet_reconciliation.expected_onchain_balance}
                                    </div>
                                </div>
                                <div className="p-4 rounded-xl border flex flex-col justify-center items-center gap-1">
                                    {report.wallet_reconciliation.onchain_out_of_sync ? (
                                        <>
                                            <div className="p-1 bg-rose-100 text-rose-600 rounded-full">
                                                <ShieldAlert size={16} />
                                            </div>
                                            <span className="text-[10px] font-bold text-rose-600 uppercase">Out of Sync</span>
                                        </>
                                    ) : (
                                        <>
                                            <div className="p-1 bg-emerald-100 text-emerald-600 rounded-full">
                                                <CheckCircle2 size={16} />
                                            </div>
                                            <span className="text-[10px] font-bold text-emerald-600 uppercase">Balanced</span>
                                        </>
                                    )}
                                </div>
                            </div>

                            {/* Detailed Stats */}
                            <div className="space-y-3">
                                <h4 className="text-xs font-bold text-slate-500 uppercase tracking-widest">Audit Details</h4>
                                <div className="bg-slate-50 rounded-xl border border-slate-100 divide-y divide-slate-200/60">
                                    <div className="flex items-center justify-between p-3">
                                        <span className="text-sm text-slate-600">On-chain Deposits (Raw)</span>
                                        <span className="text-sm font-bold text-slate-900">{report.audit_summary.onchain_deposits_raw}</span>
                                    </div>
                                    <div className="flex items-center justify-between p-3">
                                        <span className="text-sm text-slate-600">On-chain Withdrawals (Raw)</span>
                                        <span className="text-sm font-bold text-slate-900">{report.audit_summary.onchain_withdrawals_raw}</span>
                                    </div>
                                    <div className="flex items-center justify-between p-3">
                                        <span className="text-sm text-slate-600">Missing in DB (Rectified)</span>
                                        <span className="text-sm font-bold text-amber-600">+{report.audit_summary.missing_in_db_rectified}</span>
                                    </div>
                                </div>
                            </div>

                            {report.potential_ghosts && report.potential_ghosts.length > 0 && (
                                <div className="space-y-3">
                                    <h4 className="text-xs font-bold text-rose-500 uppercase tracking-widest">Potential Ghost Transactions</h4>
                                    <div className="space-y-2">
                                        {report.potential_ghosts.map((ghost: any, i: number) => (
                                            <div key={i} className="p-3 bg-rose-50 border border-rose-100 rounded-lg flex items-center justify-between">
                                                <div>
                                                    <div className="text-xs font-bold text-rose-900 font-mono">{ghost.hash.substring(0, 10)}...</div>
                                                    <div className="text-[10px] text-rose-600 uppercase font-semibold">{ghost.reason}</div>
                                                </div>
                                                <div className="text-sm font-bold text-rose-700">
                                                    {ghost.amount} {cryptoType}
                                                </div>
                                            </div>
                                        ))}
                                    </div>
                                </div>
                            )}

                            {report.actions_taken && report.actions_taken.length > 0 && (
                                <div className="space-y-3">
                                    <h4 className="text-xs font-bold text-emerald-500 uppercase tracking-widest">Actions Taken</h4>
                                    <div className="space-y-2">
                                        {report.actions_taken.map((action: string, i: number) => (
                                            <div key={i} className="p-3 bg-emerald-50 border border-emerald-100 rounded-lg flex items-center gap-3">
                                                <CheckCircle2 size={14} className="text-emerald-600" />
                                                <span className="text-xs font-medium text-emerald-900">{action}</span>
                                            </div>
                                        ))}
                                    </div>
                                </div>
                            )}
                        </div>
                    )}
                </div>

                {/* Footer */}
                <div className="px-6 py-4 border-t border-slate-100 bg-slate-50/50 flex items-center justify-between">
                    <button 
                        onClick={report ? () => setReport(null) : onClose}
                        className="px-4 py-2 text-sm font-medium text-slate-600 hover:text-slate-900 transition-colors"
                    >
                        {report ? 'Start New Audit' : 'Cancel'}
                    </button>
                    {!report && (
                        <button 
                            onClick={handleRectify}
                            disabled={loading || !address}
                            className="flex items-center gap-2 px-6 py-2 bg-primary-600 text-white rounded-lg text-sm font-bold hover:bg-primary-700 transition-all shadow-lg shadow-primary-200 disabled:opacity-50 disabled:shadow-none"
                        >
                            {loading ? (
                                <>
                                    <Loader2 size={18} className="animate-spin" />
                                    Processing...
                                </>
                            ) : (
                                <>
                                    <ShieldCheck size={18} />
                                    {dryRun ? 'Analyze On-Chain' : 'Perform Force Sync'}
                                </>
                            )}
                        </button>
                    )}
                </div>
            </div>
        </div>
    );
};

export default RectifyModal;
