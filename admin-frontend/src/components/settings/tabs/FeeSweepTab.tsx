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
        <div className="p-6 space-y-6 animate-in fade-in duration-500">
            <div>
                <h2 className="text-lg font-bold text-slate-900 leading-tight">Smart Fee Sweeping</h2>
                <p className="text-sm text-slate-500 mt-1">Configure thresholds for automated fee collection to save on gas costs.</p>
            </div>

            <div className="flex items-center justify-between p-4 bg-slate-50 rounded-xl border border-slate-100">
                <div className="flex flex-col">
                    <span className="font-bold text-slate-900">Enable Automated Sweeping</span>
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
                    <div className="w-11 h-6 bg-slate-200 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-primary-100 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-slate-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-primary-500"></div>
                </label>
            </div>

            <div className="grid grid-cols-1 md:grid-cols-2 gap-8">
                <div className="space-y-2">
                    <label className="text-sm font-bold text-slate-700">Minimum Accumulated Balance (USD)</label>
                    <div className="relative">
                        <div className="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
                            <span className="text-slate-400 font-bold">$</span>
                        </div>
                        <input
                            type="number"
                            step="0.01"
                            name="min_accumulated_usd"
                            value={settings.min_accumulated_usd}
                            onChange={onChange}
                            className="block w-full pl-8 pr-12 py-2 bg-white border border-slate-200 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-primary-500/20 focus:border-primary-500 transition-all font-medium"
                            placeholder="50.00"
                        />
                        <div className="absolute inset-y-0 right-0 pr-3 flex items-center pointer-events-none">
                            <span className="text-slate-400 text-xs font-bold uppercase">USD</span>
                        </div>
                    </div>
                    <p className="text-[10px] text-slate-500 italic">Sweep triggers when wallet hits this value equivalent.</p>
                </div>

                <div className="space-y-2">
                    <label className="text-sm font-bold text-slate-700">Batch Collection Schedule</label>
                    <input
                        type="text"
                        name="schedule_cron"
                        value={settings.schedule_cron}
                        onChange={onChange}
                        className="block w-full px-4 py-2 bg-white border border-slate-200 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-primary-500/20 focus:border-primary-500 transition-all font-mono"
                        placeholder="0 0 * * *"
                    />
                    <p className="text-[10px] text-slate-500 italic">CRON expression. Default is every midnight (UTC).</p>
                </div>
            </div>
        </div>
    );
};

export default FeeSweepTab;
