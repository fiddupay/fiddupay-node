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
    Globe,
    RefreshCw
} from 'lucide-react';
import { adminAPI } from '../lib/api';
import clsx from 'clsx';

interface RectifyModalProps {
    isOpen: boolean;
    onClose: () => void;
    initialAddress?: string;
    initialCrypto?: string;
}

const RectifyModal: React.FC<RectifyModalProps> = ({ isOpen, onClose, initialAddress = '', initialCrypto = 'BNB' }) => {
    const [mode, setMode] = useState<'reconcile' | 'reverify'>('reconcile');

    // Reconcile Form States
    const [address, setAddress] = useState(initialAddress);
    const [cryptoType, setCryptoType] = useState(initialCrypto);
    const [dryRun, setDryRun] = useState(true);

    // Re-verify Form States
    const [txHash, setTxHash] = useState('');
    const [txType, setTxType] = useState<'customer' | 'merchant'>('customer');
    const [associatedId, setAssociatedId] = useState('');
    const [sandboxMode, setSandboxMode] = useState(false);

    // Shared Status States
    const [loading, setLoading] = useState(false);
    const [report, setReport] = useState<any>(null);
    const [error, setError] = useState<string | null>(null);
    const [successMessage, setSuccessMessage] = useState<string | null>(null);

    const handleRectify = async () => {
        setLoading(true);
        setError(null);
        setReport(null);
        setSuccessMessage(null);
        
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

    const handleReverify = async () => {
        setLoading(true);
        setError(null);
        setReport(null);
        setSuccessMessage(null);

        try {
            const response = await adminAPI.reverifyTransaction({
                hash: txHash,
                tx_type: txType,
                id: parseInt(associatedId, 10),
                crypto_type: cryptoType,
                sandbox_mode: sandboxMode
            });

            if (response.data && (response.data.success || response.data.status === 'confirmed' || response.data.status === 'success')) {
                setSuccessMessage(response.data.message || 'Transaction re-verified and synchronized successfully.');
            } else {
                setError(response.data.error || 'Failed to re-verify transaction');
            }
        } catch (err: any) {
            setError(err.response?.data?.error || err.message || 'An unexpected error occurred during re-verification');
        } finally {
            setLoading(false);
        }
    };

    if (!isOpen) return null;

    return (
        <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-slate-950/80 backdrop-blur-sm animate-in fade-in duration-200">
            <div className="bg-[#151c2c] w-full max-w-2xl rounded-2xl border border-white/5 shadow-2xl overflow-hidden flex flex-col max-h-[90vh] animate-in zoom-in-95 duration-200">
                {/* Header */}
                <div className="px-6 py-4 border-b border-white/5 flex items-center justify-between bg-[#1a2336]/40">
                    <div className="flex items-center gap-2">
                        <div className="p-2 bg-primary-600/10 border border-primary-500/20 text-primary-400 rounded-xl">
                            <ShieldCheck size={20} />
                        </div>
                        <div>
                            <h2 className="text-lg font-bold text-slate-200">Manual Audit & Rectification</h2>
                            <p className="text-xs text-slate-400 mt-0.5">Audit wallet states or verify specific on-chain transactions.</p>
                        </div>
                    </div>
                    <button onClick={onClose} className="p-2 hover:bg-white/5 rounded-full transition-all">
                        <X size={20} className="text-slate-400 hover:text-slate-200" />
                    </button>
                </div>

                {/* Audit Tab Selector */}
                <div className="px-6 pt-4 bg-[#1a2336]/10 border-b border-white/5 flex gap-4">
                    <button 
                        onClick={() => { setMode('reconcile'); setReport(null); setError(null); setSuccessMessage(null); }}
                        className={clsx(
                            "pb-3 text-sm font-semibold border-b-2 transition-all",
                            mode === 'reconcile' ? "border-primary-500 text-primary-400" : "border-transparent text-slate-400 hover:text-slate-200"
                        )}
                    >
                        On-Chain Reconcile
                    </button>
                    <button 
                        onClick={() => { setMode('reverify'); setReport(null); setError(null); setSuccessMessage(null); }}
                        className={clsx(
                            "pb-3 text-sm font-semibold border-b-2 transition-all",
                            mode === 'reverify' ? "border-primary-500 text-primary-400" : "border-transparent text-slate-400 hover:text-slate-200"
                        )}
                    >
                        Re-Verify Static Tx
                    </button>
                </div>

                {/* Body */}
                <div className="p-6 overflow-y-auto flex-1 space-y-6">
                    {mode === 'reconcile' ? (
                        /* ON-CHAIN RECONCILE MODE */
                        !report ? (
                            <div className="space-y-4">
                                <div className="grid grid-cols-2 gap-4">
                                    <div className="space-y-1.5">
                                        <label className="text-xs font-bold text-slate-400 uppercase">Network / Asset</label>
                                        <select 
                                            value={cryptoType}
                                            onChange={(e) => setCryptoType(e.target.value)}
                                            className="w-full px-4 py-2 bg-[#0b0f19] border border-white/5 rounded-xl text-sm focus:outline-none focus:border-primary-500 text-slate-200 font-medium"
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
                                        <label className="text-xs font-bold text-slate-400 uppercase">Audit Mode</label>
                                        <div className="flex p-1 bg-[#0b0f19] rounded-xl border border-white/5">
                                            <button 
                                                onClick={() => setDryRun(true)}
                                                className={`flex-1 py-1.5 text-xs font-bold rounded-lg transition-all ${dryRun ? 'bg-primary-600 text-white shadow-glow' : 'text-slate-400 hover:text-slate-200'}`}
                                            >
                                                Dry Run
                                            </button>
                                            <button 
                                                onClick={() => setDryRun(false)}
                                                className={`flex-1 py-1.5 text-xs font-bold rounded-lg transition-all ${!dryRun ? 'bg-rose-500/20 text-rose-400 border border-rose-500/30' : 'text-slate-400 hover:text-slate-200'}`}
                                            >
                                                Force Sync
                                            </button>
                                        </div>
                                    </div>
                                </div>

                                <div className="space-y-1.5">
                                    <label className="text-xs font-bold text-slate-400 uppercase">Wallet Address</label>
                                    <div className="relative">
                                        <Search className="absolute left-3 top-1/2 -translate-y-1/2 text-slate-500" size={16} />
                                        <input 
                                            type="text" 
                                            value={address}
                                            onChange={(e) => setAddress(e.target.value)}
                                            placeholder="Enter 0x... or Solana address"
                                            className="w-full pl-10 pr-4 py-2 bg-[#0b0f19] border border-white/5 rounded-xl text-sm focus:outline-none focus:border-primary-500 text-slate-200 font-mono"
                                        />
                                    </div>
                                </div>

                                {error && (
                                    <div className="p-3 bg-rose-500/10 border border-rose-500/20 rounded-xl flex gap-3 text-rose-400 text-sm">
                                        <AlertCircle size={18} className="shrink-0" />
                                        {error}
                                    </div>
                                )}

                                <div className="p-4 bg-amber-500/10 border border-amber-500/20 rounded-xl flex gap-4 shadow-glow">
                                    <Zap className="text-amber-400 shrink-0" size={24} />
                                    <div>
                                        <h4 className="text-sm font-bold text-amber-300">Platform Audit Rule</h4>
                                        <p className="text-xs text-amber-400 mt-1 leading-relaxed">
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
                                    <div className="p-4 bg-[#0b0f19]/30 rounded-xl border border-white/5">
                                        <div className="flex items-center gap-2 text-slate-400 mb-1">
                                            <Globe size={14} />
                                            <span className="text-[10px] font-bold uppercase">RPC Balance</span>
                                        </div>
                                        <div className="text-lg font-bold text-slate-200">
                                            {report.wallet_reconciliation?.actual_rpc_balance} {report.blockchain}
                                        </div>
                                    </div>
                                    <div className="p-4 bg-[#0b0f19]/30 rounded-xl border border-white/5">
                                        <div className="flex items-center gap-2 text-slate-400 mb-1">
                                            <Database size={14} />
                                            <span className="text-[10px] font-bold uppercase">Expected</span>
                                        </div>
                                        <div className="text-lg font-bold text-slate-200">
                                            {report.wallet_reconciliation?.expected_onchain_balance}
                                        </div>
                                    </div>
                                    <div className="p-4 rounded-xl border border-white/5 flex flex-col justify-center items-center gap-1 bg-[#0b0f19]/30">
                                        {report.wallet_reconciliation?.onchain_out_of_sync ? (
                                            <>
                                                <div className="p-1 bg-rose-500/10 text-rose-400 rounded-full border border-rose-500/20">
                                                    <ShieldAlert size={16} />
                                                </div>
                                                <span className="text-[10px] font-bold text-rose-400 uppercase">Out of Sync</span>
                                            </>
                                        ) : (
                                            <>
                                                <div className="p-1 bg-emerald-500/10 text-emerald-400 rounded-full border border-emerald-500/20">
                                                    <CheckCircle2 size={16} />
                                                </div>
                                                <span className="text-[10px] font-bold text-emerald-400 uppercase">Balanced</span>
                                            </>
                                        )}
                                    </div>
                                </div>

                                {/* Detailed Stats */}
                                <div className="space-y-3">
                                    <h4 className="text-xs font-bold text-slate-400 uppercase tracking-widest">Audit Details</h4>
                                    <div className="bg-[#0b0f19]/30 rounded-xl border border-white/5 divide-y divide-white/5">
                                        <div className="flex items-center justify-between p-3">
                                            <span className="text-sm text-slate-400">On-chain Deposits (Raw)</span>
                                            <span className="text-sm font-bold text-slate-200">{report.audit_summary?.onchain_deposits_raw}</span>
                                        </div>
                                        <div className="flex items-center justify-between p-3">
                                            <span className="text-sm text-slate-400">On-chain Withdrawals (Raw)</span>
                                            <span className="text-sm font-bold text-slate-200">{report.audit_summary?.onchain_withdrawals_raw}</span>
                                        </div>
                                        <div className="flex items-center justify-between p-3">
                                            <span className="text-sm text-slate-400">Missing in DB (Rectified)</span>
                                            <span className="text-sm font-bold text-amber-400">+{report.audit_summary?.missing_in_db_rectified}</span>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        )
                    ) : (
                        /* RE-VERIFY STATIC TX MODE */
                        <div className="space-y-4">
                            <div className="grid grid-cols-2 gap-4">
                                <div className="space-y-1.5">
                                    <label className="text-xs font-bold text-slate-400 uppercase">Transaction Type</label>
                                    <select 
                                        value={txType}
                                        onChange={(e) => setTxType(e.target.value as any)}
                                        className="w-full px-4 py-2 bg-[#0b0f19] border border-white/5 rounded-xl text-sm focus:outline-none focus:border-primary-500 text-slate-200 font-medium"
                                    >
                                        <option value="customer">Customer Deposit</option>
                                        <option value="merchant">Merchant Deposit</option>
                                    </select>
                                </div>
                                <div className="space-y-1.5">
                                    <label className="text-xs font-bold text-slate-400 uppercase">Crypto Asset / Network</label>
                                    <select 
                                        value={cryptoType}
                                        onChange={(e) => setCryptoType(e.target.value)}
                                        className="w-full px-4 py-2 bg-[#0b0f19] border border-white/5 rounded-xl text-sm focus:outline-none focus:border-primary-500 text-slate-200 font-medium"
                                    >
                                        <option value="SOL">Solana (SOL)</option>
                                        <option value="USDT_SPL">USDT (SPL)</option>
                                        <option value="BNB">Binance Smart Chain (BNB)</option>
                                        <option value="USDT_BEP20">USDT (BEP20)</option>
                                        <option value="ETH">Ethereum (ETH)</option>
                                        <option value="MATIC">Polygon (MATIC)</option>
                                    </select>
                                </div>
                            </div>

                            <div className="grid grid-cols-3 gap-4">
                                <div className="col-span-2 space-y-1.5">
                                    <label className="text-xs font-bold text-slate-400 uppercase">Associated DB ID (Customer / Merchant ID)</label>
                                    <input 
                                        type="number" 
                                        required
                                        value={associatedId}
                                        onChange={(e) => setAssociatedId(e.target.value)}
                                        placeholder="e.g. 1"
                                        className="w-full px-4 py-2 bg-[#0b0f19] border border-white/5 rounded-xl text-sm focus:outline-none focus:border-primary-500 text-slate-200"
                                    />
                                </div>
                                <div className="space-y-1.5">
                                    <label className="text-xs font-bold text-slate-400 uppercase">Environment</label>
                                    <div className="flex p-1 bg-[#0b0f19] rounded-xl border border-white/5">
                                        <button 
                                            type="button"
                                            onClick={() => setSandboxMode(false)}
                                            className={`flex-1 py-1.5 text-xs font-bold rounded-lg transition-all ${!sandboxMode ? 'bg-primary-600 text-white shadow-glow' : 'text-slate-400 hover:text-slate-200'}`}
                                        >
                                            Live
                                        </button>
                                        <button 
                                            type="button"
                                            onClick={() => setSandboxMode(true)}
                                            className={`flex-1 py-1.5 text-xs font-bold rounded-lg transition-all ${sandboxMode ? 'bg-amber-600 text-white shadow-glow' : 'text-slate-400 hover:text-slate-200'}`}
                                        >
                                            Sandbox
                                        </button>
                                    </div>
                                </div>
                            </div>

                            <div className="space-y-1.5">
                                <label className="text-xs font-bold text-slate-400 uppercase">On-Chain Transaction Hash</label>
                                <input 
                                    type="text" 
                                    required
                                    value={txHash}
                                    onChange={(e) => setTxHash(e.target.value)}
                                    placeholder="Enter full transaction signature/hash"
                                    className="w-full px-4 py-2 bg-[#0b0f19] border border-white/5 rounded-xl text-sm focus:outline-none focus:border-primary-500 text-slate-200 font-mono"
                                />
                            </div>

                            {error && (
                                <div className="p-3 bg-rose-500/10 border border-rose-500/20 rounded-xl flex gap-3 text-rose-400 text-sm">
                                    <AlertCircle size={18} className="shrink-0" />
                                    {error}
                                </div>
                            )}

                            {successMessage && (
                                <div className="p-3 bg-emerald-500/10 border border-emerald-500/20 rounded-xl flex gap-3 text-emerald-400 text-sm">
                                    <CheckCircle2 size={18} className="shrink-0" />
                                    {successMessage}
                                </div>
                            )}
                        </div>
                    )}
                </div>

                {/* Footer */}
                <div className="px-6 py-4 border-t border-white/5 bg-[#1a2336]/40 flex items-center justify-between">
                    <button 
                        onClick={report || successMessage ? () => { setReport(null); setSuccessMessage(null); } : onClose}
                        className="px-4 py-2 text-sm font-semibold text-slate-400 hover:text-slate-200 transition-colors"
                    >
                        {report || successMessage ? 'Start New Audit' : 'Cancel'}
                    </button>
                    {!report && !successMessage && (
                        mode === 'reconcile' ? (
                            <button 
                                onClick={handleRectify}
                                disabled={loading || !address}
                                className="flex items-center gap-2 px-6 py-2.5 bg-primary-600 text-white rounded-xl text-sm font-bold hover:bg-primary-500 transition-all shadow-glow disabled:opacity-30 disabled:shadow-none active:scale-95"
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
                        ) : (
                            <button 
                                onClick={handleReverify}
                                disabled={loading || !txHash || !associatedId}
                                className="flex items-center gap-2 px-6 py-2.5 bg-emerald-600 text-white rounded-xl text-sm font-bold hover:bg-emerald-500 transition-all shadow-glow disabled:opacity-30 disabled:shadow-none active:scale-95"
                            >
                                {loading ? (
                                    <>
                                        <Loader2 size={18} className="animate-spin" />
                                        Verifying...
                                    </>
                                ) : (
                                    <>
                                        <RefreshCw size={18} />
                                        Re-Verify On-Chain
                                    </>
                                )}
                            </button>
                        )
                    )}
                </div>
            </div>
        </div>
    );
};

export default RectifyModal;
