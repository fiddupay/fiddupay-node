import React from 'react';
import { MdVpnKey, MdRefresh, MdContentCopy, MdLanguage, MdSecurity, MdAdd, MdDelete } from 'react-icons/md';

interface ApiSettingsTabProps {
    user: any;
    apiKey: string;
    showApiKey: boolean;
    setShowApiKey: (show: boolean) => void;
    handleRotateKey: () => Promise<void>;
    copyToClipboard: (text: string, label: string) => void;
    redirectUrl: string;
    setRedirectUrl: (url: string) => void;
    handleUpdateRedirect: () => Promise<void>;
    ipWhitelist: string[];
    newIp: string;
    setNewIp: (ip: string) => void;
    handleAddIp: () => Promise<void>;
    handleRemoveIp: (ip: string) => Promise<void>;
    loading: boolean;
    styles: any;
}

const ApiSettingsTab: React.FC<ApiSettingsTabProps> = ({
    user,
    apiKey,
    showApiKey,
    setShowApiKey,
    handleRotateKey,
    copyToClipboard,
    redirectUrl,
    setRedirectUrl,
    handleUpdateRedirect,
    ipWhitelist,
    newIp,
    setNewIp,
    handleAddIp,
    handleRemoveIp,
    loading,
    styles
}) => {
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
                        <div key={ip} className={styles.ipTag} style={{
                            display: 'flex',
                            alignItems: 'center',
                            gap: '8px',
                            background: '#f1f5f9',
                            padding: '6px 12px',
                            borderRadius: '6px',
                            fontSize: '13px'
                        }}>
                            <code>{ip}</code>
                            <MdDelete 
                                style={{ cursor: 'pointer', color: '#ef4444' }} 
                                onClick={() => handleRemoveIp(ip)}
                            />
                        </div>
                    ))}
                    {ipWhitelist.length === 0 && <span style={{ color: '#94a3b8', fontSize: '13px' }}>No IP restrictions set.</span>}
                </div>
            </div>
        </section>
    );
};

export default ApiSettingsTab;
