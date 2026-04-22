import React, { useState, useEffect } from 'react';
import { MdVpnKey, MdRefresh, MdContentCopy, MdLanguage, MdSecurity, MdAdd, MdDelete, MdWarning, MdClose, MdError } from 'react-icons/md';
import { merchantAPI } from '@/services/apiService';
import { useToast } from '@/contexts/ToastContext';
import { useAuthStore } from '@/stores/authStore';

interface ApiSettingsTabProps {
    user: any;
    styles: any;
}

const ApiSettingsTab: React.FC<ApiSettingsTabProps> = ({
    user,
    styles
}) => {
    const { showToast } = useToast();
    const { loadUser } = useAuthStore();
    const [loading, setLoading] = useState(false);
    
    const [apiKey, setApiKey] = useState('');
    const [showApiKey, setShowApiKey] = useState(false);
    const [showRotateModal, setShowRotateModal] = useState(false);
    const [redirectUrl, setRedirectUrl] = useState('');
    const [ipWhitelist, setIpWhitelist] = useState<string[]>([]);
    const [newIp, setNewIp] = useState('');

    useEffect(() => {
        if (user) {
            const incomingKey = user.api_key || '';
            const isIncomingMasked = incomingKey.includes('********');
            const isCurrentMasked = apiKey.includes('********') || !apiKey;

            if ((!apiKey && incomingKey) || (isCurrentMasked && incomingKey) || (!isIncomingMasked && incomingKey !== apiKey)) {
                setApiKey(incomingKey);
            }

            setRedirectUrl(user.redirect_url || '');
            setIpWhitelist(user.ip_whitelist || []);
        }
    }, [user]);

    const copyToClipboard = (text: string, label: string) => {
        navigator.clipboard.writeText(text);
        showToast(`${label} copied to clipboard`, 'success');
    };

    const handleUpdateRedirect = async () => {
        try {
            setLoading(true);
            await merchantAPI.updateSettings({ redirect_url: redirectUrl });
            await loadUser(true);
            showToast('Redirect URL updated', 'success');
        } catch (error: any) {
            showToast('Failed to update redirect URL', 'error');
        } finally {
            setLoading(false);
        }
    };

    const handleAddIp = async () => {
        if (!newIp) return;
        if (ipWhitelist.includes(newIp)) {
            showToast('IP already in whitelist', 'warning');
            return;
        }
        const updated = [...ipWhitelist, newIp];
        try {
            setLoading(true);
            await merchantAPI.updateSettings({ ip_whitelist: updated });
            setIpWhitelist(updated);
            setNewIp('');
            await loadUser(true);
            showToast('IP added to whitelist', 'success');
        } catch (error: any) {
            showToast('Failed to update IP whitelist', 'error');
        } finally {
            setLoading(false);
        }
    };

    const handleRemoveIp = async (ip: string) => {
        const updated = ipWhitelist.filter(i => i !== ip);
        try {
            setLoading(true);
            await merchantAPI.updateSettings({ ip_whitelist: updated });
            setIpWhitelist(updated);
            await loadUser(true);
            showToast('IP removed from whitelist', 'success');
        } catch (error: any) {
            showToast('Failed to update IP whitelist', 'error');
        } finally {
            setLoading(false);
        }
    };

    const handleRotateKey = async () => {
        if (!user) return;

        if (!apiKey || apiKey === 'Not generated' || apiKey === 'No API key generated') {
            try {
                setLoading(true);
                const isLive = !user.sandbox_mode;
                if (isLive && user.kyc_tier === 0) {
                    showToast('Tier 1 Verification required to generate Live API keys', 'warning');
                    return;
                }
                const response = await merchantAPI.generateApiKey(isLive);
                setApiKey(response.data.api_key);
                await loadUser(true);
                showToast(`New ${user.sandbox_mode ? 'Sandbox' : 'Live'} API key generated`, 'success');
            } catch (error: any) {
                showToast(error.response?.data?.message || 'Failed to generate API key', 'error');
            } finally {
                setLoading(false);
            }
            return;
        }

        setShowRotateModal(true);
    };

    const confirmRotation = async () => {
        try {
            setLoading(true);
            const isLive = !user.sandbox_mode;
            if (isLive && user.kyc_tier === 0) {
                showToast('Tier 1 Verification required to rotate Live API keys', 'warning');
                setShowRotateModal(false);
                return;
            }
            const response = await merchantAPI.rotateApiKey(isLive);
            setApiKey(response.data.api_key);
            await loadUser(true);
            setShowRotateModal(false);
            showToast(`API key rotated successfully.`, 'success');
        } catch (error: any) {
            showToast(error.response?.data?.message || 'Failed to rotate API key', 'error');
        } finally {
            setLoading(false);
        }
    };

    return (
        <section className={styles.section}>
            <h2>API & Integration</h2>
            <p>Manage your API keys and integration settings.</p>

            <div className={styles.keyGrid}>
                <div className={styles.keyCard}>
                    <div className={styles.keyHeader}>
                        <h4><MdVpnKey /> {user?.sandbox_mode ? 'Sandbox' : 'Live'} API Key</h4>
                        <span className={user?.sandbox_mode ? styles.badgeSandbox : styles.badgeLive}>
                            {user?.sandbox_mode ? 'Sandbox' : 'Live'}
                        </span>
                    </div>
                    <div className={styles.keyInputGroup}>
                        <div className={styles.keyDisplay}>
                            {showApiKey ? apiKey : '•'.repeat(40)}
                        </div>
                        <button className={styles.viewBtn} onClick={() => setShowApiKey(!showApiKey)}>
                            {showApiKey ? 'Hide' : 'Show'}
                        </button>
                    </div>
                    <div className={styles.keyFooter}>
                        <div className={styles.keyNote}>
                            Keep your key secure. Never share it or use it in client-side code.
                        </div>
                        <div style={{ display: 'flex', gap: '8px' }}>
                            <button className={styles.copyBtn} onClick={() => copyToClipboard(apiKey, 'API Key')}>
                                <MdContentCopy /> Copy
                            </button>
                            <button className={styles.rotateBtn} onClick={handleRotateKey}>
                                <MdRefresh /> Rotate
                            </button>
                        </div>
                    </div>
                </div>
            </div>

            <div className={styles.redirectSection}>
                <div className={styles.redirectHeader}>
                    <h4><MdLanguage /> Redirect URL</h4>
                    <button 
                        className={styles.saveBtn} 
                        onClick={handleUpdateRedirect}
                        disabled={loading}
                    >
                        Save
                    </button>
                </div>
                <p className={styles.redirectNote}>
                    Customers will be redirected to this URL after completing a payment.
                </p>
                <div className={styles.inputWrapper}>
                    <input 
                        type="url" 
                        className={styles.urlInput}
                        placeholder="https://yourwebsite.com/callback"
                        value={redirectUrl}
                        onChange={(e) => setRedirectUrl(e.target.value)}
                    />
                </div>
            </div>

            <div className={styles.redirectSection}>
                <div className={styles.redirectHeader}>
                    <h4><MdSecurity /> IP Whitelist</h4>
                </div>
                <p className={styles.redirectNote}>
                    Restrict API access to these IP addresses. Leave empty for no restriction.
                </p>
                <div className={styles.inputWrapper} style={{ marginBottom: '16px' }}>
                    <input 
                        type="text" 
                        className={styles.urlInput}
                        placeholder="e.g. 192.168.1.1"
                        value={newIp}
                        onChange={(e) => setNewIp(e.target.value)}
                    />
                    <button 
                        className={styles.saveBtn}
                        onClick={handleAddIp}
                        disabled={loading || !newIp}
                    >
                        <MdAdd /> Add IP
                    </button>
                </div>
                
                <div className={styles.ipList} style={{ display: 'flex', flexWrap: 'wrap', gap: '8px' }}>
                    {ipWhitelist.map((ip) => (
                        <div key={ip} className={styles.ipTag}>
                            <code>{ip}</code>
                            <MdDelete 
                                style={{ cursor: 'pointer', color: '#f87171', fontSize: '16px' }} 
                                onClick={() => handleRemoveIp(ip)}
                            />
                        </div>
                    ))}
                    {ipWhitelist.length === 0 && <span style={{ color: 'var(--text-muted)', fontSize: '13px' }}>No IP restrictions set.</span>}
                </div>
            </div>

            {/* API Key Rotation Confirmation Modal */}
            {showRotateModal && (
                <div className={styles.modalOverlay}>
                    <div className={styles.modal}>
                        <div className={styles.modalHeader}>
                            <h2><MdWarning /> Confirm Key Rotation</h2>
                            <button
                                className={styles.closeBtn}
                                onClick={() => setShowRotateModal(false)}
                                disabled={loading}
                            >
                                <MdClose />
                            </button>
                        </div>
                        <div className={styles.modalBody}>
                            <p>
                                Are you sure you want to rotate your <strong>{user?.sandbox_mode ? 'Sandbox' : 'Live'}</strong> API key?
                                This is a destructive action that cannot be undone.
                            </p>
                            <div className={styles.warningBox}>
                                <MdError />
                                <p>
                                    Rotating your key will immediately invalidate the current one.
                                </p>
                            </div>
                        </div>
                        <div className={styles.modalActions}>
                            <button
                                className={styles.cancelBtn}
                                onClick={() => setShowRotateModal(false)}
                                disabled={loading}
                            >
                                Cancel
                            </button>
                            <button
                                className={styles.confirmRotateBtn}
                                onClick={confirmRotation}
                                disabled={loading}
                            >
                                {loading ? 'Rotating...' : 'Confirm Rotation'}
                            </button>
                        </div>
                    </div>
                </div>
            )}
        </section>
    );
};

export default ApiSettingsTab;
