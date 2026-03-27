import React, { useEffect, useState } from 'react';
import { Settings as SettingsIcon, Save, Zap, Bell, Activity } from 'lucide-react';
import { adminAPI } from '../lib/api';
import FeeSweepTab from '../components/settings/tabs/FeeSweepTab';
import GasAlertsTab from '../components/settings/tabs/GasAlertsTab';
import ManualSweepTab from '../components/settings/tabs/ManualSweepTab';

const NETWORKS = ["ETHEREUM", "BSC", "POLYGON", "ARBITRUM", "SOLANA"];

const Settings: React.FC = () => {
    const [loading, setLoading] = useState(false);
    const [saving, setSaving] = useState(false);
    const [triggering, setTriggering] = useState<string | null>(null);
    const [activeTab, setActiveTab] = useState<'sweep' | 'gas' | 'manual'>('sweep');

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

    if (loading) return (
        <div className="flex flex-col items-center justify-center min-h-[400px] text-slate-400 space-y-4">
            <SettingsIcon className="w-12 h-12 animate-spin" />
            <span className="text-sm font-medium">Loading platform configurations...</span>
        </div>
    );

    const tabs = [
        { id: 'sweep', label: 'Fee Sweeping', icon: Zap },
        { id: 'gas', label: 'Gas Alerts', icon: Bell },
        { id: 'manual', label: 'Manual Sweep', icon: Activity },
    ];

    return (
        <div className="max-w-5xl mx-auto space-y-8 animate-in fade-in duration-700">
            <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
                <div>
                    <h1 className="text-2xl font-bold text-slate-900 tracking-tight flex items-center gap-2">
                        System Configuration
                    </h1>
                    <p className="text-slate-500 text-sm mt-1">Manage global platform behaviors, monitoring thresholds, and manual execution.</p>
                </div>
                {activeTab !== 'manual' && (
                    <button
                        onClick={handleSave}
                        disabled={saving}
                        className="flex items-center gap-2 px-6 py-2 bg-primary-600 text-white rounded-xl text-sm font-bold hover:bg-primary-700 transition-all shadow-md active:scale-95 disabled:opacity-50"
                    >
                        <Save size={18} />
                        {saving ? 'Saving...' : 'Save Changes'}
                    </button>
                )}
            </div>

            <div className="bg-white rounded-3xl border border-slate-200 shadow-xl overflow-hidden min-h-[500px] flex flex-col md:flex-row">
                <div className="w-full md:w-64 bg-slate-50 border-b md:border-b-0 md:border-r border-slate-200 p-4 space-y-1">
                    {tabs.map((tab) => (
                        <button
                            key={tab.id}
                            onClick={() => setActiveTab(tab.id as any)}
                            className={`w-full flex items-center gap-3 px-4 py-3 rounded-xl text-sm font-bold transition-all ${
                                activeTab === tab.id 
                                ? 'bg-white text-primary-600 shadow-md ring-1 ring-slate-200' 
                                : 'text-slate-500 hover:bg-slate-100 hover:text-slate-700'
                            }`}
                        >
                            <tab.icon size={18} />
                            {tab.label}
                        </button>
                    ))}
                </div>

                <div className="flex-1 min-h-[400px]">
                    <form onSubmit={handleSave}>
                        {activeTab === 'sweep' && (
                            <FeeSweepTab settings={settings} onChange={handleChange} />
                        )}
                        {activeTab === 'gas' && (
                            <GasAlertsTab settings={settings} onChange={handleChange} />
                        )}
                        {activeTab === 'manual' && (
                            <ManualSweepTab 
                                networks={NETWORKS} 
                                triggering={triggering} 
                                onSweep={handleManualSweep} 
                            />
                        )}
                    </form>
                </div>
            </div>
        </div>
    );
};

export default Settings;
