import React, { useState, useEffect } from 'react'
import {
    MdAccountBalanceWallet,
    MdForward,
    MdCloudDone,
    MdCheckCircle,
    MdContentCopy,
    MdRefresh,
    MdVpnKey,
    MdCode,
    MdPayment,
    MdNotificationsActive,
    MdFlashOn,
    MdInfo,
    MdVisibility,
    MdVisibilityOff,
    MdHelp,
    MdLock,
    MdClose,
    MdWarning,
    MdError
} from 'react-icons/md'
import { useAuthStore } from '@/stores/authStore'
import { merchantAPI } from '@/services/apiService'
import { useToast } from '@/contexts/ToastContext'
import styles from '@/styles/pages/SettingsPage.module.css'

type TabType = 'settlement' | 'fees' | 'api' | 'webhooks'

const SettingsPage: React.FC = () => {
    const { user, loadUser } = useAuthStore()
    const { showToast } = useToast()
    const [loading, setLoading] = useState(false)
    const [activeTab, setActiveTab] = useState<TabType>('settlement')
    const [selectedMode, setSelectedMode] = useState<'forwarding' | 'managed' | 'imported'>('managed')
    const [customerPaysFee, setCustomerPaysFee] = useState(false)
    const [webhookUrls, setWebhookUrls] = useState({
        standard: '',
        discord: '',
        slack: ''
    })
    const [redirectUrl, setRedirectUrl] = useState('')
    const [webhookFormat, setWebhookFormat] = useState('standard')
    const [apiKey, setApiKey] = useState('')
    const [showApiKey, setShowApiKey] = useState(false)
    const [showRotateModal, setShowRotateModal] = useState(false)
    const [showSecret, setShowSecret] = useState(false)
    const [signingSecret, setSigningSecret] = useState('••••••••••••••••••••••••••••••••')
    const [showRotateSecretConfirm, setShowRotateSecretConfirm] = useState(false)

    useEffect(() => {
        fetchSettings()
    }, [])

    useEffect(() => {
        if (user) {
            setSelectedMode(user.settlement_mode || 'managed')

            // Only update local apiKey if:
            // 1. It's currently empty
            // 2. The incoming key is valid AND we don't have a plaintext key currently shown
            //    (This prevents overwriting a newly generated key with the masked version from profile fetch)
            const incomingKey = user.api_key || ''
            const isIncomingMasked = incomingKey.includes('********')
            const isCurrentMasked = apiKey.includes('********') || !apiKey

            // If we have a plaintext key displayed (newly generated), and the incoming is masked, 
            // DON'T replace it. Otherwise (first load, or switching environments), update it.
            if ((!apiKey && incomingKey) || (isCurrentMasked && incomingKey) || (!isIncomingMasked && incomingKey !== apiKey)) {
                setApiKey(incomingKey)
            }

            setRedirectUrl(user.redirect_url || '')

            // Populate the correct webhook URL based on the format
            // Other formats start empty (avoids bleeding)
            const format = user.webhook_format || 'standard'
            setWebhookUrls(prev => ({
                ...prev,
                [format]: user.webhook_url || ''
            }))

            setWebhookFormat(format)
        }
    }, [user]) // Removed user?.sandbox_mode from deps to rely on the full user object check

    const fetchSettings = async () => {
        try {
            // We don't need to call getProfile here because loadUser() in authStore does it
            // But we do need specific settings that might not be on the user object yet or require separate calls?
            // Actually, getProfile returns the user with webhook_url. 
            // However, let's refresh the user to be sure.
            await loadUser(true)

            // Also fetch fee settings which are separate
            const feeRes = await merchantAPI.getFeeSetting()
            setCustomerPaysFee(feeRes.data.customer_pays_fee)

            // Get webhook signing secret (it returns inside getProfile too but let's check)
            // The getProfile endpoint returns everything we need in 'user' object except maybe the secret?
            // Use the specific getMerchantSettings endpoint if available, but getProfile seems to have most.
            // Let's check getMerchantSettings implementation in backend...
            // It returns webhook_signing_secret. 
            // Let's use getMerchantSettings to get the secret.
            const settingsRes = await merchantAPI.getMerchantSettings()
            setSigningSecret(settingsRes.data.webhook_signing_secret || '••••••••••••••••••••••••••••••••')

        } catch (error) {
            console.error('Failed to fetch settings', error)
        }
    }

    const handleUpdateSettings = async (updates: any) => {
        try {
            setLoading(true)
            await merchantAPI.updateSettings(updates)
            await loadUser(true)
            showToast('Settings updated successfully', 'success')
        } catch (error: any) {
            showToast(error.response?.data?.error?.message || error.response?.data?.error || 'Failed to update settings', 'error')
        } finally {
            setLoading(false)
        }
    }

    const handleUpdateSettlementMode = async (mode: 'forwarding' | 'managed' | 'imported') => {
        await handleUpdateSettings({ settlement_mode: mode })
        setSelectedMode(mode)
    }

    const handleUpdateFeeSetting = async () => {
        const newValue = !customerPaysFee
        try {
            setLoading(true)
            await merchantAPI.updateSettings({ customer_pays_fee: newValue })
            setCustomerPaysFee(newValue)
            showToast(`Fees will now be paid by ${newValue ? 'customers' : 'you'}`, 'success')
        } catch (error: any) {
            showToast('Failed to update fee preferences', 'error')
        } finally {
            setLoading(false)
        }
    }

    const handleUpdateWebhook = async () => {
        // Get the URL for the currently selected format
        // @ts-ignore - dynamic key access
        const urlToSave = webhookUrls[webhookFormat] || ''

        await handleUpdateSettings({
            webhook_url: urlToSave,
            webhook_format: webhookFormat
        })
    }

    const handleUpdateRedirect = async () => {
        await handleUpdateSettings({
            redirect_url: redirectUrl
        })
    }

    const copyToClipboard = (text: string, label: string) => {
        navigator.clipboard.writeText(text)
        showToast(`${label} copied to clipboard`, 'success')
    }

    const handleRotateKey = async () => {
        if (!user) return

        // If no key exists, we can generate directly without confirmation
        if (!apiKey || apiKey === 'No API key generated') {
            try {
                setLoading(true)
                const response = await merchantAPI.generateApiKey(!user.sandbox_mode)
                const newKey = response.data.api_key
                setApiKey(newKey)
                await loadUser(true)
                showToast(`New ${user.sandbox_mode ? 'Sandbox' : 'Live'} API key generated successfully`, 'success')
            } catch (error: any) {
                showToast(error.response?.data?.message || 'Failed to generate API key', 'error')
            } finally {
                setLoading(false)
            }
            return
        }

        // Existing key rotation requires modal confirmation
        setShowRotateModal(true)
    }

    const confirmRotation = async () => {
        if (!user) return

        try {
            setLoading(true)
            const response = await merchantAPI.rotateApiKey(!user.sandbox_mode)
            const newKey = response.data.api_key
            setApiKey(newKey)
            await loadUser(true)
            setShowRotateModal(false)
            showToast(`API key rotated successfully. Old key is now invalid.`, 'success')
        } catch (error: any) {
            showToast(error.response?.data?.message || 'Failed to rotate API key', 'error')
        } finally {
            setLoading(false)
        }
    }

    const handleRotateSecret = async () => {
        if (!showRotateSecretConfirm) {
            setShowRotateSecretConfirm(true)
            showToast('Click rotate again to confirm. This will invalidate your current secret.', 'info')
            setTimeout(() => setShowRotateSecretConfirm(false), 5000)
            return
        }

        try {
            setLoading(true)
            await merchantAPI.updateSettings({ rotate_webhook_secret: true })
            await fetchSettings()
            setShowRotateSecretConfirm(false)
            showToast('Webhook signing secret rotated successfully', 'success')
        } catch (error: any) {
            showToast('Failed to rotate signing secret', 'error')
        } finally {
            setLoading(false)
        }
    }

    const handleSendTestWebhook = async () => {
        try {
            setLoading(true)
            await merchantAPI.sendTestWebhook()
            showToast('Test webhook queued for delivery', 'success')
        } catch (error: any) {
            showToast('Failed to send test webhook', 'error')
        } finally {
            setLoading(false)
        }
    }

    return (
        <div className={styles.page}>
            <div className={styles.header}>
                <h1>Settings</h1>
                <p>Global account configuration and payout preferences</p>
            </div>

            <div className={styles.tabs}>
                <button
                    className={`${styles.tabBtn} ${activeTab === 'settlement' ? styles.activeTab : ''}`}
                    onClick={() => setActiveTab('settlement')}
                >
                    <MdAccountBalanceWallet /> Settlement Mode
                </button>
                <button
                    className={`${styles.tabBtn} ${activeTab === 'fees' ? styles.activeTab : ''}`}
                    onClick={() => setActiveTab('fees')}
                >
                    <MdPayment /> Fee Preferences
                </button>
                <button
                    className={`${styles.tabBtn} ${activeTab === 'api' ? styles.activeTab : ''}`}
                    onClick={() => setActiveTab('api')}
                >
                    <MdCode /> API Settings
                </button>
                <button
                    className={`${styles.tabBtn} ${activeTab === 'webhooks' ? styles.activeTab : ''}`}
                    onClick={() => setActiveTab('webhooks')}
                >
                    <MdNotificationsActive /> Webhooks
                </button>
            </div>

            <div className={styles.content}>
                {activeTab === 'settlement' && (
                    <section className={styles.section}>
                        <h2>Settlement Mode</h2>
                        <p>Choose how you want to receive and manage your funds.</p>

                        <div className={styles.modeGrid}>
                            <div
                                className={`${styles.modeCard} ${selectedMode === 'forwarding' ? styles.activeCard : ''}`}
                                onClick={() => handleUpdateSettlementMode('forwarding')}
                            >
                                {selectedMode === 'forwarding' && <MdCheckCircle className={styles.checkIcon} />}
                                <MdForward size={32} />
                                <h3>Forwarding Bridge</h3>
                                <span>Auto-forwards funds to your external addresses.</span>
                            </div>

                            <div
                                className={`${styles.modeCard} ${selectedMode === 'managed' ? styles.activeCard : ''}`}
                                onClick={() => handleUpdateSettlementMode('managed')}
                            >
                                {selectedMode === 'managed' && <MdCheckCircle className={styles.checkIcon} />}
                                <MdCloudDone size={32} />
                                <h3>Managed Wallet</h3>
                                <span>Funds are held in FidduPay generated wallets.</span>
                            </div>

                            <div
                                className={`${styles.modeCard} ${selectedMode === 'imported' ? styles.activeCard : ''}`}
                                onClick={() => handleUpdateSettlementMode('imported')}
                            >
                                {selectedMode === 'imported' && <MdCheckCircle className={styles.checkIcon} />}
                                <MdAccountBalanceWallet size={32} />
                                <h3>Imported Wallet</h3>
                                <span>Use your own private keys for advanced setup.</span>
                            </div>
                        </div>
                    </section>
                )}

                {activeTab === 'fees' && (
                    <section className={styles.section}>
                        <h2>Fee Preferences</h2>
                        <p>Configure who covers the platform processing fees.</p>

                        <div className={styles.toggleGroup}>
                            <div className={styles.toggleLabel}>
                                <h4>Pass Fee to Customer</h4>
                                <span>When enabled, the processing fee is added to the customer's total.</span>
                            </div>

                            <label className={styles.switch}>
                                <input
                                    type="checkbox"
                                    checked={customerPaysFee}
                                    onChange={handleUpdateFeeSetting}
                                    disabled={loading}
                                />
                                <span className={styles.slider}></span>
                            </label>
                        </div>
                    </section>
                )}

                {activeTab === 'api' && (
                    <section className={styles.section}>
                        <h2>API Settings</h2>
                        <p>Manage your API credentials for integrating FidduPay.</p>

                        <div className={styles.keyGrid}>
                            <div className={styles.keyCard}>
                                <div className={styles.keyHeader}>
                                    <div className="flex items-center gap-2">
                                        <MdVpnKey className="text-blue-500" />
                                        <h4>Merchant API Key</h4>
                                    </div>
                                    <span className={`${styles.badge} ${user?.sandbox_mode ? styles.badgeSandbox : styles.badgeLive}`}>
                                        {user?.sandbox_mode ? 'Sandbox' : 'Live'}
                                    </span>
                                </div>

                                <div className={styles.keyInputGroup}>
                                    <div className={styles.keyDisplay}>
                                        {apiKey ? (
                                            showApiKey ? apiKey : (
                                                apiKey.includes('*') ? apiKey : `${apiKey.substring(0, 12)}...`
                                            )
                                        ) : 'No API key generated'}
                                    </div>
                                    <button
                                        className={`${styles.copyBtn} ${apiKey.includes('*') ? 'opacity-50 cursor-not-allowed' : ''}`}
                                        onClick={() => {
                                            if (apiKey.includes('*')) {
                                                showToast('For security, existing keys cannot be viewed. Rotate to generate a new one.', 'info')
                                                return
                                            }
                                            setShowApiKey(!showApiKey)
                                        }}
                                        disabled={!apiKey}
                                        title={apiKey.includes('*') ? 'Cannot view existing key (Hidden for security)' : (showApiKey ? 'Hide Key' : 'Show Key')}
                                    >
                                        {showApiKey ? <MdVisibilityOff /> : <MdVisibility />}
                                    </button>
                                    <button
                                        className={styles.copyBtn}
                                        onClick={() => copyToClipboard(apiKey, 'API Key')}
                                        disabled={!apiKey || apiKey.includes('*')}
                                        title={apiKey.includes('*') ? 'Rotate key to get a valid copy' : 'Copy Key'}
                                    >
                                        <MdContentCopy /> Copy
                                    </button>
                                    <button
                                        className={styles.rotateBtn}
                                        onClick={handleRotateKey}
                                        disabled={loading}
                                    >
                                        {(!apiKey || apiKey === 'No API key generated') ? (
                                            <>
                                                <MdFlashOn className={loading ? 'animate-pulse' : ''} />
                                                Generate Key
                                            </>
                                        ) : (
                                            <>
                                                <MdRefresh className={loading ? 'animate-spin' : ''} />
                                                Rotate Key
                                            </>
                                        )}
                                    </button>
                                </div>

                                <div className={styles.keyFooter}>
                                    <span className={styles.keyNote}>
                                        Keep your keys secure. Never share them in client-side code.
                                    </span>
                                </div>
                            </div>

                            <div className={styles.redirectSection}>
                                <div className={styles.redirectHeader}>
                                    <div className="flex items-center gap-2">
                                        <MdForward className="text-blue-500" />
                                        <h4>Customer Redirect URL</h4>
                                    </div>
                                    <span className={styles.badge}>Optional</span>
                                </div>
                                <p className={styles.redirectNote}>
                                    After a successful payment, the customer will be automatically sent back to this URL.
                                </p>
                                <div className={styles.inputWrapper}>
                                    <input
                                        type="url"
                                        value={redirectUrl}
                                        onChange={(e) => setRedirectUrl(e.target.value)}
                                        placeholder="https://your-site.com/checkout/success"
                                        className={styles.urlInput}
                                    />
                                    <button
                                        className={styles.saveBtn}
                                        onClick={handleUpdateRedirect}
                                        disabled={loading || !redirectUrl}
                                    >
                                        {loading ? 'Saving...' : 'Update URL'}
                                    </button>
                                </div>
                            </div>
                        </div>
                    </section>
                )}

                {activeTab === 'webhooks' && (
                    <section className={styles.section}>
                        <div className={styles.webhookLayout}>
                            <div className={styles.webhookMain}>
                                <div className={styles.configSide}>
                                    <h2>Webhook Configuration</h2>
                                    <p>FidduPay will send real-time notifications to your URL when payment statuses change.</p>

                                    <div className={styles.inputGroup}>
                                        <label className={styles.toggleLabel} style={{ marginBottom: '12px' }}>
                                            <h4>Notification Format</h4>
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
                                                Discord Webhook
                                            </button>
                                            <button
                                                className={`${styles.formatBtn} ${webhookFormat === 'slack' ? styles.activeFormat : ''}`}
                                                onClick={() => setWebhookFormat('slack')}
                                            >
                                                Slack Webhook
                                            </button>
                                        </div>

                                        <div className={styles.inputWrapper}>
                                            <input
                                                type="url"
                                                // @ts-ignore
                                                value={webhookUrls[webhookFormat]}
                                                onChange={(e) => setWebhookUrls(prev => ({
                                                    ...prev,
                                                    [webhookFormat]: e.target.value
                                                }))}
                                                placeholder={webhookFormat === 'discord' ? "https://discord.com/api/webhooks/..." : (webhookFormat === 'slack' ? "https://hooks.slack.com/services/..." : "https://your-domain.com/webhooks/fiddupay")}
                                                className={styles.urlInput}
                                            />
                                            <button
                                                className={styles.saveBtn}
                                                onClick={handleUpdateWebhook}
                                                disabled={loading || !webhookUrls[webhookFormat as keyof typeof webhookUrls]}
                                            >
                                                {loading ? 'Saving...' : 'Update Settings'}
                                            </button>
                                            <button
                                                className={styles.copyBtn}
                                                onClick={handleSendTestWebhook}
                                                disabled={loading || !webhookUrls[webhookFormat as keyof typeof webhookUrls]}
                                                title="Send a sample notification to your URL"
                                            >
                                                <MdFlashOn /> Test Webhook
                                            </button>
                                        </div>
                                    </div>

                                    {webhookFormat === 'standard' && (
                                        <div className={styles.secretSection}>
                                            <div className={styles.secretHeader}>
                                                <h4>Webhook Signing Secret</h4>
                                                <span className={styles.badge}>Per-Merchant Key</span>
                                            </div>
                                            <div className={styles.secretWrapper}>
                                                <div className={styles.secretDisplay}>
                                                    {showSecret ? signingSecret : '••••••••••••••••••••••••••••••••'}
                                                </div>
                                                <button
                                                    className={styles.viewBtn}
                                                    onClick={() => setShowSecret(!showSecret)}
                                                >
                                                    {showSecret ? <MdVisibilityOff /> : <MdVisibility />}
                                                    {showSecret ? 'Hide' : 'View'}
                                                </button>
                                                <button
                                                    className={`${styles.rotateBtn} ${showRotateSecretConfirm ? 'bg-red-50 border-red-500' : ''}`}
                                                    onClick={handleRotateSecret}
                                                    disabled={loading}
                                                >
                                                    <MdRefresh className={loading ? 'animate-spin' : ''} />
                                                    {showRotateSecretConfirm ? 'Confirm' : 'Rotate'}
                                                </button>
                                            </div>
                                            <p className={styles.keyNote} style={{ marginTop: '12px', marginBottom: 0 }}>
                                                <MdInfo /> Use this secret to verify that webhook requests are genuinely from FidduPay.
                                            </p>
                                        </div>
                                    )}
                                </div>

                                <div className={styles.docSide}>
                                    <div className={styles.docSection}>
                                        <h3><MdHelp /> How it works</h3>
                                        <div className={styles.docGrid}>
                                            <div className={styles.docItem}>
                                                <div className={styles.docIcon}>1</div>
                                                <div className={styles.docContent}>
                                                    <h4>Event Triggered</h4>
                                                    <p>An event occurs (e.g., a payment is confirmed by the network).</p>
                                                </div>
                                            </div>
                                            <div className={styles.docItem}>
                                                <div className={styles.docIcon}>2</div>
                                                <div className={styles.docContent}>
                                                    <h4>POST Request</h4>
                                                    <p>
                                                        {webhookFormat === 'discord'
                                                            ? "FidduPay sends a Discord-formatted message directly to your channel."
                                                            : (webhookFormat === 'slack'
                                                                ? "FidduPay sends a Slack-formatted message directly to your channel."
                                                                : "FidduPay sends a structured JSON payload to your server.")
                                                        }
                                                    </p>
                                                </div>
                                            </div>
                                            <div className={styles.docItem}>
                                                <div className={styles.docIcon}>3</div>
                                                <div className={styles.docContent}>
                                                    <h4>Acknowledgement</h4>
                                                    <p>Your server (or Discord) acknowledges the notification.</p>
                                                </div>
                                            </div>
                                        </div>
                                    </div>
                                </div>
                            </div>

                            <div className={styles.verificationBox}>
                                <div className={styles.boxHeader}>
                                    <span><MdLock /> {(webhookFormat === 'discord' || webhookFormat === 'slack') ? 'Payload Formatting' : 'Signature Verification'}</span>
                                    <span>{(webhookFormat === 'discord' || webhookFormat === 'slack') ? 'No Verification' : 'Standard Headers'}</span>
                                </div>
                                {webhookFormat === 'standard' ? (
                                    <>
                                        <div className={styles.headerList}>
                                            <div className={styles.headerItem}>
                                                <span className={styles.headerKey}>X-Signature:</span>
                                                <span className={styles.headerValue}>t=1707172800,v1=sha256_hmac_hex_result...</span>
                                            </div>
                                            <div className={styles.headerItem}>
                                                <span className={styles.headerKey}>X-Timestamp:</span>
                                                <span className={styles.headerValue}>1707172800</span>
                                            </div>
                                        </div>
                                        <span className={styles.payloadLabel}>Example Payload JSON:</span>
                                        <pre className={styles.payloadPre}>
                                            {`{
  "event_type": "payment.confirmed",
  "payment_id": "pay_5f9a2c3b4",
  "status": "CONFIRMED",
  "amount": "150.00",
  "crypto_type": "SOL",
  "timestamp": 1707172800
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
                    </section>
                )}
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
                                    Any applications or services using the old key will stop working until updated.
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
                                {loading ? (
                                    <>
                                        <MdRefresh className="animate-spin" /> Rotating...
                                    </>
                                ) : (
                                    <>
                                        <MdRefresh /> Confirm Rotation
                                    </>
                                )}
                            </button>
                        </div>
                    </div>
                </div>
            )}
        </div>
    )
}

export default SettingsPage
