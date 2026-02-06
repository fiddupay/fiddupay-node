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
    MdLock
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
    const [webhookUrl, setWebhookUrl] = useState('')
    const [redirectUrl, setRedirectUrl] = useState('')
    const [webhookFormat, setWebhookFormat] = useState('standard')
    const [apiKey, setApiKey] = useState('')
    const [showRotateConfirm, setShowRotateConfirm] = useState(false)
    const [showSecret, setShowSecret] = useState(false)

    useEffect(() => {
        if (user) {
            setSelectedMode(user.settlement_mode || 'managed')
            fetchSettings()
            setApiKey(user.api_key || '')
            setRedirectUrl(user.redirect_url || '')
        }
    }, [user, user?.sandbox_mode])

    const fetchSettings = async () => {
        try {
            const profileRes = await merchantAPI.getProfile()
            const feeRes = await merchantAPI.getFeeSetting()
            setWebhookUrl(profileRes.data.user.webhook_url || '')
            setRedirectUrl(profileRes.data.user.redirect_url || '')
            setWebhookFormat(profileRes.data.user.webhook_format || 'standard')
            setCustomerPaysFee(feeRes.data.customer_pays_fee)
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
            await merchantAPI.updateFeeSetting({ customer_pays_fee: newValue })
            setCustomerPaysFee(newValue)
            showToast(`Fees will now be paid by ${newValue ? 'customers' : 'you'}`, 'success')
        } catch (error: any) {
            showToast('Failed to update fee preferences', 'error')
        } finally {
            setLoading(false)
        }
    }

    const handleUpdateWebhook = async () => {
        await handleUpdateSettings({
            webhook_url: webhookUrl,
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
        if (!apiKey) {
            try {
                setLoading(true)
                const response = await merchantAPI.generateApiKey(!user.sandbox_mode)
                const newKey = response.data.api_key

                localStorage.setItem('fiddupay_token', newKey)
                if (sessionStorage.getItem('fiddupay_token')) {
                    sessionStorage.setItem('fiddupay_token', newKey)
                }

                setApiKey(newKey)
                await loadUser(true)
                showToast('API key generated successfully', 'success')
            } catch (error: any) {
                showToast('Failed to generate API key', 'error')
            } finally {
                setLoading(false)
            }
            return
        }

        // Existing key rotation requires confirmation
        if (!showRotateConfirm) {
            setShowRotateConfirm(true)
            showToast('Click rotate again to confirm. This will invalidate your current key.', 'info')
            setTimeout(() => setShowRotateConfirm(false), 5000)
            return
        }

        try {
            setLoading(true)
            const response = await merchantAPI.rotateApiKey()
            const newKey = response.data.api_key

            localStorage.setItem('fiddupay_token', newKey)
            if (sessionStorage.getItem('fiddupay_token')) {
                sessionStorage.setItem('fiddupay_token', newKey)
            }

            setApiKey(newKey)
            await loadUser(true)
            setShowRotateConfirm(false)
            showToast('API key rotated successfully', 'success')
        } catch (error: any) {
            showToast('Failed to rotate API key', 'error')
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
                                        {apiKey ? `${apiKey.substring(0, 12)}**************************` : 'No API key generated'}
                                    </div>
                                    <button
                                        className={styles.copyBtn}
                                        onClick={() => copyToClipboard(apiKey, 'API Key')}
                                        disabled={!apiKey}
                                    >
                                        <MdContentCopy /> Copy
                                    </button>
                                    <button
                                        className={`${styles.rotateBtn} ${showRotateConfirm ? 'bg-red-50 border-red-500' : ''}`}
                                        onClick={handleRotateKey}
                                        disabled={loading}
                                    >
                                        {apiKey ? (
                                            <>
                                                <MdRefresh className={loading ? 'animate-spin' : ''} />
                                                {showRotateConfirm ? 'Confirm Rotation' : 'Rotate Key'}
                                            </>
                                        ) : (
                                            <>
                                                <MdFlashOn className={loading ? 'animate-pulse' : ''} />
                                                Generate Key
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
                                        </div>

                                        <div className={styles.inputWrapper}>
                                            <input
                                                type="url"
                                                value={webhookUrl}
                                                onChange={(e) => setWebhookUrl(e.target.value)}
                                                placeholder={webhookFormat === 'discord' ? "https://discord.com/api/webhooks/..." : "https://your-domain.com/webhooks/fiddupay"}
                                                className={styles.urlInput}
                                            />
                                            <button
                                                className={styles.saveBtn}
                                                onClick={handleUpdateWebhook}
                                                disabled={loading || !webhookUrl}
                                            >
                                                {loading ? 'Saving...' : 'Update Settings'}
                                            </button>
                                        </div>
                                    </div>

                                    <div className={styles.secretSection}>
                                        <div className={styles.secretHeader}>
                                            <h4>Webhook Signing Secret</h4>
                                            <span className={styles.badge}>Global Key</span>
                                        </div>
                                        <div className={styles.secretWrapper}>
                                            <div className={styles.secretDisplay}>
                                                {showSecret ? 'whsec_8b2f9c4d1e0a7b6c5d4e3f2a1b0c9d8e' : '••••••••••••••••••••••••••••••••'}
                                            </div>
                                            <button
                                                className={styles.viewBtn}
                                                onClick={() => setShowSecret(!showSecret)}
                                            >
                                                {showSecret ? <MdVisibilityOff /> : <MdVisibility />}
                                                {showSecret ? 'Hide' : 'View'}
                                            </button>
                                        </div>
                                        <p className={styles.keyNote} style={{ marginTop: '12px', marginBottom: 0 }}>
                                            <MdInfo /> Use this secret to verify that webhook requests are genuinely from FidduPay.
                                        </p>
                                    </div>
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
                                                            : "FidduPay sends a structured JSON payload to your server."
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
                                    <span><MdLock /> {webhookFormat === 'discord' ? 'Discord Formatting' : 'Signature Verification'}</span>
                                    <span>{webhookFormat === 'discord' ? 'No Verification' : 'Standard Headers'}</span>
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
                                            Discord webhooks do not support HMAC signatures.
                                            FidduPay will send a simplified message format compatible with Discord.
                                        </p>
                                        <span className={styles.payloadLabel}>Example Discord Payload:</span>
                                        <pre className={styles.payloadPre}>
                                            {`{
  "content": "✅ **Payment Confirmed**\\nID: \`pay_5f9a2c3b4\`\\nAmount: \`150.00 SOL\`"
}`}
                                        </pre>
                                    </>
                                )}
                            </div>
                        </div>
                    </section>
                )}
            </div>
        </div>
    )
}

export default SettingsPage
