import React from 'react';

interface FeeSweepTabProps {
    settings: {
        is_auto_sweep_enabled: boolean;
        min_accumulated_usd: string;
        schedule_cron: string;
    };
    onChange: (e: React.ChangeEvent<HTMLInputElement | HTMLSelectElement>) => void;
}

const FeeSweepTab: React.FC<FeeSweepTabProps> = ({ settings, onChange }) => {
    return (
        <div className="p-6 space-y-6 animate-in fade-in duration-500 bg-[#151c2c]">
            <div>
                <h2 className="text-lg font-bold text-slate-200 leading-tight">Smart Fee Sweeping</h2>
                <p className="text-sm text-slate-400 mt-1">Configure thresholds for automated fee collection to save on gas costs.</p>
            </div>

            <div className="flex items-center justify-between p-4 bg-[#0b0f19]/30 rounded-xl border border-white/5">
                <div className="flex flex-col">
                    <span className="font-bold text-slate-200">Enable Automated Sweeping</span>
                    <span className="text-xs text-slate-500 mt-0.5">When enabled, platform fees will be batched and swept periodically.</span>
                </div>
                <label className="relative inline-flex items-center cursor-pointer">
                    <input
                        type="checkbox"
                        name="is_auto_sweep_enabled"
                        checked={settings.is_auto_sweep_enabled}
                        onChange={onChange}
                        className="sr-only peer"
                    />
                    <div className="w-11 h-6 bg-slate-800 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-slate-400 after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-primary-600 peer-checked:after:bg-white"></div>
                </label>
            </div>

            <div className="grid grid-cols-1 md:grid-cols-2 gap-8">
                <div className="space-y-2">
                    <label className="text-sm font-bold text-slate-400">Minimum Accumulated Balance (USD)</label>
                    <div className="relative">
                        <div className="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
                            <span className="text-slate-500 font-bold">$</span>
                        </div>
                        <input
                            type="number"
                            step="0.01"
                            name="min_accumulated_usd"
                            value={settings.min_accumulated_usd}
                            onChange={onChange}
                            className="block w-full pl-8 pr-12 py-2 bg-[#0b0f19] border border-white/5 rounded-xl text-sm focus:outline-none focus:border-primary-500 transition-all text-slate-200 font-medium"
                            placeholder="50.00"
                        />
                        <div className="absolute inset-y-0 right-0 pr-3 flex items-center pointer-events-none">
                            <span className="text-slate-500 text-xs font-bold uppercase">USD</span>
                        </div>
                    </div>
                    <p className="text-[10px] text-slate-500 italic font-semibold">Sweep triggers when wallet hits this value equivalent.</p>
                </div>

                <div className="space-y-2">
                    <label className="text-sm font-bold text-slate-400">Batch Collection Schedule</label>
                    <input
                        type="text"
                        name="schedule_cron"
                        value={settings.schedule_cron}
                        onChange={onChange}
                        className="block w-full px-4 py-2 bg-[#0b0f19] border border-white/5 rounded-xl text-sm focus:outline-none focus:border-primary-500 transition-all text-slate-200 font-mono"
                        placeholder="0 0 * * *"
                    />
                    <p className="text-[10px] text-slate-500 italic font-semibold">CRON expression. Default is every midnight (UTC).</p>
                </div>
            </div>
        </div>
    );
};

export default FeeSweepTab;
