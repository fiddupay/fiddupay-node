import React, { useEffect, useState } from 'react';
import { Settings as SettingsIcon, Save, Zap, AlertTriangle } from 'lucide-react';
import { adminAPI } from '../lib/api';

const NETWORKS = ["ETHEREUM", "BSC", "POLYGON", "ARBITRUM", "SOLANA"];

const Settings: React.FC = () => {
    const [loading, setLoading] = useState(false);
    const [saving, setSaving] = useState(false);
    const [triggering, setTriggering] = useState<string | null>(null);

    const [settings, setSettings] = useState({
        is_auto_sweep_enabled: false,
        min_accumulated_usd: '50.00',
        schedule_cron: '0 0 * * *',
        discord_webhook_url: '',
        gas_alert_threshold_gwei: '20.00',
        gas_alert_threshold_lamports: '5000',
    });

    useEffect(() => {
        fetchSettings();
    }, []);

    const fetchSettings = async () => {
        try {
            setLoading(true);
            const res = await adminAPI.getFeeSweepSettings();
            if (res.data) {
                setSettings({
                    is_auto_sweep_enabled: res.data.is_auto_sweep_enabled ?? false,
                    min_accumulated_usd: res.data.min_accumulated_usd ?? '50.00',
                    schedule_cron: res.data.schedule_cron ?? '0 0 * * *',
                    discord_webhook_url: res.data.discord_webhook_url ?? '',
                    gas_alert_threshold_gwei: res.data.gas_alert_threshold_gwei ?? '20.00',
                    gas_alert_threshold_lamports: res.data.gas_alert_threshold_lamports?.toString() ?? '5000',
                });
            }
        } catch (err) {
            console.error('Failed to fetch settings', err);
        } finally {
            setLoading(false);
        }
    };

    const handleChange = (e: React.ChangeEvent<HTMLInputElement | HTMLSelectElement>) => {
        const { name, value, type } = e.target;
        const checked = (e.target as HTMLInputElement).checked;

        setSettings((prev) => ({
            ...prev,
            [name]: type === 'checkbox' ? checked : value,
        }));
    };

    const handleSave = async (e: React.FormEvent) => {
        e.preventDefault();
        try {
            setSaving(true);
            await adminAPI.updateFeeSweepSettings({
                ...settings,
                gas_alert_threshold_lamports: parseInt(settings.gas_alert_threshold_lamports, 10)
            });
            alert('Settings saved successfully!');
        } catch (err) {
            console.error('Failed to save', err);
            alert('Failed to save settings');
        } finally {
            setSaving(false);
        }
    };

    const handleManualSweep = async (network: string) => {
        if (!window.confirm(`Trigger manual sweep for ${network}?`)) return;
        try {
            setTriggering(network);
            const res = await adminAPI.triggerManualSweep(network);
            alert(`Sweep completed! Result: ${JSON.stringify(res.data)}`);
        } catch (err) {
            console.error('Sweep failed', err);
            alert(`Manual sweep failed for ${network}`);
        } finally {
            setTriggering(null);
        }
    };

    if (loading) return <div>Loading settings...</div>;

    return (
        <div className="max-w-4xl mx-auto space-y-8 pb-12">
            <div>
                <h1 className="text-2xl font-bold text-slate-900 flex items-center gap-2">
                    <SettingsIcon className="w-6 h-6" />
                    System Settings
                </h1>
                <p className="mt-1 text-sm text-slate-500">
                    Manage platform configurations, including smart fee sweeping and gas alerts.
                </p>
            </div>

            <div className="bg-surface rounded-xl shadow-sm border border-slate-200 overflow-hidden">
                <form onSubmit={handleSave} className="divide-y divide-slate-200">
                    <div className="p-6 space-y-6">
                        <div>
                            <h2 className="text-lg font-semibold text-slate-900">Smart Fee Sweeping</h2>
                            <p className="text-sm text-slate-500">Configure thresholds for automated fee collection to save on gas costs.</p>
                        </div>

                        <div className="flex items-center justify-between p-4 bg-slate-50 rounded-lg border border-slate-100">
                            <div className="flex flex-col">
                                <span className="font-medium text-slate-900">Enable Automated Sweeping</span>
                                <span className="text-sm text-slate-500">When enabled, platform fees will be batched and swept periodically.</span>
                            </div>
                            <label className="relative inline-flex items-center cursor-pointer">
                                <input
                                    type="checkbox"
                                    name="is_auto_sweep_enabled"
                                    checked={settings.is_auto_sweep_enabled}
                                    onChange={handleChange}
                                    className="sr-only peer"
                                />
                                <div className="w-11 h-6 bg-slate-200 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-primary-100 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-slate-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-primary-500"></div>
                            </label>
                        </div>

                        <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                            <div className="space-y-1">
                                <label className="text-sm font-medium text-slate-700">Minimum Accumulated Balance (USD)</label>
                                <div className="relative">
                                    <div className="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
                                        <span className="text-slate-500 sm:text-sm">$</span>
                                    </div>
                                    <input
                                        type="number"
                                        step="0.01"
                                        name="min_accumulated_usd"
                                        value={settings.min_accumulated_usd}
                                        onChange={handleChange}
                                        className="block w-full pl-7 pr-12 sm:text-sm border-slate-300 rounded-md focus:ring-primary-500 focus:border-primary-500 border p-2"
                                        placeholder="50.00"
                                    />
                                    <div className="absolute inset-y-0 right-0 pr-3 flex items-center pointer-events-none">
                                        <span className="text-slate-500 sm:text-sm">USD</span>
                                    </div>
                                </div>
                                <p className="text-xs text-slate-500">Sweep triggers when wallet hits this value equivalent.</p>
                            </div>

                            <div className="space-y-1">
                                <label className="text-sm font-medium text-slate-700">Batch Collection Schedule</label>
                                <input
                                    type="text"
                                    name="schedule_cron"
                                    value={settings.schedule_cron}
                                    onChange={handleChange}
                                    className="block w-full sm:text-sm border-slate-300 rounded-md focus:ring-primary-500 focus:border-primary-500 border p-2 text-slate-900"
                                    placeholder="0 0 * * *"
                                />
                                <p className="text-xs text-slate-500">CRON expression. Default is every midnight (UTC).</p>
                            </div>
                        </div>
                    </div>

                    <div className="p-6 space-y-6">
                        <div>
                            <h2 className="text-lg font-semibold text-slate-900">Gas Monitoring & Alerts</h2>
                            <p className="text-sm text-slate-500">Configure Webhook endpoints and low-fee thresholds to be notified of optimal clearing times.</p>
                        </div>

                        <div className="space-y-4">
                            <div className="space-y-1">
                                <label className="text-sm font-medium text-slate-700">Discord/Slack Webhook URL</label>
                                <input
                                    type="url"
                                    name="discord_webhook_url"
                                    value={settings.discord_webhook_url}
                                    onChange={handleChange}
                                    className="block w-full sm:text-sm border-slate-300 rounded-md focus:ring-primary-500 focus:border-primary-500 border p-2"
                                    placeholder="https://discord.com/api/webhooks/..."
                                />
                            </div>

                            <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                                <div className="space-y-1">
                                    <label className="text-sm font-medium text-slate-700">EVM Gas Threshold (Gwei)</label>
                                    <input
                                        type="number"
                                        step="0.01"
                                        name="gas_alert_threshold_gwei"
                                        value={settings.gas_alert_threshold_gwei}
                                        onChange={handleChange}
                                        className="block w-full sm:text-sm border-slate-300 rounded-md focus:ring-primary-500 focus:border-primary-500 border p-2"
                                        placeholder="20.00"
                                    />
                                    <p className="text-xs text-slate-500">Alert triggers if network gas falls below this Gwei value.</p>
                                </div>

                                <div className="space-y-1">
                                    <label className="text-sm font-medium text-slate-700">Solana Threshold (Lamports)</label>
                                    <input
                                        type="number"
                                        name="gas_alert_threshold_lamports"
                                        value={settings.gas_alert_threshold_lamports}
                                        onChange={handleChange}
                                        className="block w-full sm:text-sm border-slate-300 rounded-md focus:ring-primary-500 focus:border-primary-500 border p-2"
                                        placeholder="5000"
                                    />
                                    <p className="text-xs text-slate-500">Alert triggers if base fee multiplier falls below this.</p>
                                </div>
                            </div>
                        </div>
                    </div>

                    <div className="p-6 bg-slate-50 flex items-center justify-end">
                        <button
                            type="submit"
                            disabled={saving}
                            className="inline-flex items-center justify-center px-4 py-2 border border-transparent text-sm font-medium rounded-md shadow-sm text-white bg-primary-600 hover:bg-primary-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-primary-500 disabled:opacity-50"
                        >
                            <Save className="w-4 h-4 mr-2" />
                            {saving ? 'Saving...' : 'Save Settings'}
                        </button>
                    </div>
                </form>
            </div>

            <div className="bg-white rounded-xl shadow-sm border border-red-100 overflow-hidden">
                <div className="p-6">
                    <div className="flex items-center gap-2 text-red-700 mb-4">
                        <AlertTriangle className="w-5 h-5" />
                        <h2 className="text-lg font-semibold">Manual Fee Execution</h2>
                    </div>
                    <p className="text-sm text-slate-600 mb-6 max-w-2xl">
                        You can bypass the scheduled minimums and immediately attempt to sweep all eligible merchant custody wallets on a chosen network into the platform central treasury. Use this if you have been alerted to exceptionally low network gas prices.
                    </p>

                    <div className="grid grid-cols-2 md:grid-cols-5 gap-4">
                        {NETWORKS.map(net => (
                            <button
                                key={net}
                                onClick={() => handleManualSweep(net)}
                                disabled={triggering !== null}
                                className="inline-flex flex-col items-center justify-center p-4 border rounded-lg hover:bg-slate-50 hover:border-slate-300 transition-all hover:scale-105 bg-white group disabled:opacity-50 shadow-sm hover:shadow-md"
                            >
                                <Zap className={`w-5 h-5 mb-2 ${triggering === net ? 'text-orange-500 animate-pulse' : 'text-slate-400 group-hover:text-amber-500'}`} />
                                <span className="text-xs font-semibold text-slate-700">{net}</span>
                                {triggering === net && <span className="text-[10px] text-orange-500 mt-1">Sweeping...</span>}
                            </button>
                        ))}
                    </div>
                </div>
            </div>
        </div>
    );
};

export default Settings;
