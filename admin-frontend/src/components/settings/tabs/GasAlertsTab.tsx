import React from 'react';

interface GasAlertsTabProps {
    settings: {
        discord_webhook_url: string;
        gas_alert_threshold_gwei: string;
        gas_alert_threshold_lamports: string;
    };
    onChange: (e: React.ChangeEvent<HTMLInputElement | HTMLSelectElement>) => void;
}

const GasAlertsTab: React.FC<GasAlertsTabProps> = ({ settings, onChange }) => {
    return (
        <div className="p-6 space-y-6 animate-in fade-in duration-500">
            <div>
                <h2 className="text-lg font-bold text-slate-900 leading-tight">Gas Monitoring & Alerts</h2>
                <p className="text-sm text-slate-500 mt-1">Configure Webhook endpoints and low-fee thresholds to be notified of optimal clearing times.</p>
            </div>

            <div className="space-y-4">
                <div className="space-y-2">
                    <label className="text-sm font-bold text-slate-700">Discord/Slack Webhook URL</label>
                    <input
                        type="url"
                        name="discord_webhook_url"
                        value={settings.discord_webhook_url}
                        onChange={onChange}
                        className="block w-full px-4 py-2 bg-white border border-slate-200 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-primary-500/20 focus:border-primary-500 transition-all"
                        placeholder="https://discord.com/api/webhooks/..."
                    />
                </div>

                <div className="grid grid-cols-1 md:grid-cols-2 gap-8">
                    <div className="space-y-2">
                        <label className="text-sm font-bold text-slate-700">EVM Gas Threshold (Gwei)</label>
                        <input
                            type="number"
                            step="0.01"
                            name="gas_alert_threshold_gwei"
                            value={settings.gas_alert_threshold_gwei}
                            onChange={onChange}
                            className="block w-full px-4 py-2 bg-white border border-slate-200 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-primary-500/20 focus:border-primary-500 transition-all"
                            placeholder="20.00"
                        />
                        <p className="text-[10px] text-slate-500 italic">Alert triggers if network gas falls below this Gwei value.</p>
                    </div>

                    <div className="space-y-2">
                        <label className="text-sm font-bold text-slate-700">Solana Threshold (Lamports)</label>
                        <input
                            type="number"
                            name="gas_alert_threshold_lamports"
                            value={settings.gas_alert_threshold_lamports}
                            onChange={onChange}
                            className="block w-full px-4 py-2 bg-white border border-slate-200 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-primary-500/20 focus:border-primary-500 transition-all font-medium"
                            placeholder="5000"
                        />
                        <p className="text-[10px] text-slate-500 italic">Alert triggers if base fee multiplier falls below this.</p>
                    </div>
                </div>
            </div>
        </div>
    );
};

export default GasAlertsTab;
