import React from 'react';
import { MdNotifications, MdSend, MdVpnKey, MdInfo, MdVisibility, MdVisibilityOff, MdRefresh } from 'react-icons/md';

interface WebhooksTabProps {
    webhookUrls: {
        standard: string;
        discord: string;
        slack: string;
    };
    setWebhookUrls: React.Dispatch<React.SetStateAction<{
        standard: string;
        discord: string;
        slack: string;
    }>>;
    webhookFormat: 'standard' | 'discord' | 'slack' | string;
    setWebhookFormat: (format: 'standard' | 'discord' | 'slack') => void;
    handleUpdateWebhook: (url: string) => Promise<void>;
    handleSendTestWebhook: () => Promise<void>;
    signingSecret: string;
    showSecret: boolean;
    setShowSecret: (show: boolean) => void;
    handleRotateSecret: () => Promise<void>;
    loading: boolean;
    styles: any;
}

const WebhooksTab: React.FC<WebhooksTabProps> = ({
    webhookUrls,
    setWebhookUrls,
    webhookFormat,
    setWebhookFormat,
    handleUpdateWebhook,
    handleSendTestWebhook,
    signingSecret,
    showSecret,
    setShowSecret,
    handleRotateSecret,
    loading,
    styles
}) => {
    return (
        <section className={styles.section}>
            <h2>Webhooks & Notifications</h2>
            <p>Get real-time updates when payments are confirmed.</p>

            <div className={styles.webhookLayout}>
                <div className={styles.webhookMain}>
                    <div className={styles.webhookConfig}>
                        <div className={styles.inputGroup}>
                            <label style={{ fontSize: '14px', fontWeight: 600, color: '#374151', marginBottom: '8px', display: 'block' }}>
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
                                            <p>Compare the result with the <code style={{ color: '#eb5757', background: '#fff5f5', padding: '2px 4px', borderRadius: '4px' }}>x-fiddu-signature</code> header.</p>
                                        </div>
                                    </div>
                                </div>

                                <span className={styles.payloadLabel} style={{ marginTop: '24px' }}>Example Standard Payload:</span>
                                <pre className={styles.payloadPre}>
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
                                <p style={{ fontSize: '13px', color: '#888', marginBottom: '16px' }}>
                                    {webhookFormat === 'discord' ? 'Discord' : 'Slack'} webhooks do not support HMAC signatures.
                                    FidduPay will send a simplified message format compatible with {webhookFormat === 'discord' ? 'Discord' : 'Slack'}.
                                </p>
                                <span className={styles.payloadLabel}>Example {webhookFormat === 'discord' ? 'Discord' : 'Slack'} Payload:</span>
                                <pre className={styles.payloadPre}>
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
