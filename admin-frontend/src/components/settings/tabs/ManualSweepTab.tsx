import React from 'react';
import { Zap } from 'lucide-react';

interface ManualSweepTabProps {
    networks: string[];
    triggering: string | null;
    onSweep: (network: string) => void;
}

const ManualSweepTab: React.FC<ManualSweepTabProps> = ({ networks, triggering, onSweep }) => {
    return (
        <div className="p-6 space-y-6 animate-in fade-in duration-500">
            <div>
                <h2 className="text-lg font-bold text-slate-900 leading-tight">Manual Fee Execution</h2>
                <p className="text-sm text-slate-600 mt-2 max-w-2xl">
                    You can bypass the scheduled minimums and immediately attempt to sweep all eligible merchant custody wallets on a chosen network into the platform central treasury.
                </p>
                <div className="mt-3 p-3 bg-rose-50 border border-rose-100 rounded-lg text-xs text-rose-600 font-medium">
                    Use this ONLY if you have been alerted to exceptionally low network gas prices.
                </div>
            </div>

            <div className="grid grid-cols-2 md:grid-cols-5 gap-4">
                {networks.map(net => (
                    <button
                        key={net}
                        onClick={() => onSweep(net)}
                        disabled={triggering !== null}
                        className="flex flex-col items-center justify-center p-6 bg-white border border-slate-200 rounded-2xl hover:border-primary-300 hover:bg-slate-50 transition-all hover:-translate-y-1 shadow-sm group disabled:opacity-50"
                    >
                        <div className={`p-4 rounded-full mb-3 transition-colors ${
                            triggering === net ? 'bg-amber-100 text-amber-600' : 'bg-slate-100 text-slate-400 group-hover:bg-amber-50 group-hover:text-amber-500'
                        }`}>
                            <Zap size={24} className={triggering === net ? 'animate-pulse' : ''} />
                        </div>
                        <span className="text-sm font-bold text-slate-900">{net}</span>
                        {triggering === net ? (
                             <span className="text-[10px] font-bold text-amber-600 mt-2 uppercase tracking-widest animate-pulse">Running...</span>
                        ) : (
                             <span className="text-[10px] font-bold text-slate-400 mt-2 uppercase tracking-widest opacity-0 group-hover:opacity-100 transition-opacity">Sweep now</span>
                        )}
                    </button>
                ))}
            </div>
        </div>
    );
};

export default ManualSweepTab;
