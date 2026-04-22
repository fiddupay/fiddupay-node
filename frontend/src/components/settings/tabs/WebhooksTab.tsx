import React, { useState, useEffect } from 'react';
import { MdNotifications, MdSend, MdVpnKey, MdInfo, MdVisibility, MdVisibilityOff, MdRefresh } from 'react-icons/md';
import { merchantAPI } from '@/services/apiService';
import { useToast } from '@/contexts/ToastContext';
import { useAuthStore } from '@/stores/authStore';

interface WebhooksTabProps {
    user: any;
    styles: any;
}

const WebhooksTab: React.FC<WebhooksTabProps> = ({
    user,
    styles
}) => {
    const { showToast } = useToast();
    const { loadUser } = useAuthStore();
    const [loading, setLoading] = useState(false);
    
    const [webhookUrls, setWebhookUrls] = useState({
        standard: '',
        discord: '',
        slack: ''
    });
    const [webhookFormat, setWebhookFormat] = useState('standard');
    const [signingSecret, setSigningSecret] = useState('••••••••••••••••••••••••••••••••');
    const [showSecret, setShowSecret] = useState(false);
    const [showRotateSecretConfirm, setShowRotateSecretConfirm] = useState(false);

    useEffect(() => {
        if (user) {
            const format = user.webhook_format || 'standard';
            setWebhookUrls(prev => ({
                ...prev,
                [format]: user.webhook_url || ''
            }));
            setWebhookFormat(format);
            fetchSigningSecret();
        }
    }, [user]);

    const fetchSigningSecret = async () => {
        try {
            const settingsRes = await merchantAPI.getMerchantSettings();
            setSigningSecret(settingsRes.data.webhook_signing_secret || '••••••••••••••••••••••••••••••••');
        } catch (err) {
            console.error('Failed to fetch webhook secret', err);
        }
    };

    const handleUpdateWebhook = async (url: string) => {
        try {
            setLoading(true);
            await merchantAPI.updateSettings({
                webhook_url: url,
                webhook_format: webhookFormat
            });
            await loadUser(true);
            showToast('Webhook settings updated successfully', 'success');
        } catch (error: any) {
            showToast('Failed to update webhook settings', 'error');
        } finally {
            setLoading(false);
        }
    };

    const handleSendTestWebhook = async () => {
        try {
            setLoading(true);
            await merchantAPI.sendTestWebhook();
            showToast('Test webhook queued for delivery', 'success');
        } catch (error: any) {
            showToast('Failed to send test webhook', 'error');
        } finally {
            setLoading(false);
        }
    };

    const handleRotateSecret = async () => {
        if (!showRotateSecretConfirm) {
            setShowRotateSecretConfirm(true);
            showToast('Click rotate again to confirm. This will invalidate your current secret.', 'info');
            setTimeout(() => setShowRotateSecretConfirm(false), 5000);
            return;
        }

        try {
            setLoading(true);
            const response = await merchantAPI.updateSettings({ rotate_webhook_secret: true });
            const newSecret = response.data.new_webhook_secret;
            if (newSecret) {
                setSigningSecret(newSecret);
                setShowSecret(true);
                showToast('Webhook signing secret rotated', 'success');
            } else {
                await fetchSigningSecret();
                showToast('Webhook signing secret rotated', 'success');
            }
            setShowRotateSecretConfirm(false);
        } catch (error: any) {
            showToast('Failed to rotate signing secret', 'error');
        } finally {
            setLoading(false);
        }
    };

    return (
        <section className={styles.section}>
            <h2>Webhooks & Notifications</h2>
            <p>Get real-time updates when payments are confirmed.</p>

            <div className={styles.webhookLayout}>
                <div className={styles.webhookMain}>
                    <div className={styles.webhookConfig}>
                        <div className={styles.inputGroup}>
                            <label style={{ fontSize: '14px', fontWeight: 600, color: 'var(--text-main)', marginBottom: '8px', display: 'block' }}>
                                Webhook Payload Format
                            </label>
                            <div className={styles.formatSelector}>
                                <button 
                                    className={`${styles.formatBtn} ${webhookFormat === 'standard' ? styles.activeFormat : ''}`}
                                    onClick={() => setWebhookFormat('standard')}
                                >
                                    Standard JSON
                                </button>
                                <button 
                                    className={`${styles.formatBtn} ${webhookFormat === 'discord' ? styles.activeFormat : ''}`}
                                    onClick={() => setWebhookFormat('discord')}
                                >
                                    Discord
                                </button>
                                <button 
                                    className={`${styles.formatBtn} ${webhookFormat === 'slack' ? styles.activeFormat : ''}`}
                                    onClick={() => setWebhookFormat('slack')}
                                >
                                    Slack
                                </button>
                            </div>
                        </div>

                        <div className={styles.inputGroup}>
                            <div className={styles.redirectHeader}>
                                <h4><MdNotifications /> {webhookFormat === 'standard' ? 'Webhook Endpoint' : `${webhookFormat.charAt(0).toUpperCase() + webhookFormat.slice(1)} Webhook URL`}</h4>
                                <button className={styles.saveBtn} onClick={() => handleUpdateWebhook(webhookUrls[webhookFormat as keyof typeof webhookUrls])} disabled={loading}>
                                    Save Endpoint
                                </button>
                            </div>
                            <p className={styles.redirectNote}>
                                {webhookFormat === 'standard' 
                                    ? "We'll send a POST request to this URL for every transaction update." 
                                    : `Enter your ${webhookFormat} webhook URL here to receive notifications directly in your channel.`
                                }
                            </p>
                            <div className={styles.inputWrapper}>
                                <input 
                                    type="url" 
                                    className={styles.urlInput}
                                    placeholder={webhookFormat === 'standard' ? "https://api.yourwebsite.com/webhook" : `https://hooks.${webhookFormat}.com/services/...`}
                                    value={webhookUrls[webhookFormat as keyof typeof webhookUrls] || ''}
                                    onChange={(e) => {
                                        setWebhookUrls((prev: any) => ({
                                            ...prev,
                                            [webhookFormat]: e.target.value
                                        }));
                                    }}
                                />
                                <button className={styles.viewBtn} onClick={handleSendTestWebhook} disabled={loading || !webhookUrls[webhookFormat as keyof typeof webhookUrls]}>
                                    <MdSend size={14} /> Test
                                </button>
                            </div>
                        </div>

                        <div className={styles.secretSection}>
                            <div className={styles.secretHeader}>
                                <h4><MdVpnKey /> Webhook Signing Secret</h4>
                                <div style={{ display: 'flex', gap: '8px' }}>
                                    <button className={styles.viewBtn} onClick={() => setShowSecret(!showSecret)}>
                                        {showSecret ? <MdVisibilityOff size={16} /> : <MdVisibility size={16} />}
                                    </button>
                                    <button className={styles.rotateBtn} onClick={handleRotateSecret}>
                                        <MdRefresh size={16} /> Rotate
                                    </button>
                                </div>
                            </div>
                            <div className={styles.secretDisplay}>
                                {showSecret ? signingSecret : '•'.repeat(32)}
                            </div>
                            <p className={styles.redirectNote} style={{ marginTop: '12px' }}>
                                Use this secret to verify that the webhooks you receive are actually from FidduPay.
                            </p>
                        </div>
                    </div>

                    <div className={styles.docSection}>
                        <h3><MdInfo /> Verification Guide</h3>
                        {webhookFormat === 'standard' ? (
                            <>
                                <div className={styles.docGrid}>
                                    <div className={styles.docItem}>
                                        <div className={styles.docIcon}>1</div>
                                        <div className={styles.docContent}>
                                            <h4>Calculate HMAC-SHA256</h4>
                                            <p>Hash the raw request body using your signing secret.</p>
                                        </div>
                                    </div>
                                    <div className={styles.docItem}>
                                        <div className={styles.docIcon}>2</div>
                                        <div className={styles.docContent}>
                                            <h4>Compare Signature</h4>
                                            <p>Compare the result with the <code style={{ color: 'var(--primary)', background: 'var(--surface-hover)', padding: '2px 4px', borderRadius: '4px', border: '1px solid var(--border)' }}>x-fiddu-signature</code> header.</p>
                                        </div>
                                    </div>
                                </div>

                                <span className={styles.payloadLabel} style={{ marginTop: '24px', color: 'var(--text-muted)' }}>Example Standard Payload:</span>
                                <pre className={styles.payloadPre} style={{ whiteSpace: 'pre-wrap', wordBreak: 'break-all' }}>
                                    {`{
  "event": "payment.confirmed",
  "data": {
    "id": "pay_5f9a2c3b4",
    "amount": "150.00",
    "crypto_type": "SOL",
    "transaction_hash": "3xKp..."
  }
}`}
                                </pre>
                            </>
                        ) : (
                            <>
                                <p style={{ fontSize: '13px', color: 'var(--text-muted)', marginBottom: '16px' }}>
                                    {webhookFormat === 'discord' ? 'Discord' : 'Slack'} webhooks do not support HMAC signatures.
                                    FidduPay will send a simplified message format compatible with {webhookFormat === 'discord' ? 'Discord' : 'Slack'}.
                                </p>
                                <span className={styles.payloadLabel} style={{ color: 'var(--text-muted)' }}>Example {webhookFormat === 'discord' ? 'Discord' : 'Slack'} Payload:</span>
                                <pre className={styles.payloadPre} style={{ whiteSpace: 'pre-wrap', wordBreak: 'break-all' }}>
                                    {webhookFormat === 'discord' ? `{
  "content": "✅ **Payment Confirmed**\\nID: \`pay_5f9a2c3b4\`\\nAmount: \`150.00 SOL\`"
}` : `{
  "text": "✅ *Payment Confirmed*",
  "blocks": [
    {
      "type": "section",
      "text": {
        "type": "mrkdwn",
        "text": "*Payment Confirmed*\\nID: \`pay_5f9a2c3b4\`\\nAmount: \`150.00 SOL\`"
      }
    }
  ]
}`}
                                </pre>
                            </>
                        )}
                    </div>
                </div>
            </div>
        </section>
    );
};

export default WebhooksTab;
