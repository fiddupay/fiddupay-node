import React from 'react';
import { Zap } from 'lucide-react';

interface ManualSweepTabProps {
    networks: string[];
    triggering: string | null;
    onSweep: (network: string) => void;
}

const ManualSweepTab: React.FC<ManualSweepTabProps> = ({ networks, triggering, onSweep }) => {
    return (
        <div className="p-6 space-y-6 animate-in fade-in duration-500 bg-[#151c2c]">
            <div>
                <h2 className="text-lg font-bold text-slate-200 leading-tight">Manual Fee Execution</h2>
                <p className="text-sm text-slate-400 mt-2 max-w-2xl">
                    You can bypass the scheduled minimums and immediately attempt to sweep all eligible merchant custody wallets on a chosen network into the platform central treasury.
                </p>
                <div className="mt-3 p-3 bg-rose-500/10 border border-rose-500/20 rounded-xl text-xs text-rose-400 font-semibold shadow-glowGreen">
                    Use this ONLY if you have been alerted to exceptionally low network gas prices.
                </div>
            </div>

            <div className="grid grid-cols-2 md:grid-cols-5 gap-4">
                {networks.map(net => (
                    <button
                        key={net}
                        onClick={() => onSweep(net)}
                        disabled={triggering !== null}
                        className="flex flex-col items-center justify-center p-6 bg-[#0b0f19] border border-white/5 rounded-2xl hover:border-primary-500/30 hover:bg-white/5 transition-all hover:-translate-y-1 shadow-sm group disabled:opacity-50"
                    >
                        <div className={`p-4 rounded-full mb-3 transition-colors ${
                            triggering === net ? 'bg-amber-500/20 text-amber-400' : 'bg-white/5 text-slate-500 group-hover:bg-amber-500/20 group-hover:text-amber-400'
                        }`}>
                            <Zap size={24} className={triggering === net ? 'animate-pulse' : ''} />
                        </div>
                        <span className="text-sm font-bold text-slate-200">{net}</span>
                        {triggering === net ? (
                             <span className="text-[10px] font-bold text-amber-400 mt-2 uppercase tracking-widest animate-pulse">Running...</span>
                        ) : (
                             <span className="text-[10px] font-bold text-slate-500 mt-2 uppercase tracking-widest opacity-0 group-hover:opacity-100 transition-opacity">Sweep now</span>
                        )}
                    </button>
                ))}
            </div>
        </div>
    );
};

export default ManualSweepTab;
