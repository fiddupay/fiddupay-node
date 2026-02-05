import React, { useState, useEffect } from 'react'
import { MdAccountBalanceWallet, MdForward, MdCloudDone, MdCheckCircle, MdContentCopy, MdRefresh, MdVpnKey } from 'react-icons/md'
import { useAuthStore } from '@/stores/authStore'
import { merchantAPI } from '@/services/apiService'
import { useToast } from '@/contexts/ToastContext'
import styles from '@/styles/pages/SettingsPage.module.css'

const SettingsPage: React.FC = () => {
    const { user, loadUser } = useAuthStore()
    const { showToast } = useToast()
    const [loading, setLoading] = useState(false)
    const [selectedMode, setSelectedMode] = useState<'forwarding' | 'managed' | 'imported'>('managed')
    const [customerPaysFee, setCustomerPaysFee] = useState(false)
    const [webhookUrl, setWebhookUrl] = useState('')
    const [apiKey, setApiKey] = useState('')

    useEffect(() => {
        if (user) {
            setSelectedMode(user.settlement_mode || 'managed')
            fetchSettings()
            setApiKey(user.api_key || '')
        }
    }, [user, user?.sandbox_mode])

    const fetchSettings = async () => {
        try {
            const profileRes = await merchantAPI.getProfile()
            const feeRes = await merchantAPI.getFeeSetting()
            setWebhookUrl(profileRes.data.user.webhook_url || '')
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
        await handleUpdateSettings({ webhook_url: webhookUrl })
    }

    const copyToClipboard = (text: string, label: string) => {
        navigator.clipboard.writeText(text)
        showToast(`${label} copied to clipboard`, 'success')
    }

    const handleRotateKey = async () => {
        if (!window.confirm('Are you sure you want to rotate your API key? This will invalidate your current key and you will need to update your integration and log in again.')) {
            return
        }

        try {
            setLoading(true)
            const response = await merchantAPI.rotateApiKey()
            const newKey = response.data.api_key

            // Update local storage and session storage to reflect the new key
            localStorage.setItem('fiddupay_token', newKey)
            if (sessionStorage.getItem('fiddupay_token')) {
                sessionStorage.setItem('fiddupay_token', newKey)
            }

            setApiKey(newKey)
            await loadUser(true)
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

            <div className={styles.content}>
                {/* Settlement Mode Section */}
                <section className={styles.section}>
                    <h2>Settlement Mode</h2>
                    <p>Choose how you want to receive and manage your funds. This setting applies to all connected wallets.</p>

                    <div className={styles.modeGrid}>
                        {/* Forwarding Mode */}
                        <div
                            className={`${styles.modeCard} ${selectedMode === 'forwarding' ? styles.activeCard : ''}`}
                            onClick={() => handleUpdateSettlementMode('forwarding')}
                        >
                            {selectedMode === 'forwarding' && <MdCheckCircle className={styles.checkIcon} />}
                            <MdForward size={32} />
                            <h3>Forwarding Bridge</h3>
                            <span>Auto-forwards funds to your external addresses. Non-custodial and trustless.</span>
                        </div>

                        {/* Managed Mode */}
                        <div
                            className={`${styles.modeCard} ${selectedMode === 'managed' ? styles.activeCard : ''}`}
                            onClick={() => handleUpdateSettlementMode('managed')}
                        >
                            {selectedMode === 'managed' && <MdCheckCircle className={styles.checkIcon} />}
                            <MdCloudDone size={32} />
                            <h3>Managed Wallet</h3>
                            <span>Funds are held in FidduPay generated wallets. Withdraw whenever you want.</span>
                        </div>

                        {/* Imported Mode */}
                        <div
                            className={`${styles.modeCard} ${selectedMode === 'imported' ? styles.activeCard : ''}`}
                            onClick={() => handleUpdateSettlementMode('imported')}
                        >
                            {selectedMode === 'imported' && <MdCheckCircle className={styles.checkIcon} />}
                            <MdAccountBalanceWallet size={32} />
                            <h3>Imported Wallet</h3>
                            <span>Use your own private keys. Advanced custodial setup with platform management.</span>
                        </div>
                    </div>
                </section>

                {/* API Settings Section */}
                <section className={styles.section}>
                    <h2>API Settings</h2>
                    <p>Manage your API credentials for integrating FidduPay with your application.</p>

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
                                    title="Copy API Key"
                                >
                                    <MdContentCopy />
                                    Copy
                                </button>
                                <button
                                    className={styles.rotateBtn}
                                    onClick={handleRotateKey}
                                    disabled={loading}
                                    title="Rotate API Key"
                                >
                                    <MdRefresh />
                                    Rotate
                                </button>
                            </div>

                            <div className={styles.keyFooter}>
                                <span className={styles.keyNote}>
                                    Keep your keys secure. Never share them in client-side code.
                                </span>
                                <span className="text-xs text-gray-400">
                                    Last rotated: {user?.created_at ? new Date(user.created_at).toLocaleDateString() : 'N/A'}
                                </span>
                            </div>
                        </div>
                    </div>
                </section>

                {/* Fee Preferences Section */}
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

                {/* Webhook Configuration Section */}
                <section className={styles.section}>
                    <h2>Webhook Configuration</h2>
                    <p>FidduPay will send POST requests to this URL for all transaction events.</p>

                    <div className={styles.inputGroup}>
                        <div className={styles.inputWrapper}>
                            <input
                                type="url"
                                value={webhookUrl}
                                onChange={(e) => setWebhookUrl(e.target.value)}
                                placeholder="https://your-domain.com/webhooks/fiddupay"
                                className={styles.urlInput}
                            />
                            <button
                                className={styles.saveBtn}
                                onClick={handleUpdateWebhook}
                                disabled={loading || !webhookUrl}
                            >
                                {loading ? 'Saving...' : 'Update Webhook'}
                            </button>
                        </div>
                        <p className={styles.helperText}>All event data is signed for security. Ensure your endpoint is publicly accessible via HTTPS.</p>
                    </div>
                </section>
            </div>
        </div>
    )
}

export default SettingsPage
