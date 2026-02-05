import React, { useState, useEffect } from 'react'
import { MdAccountBalanceWallet, MdForward, MdCloudDone, MdCheckCircle } from 'react-icons/md'
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

    useEffect(() => {
        if (user) {
            setSelectedMode(user.settlement_mode || 'managed')
            fetchSettings()
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
            showToast(error.response?.data?.error || 'Failed to update settings', 'error')
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
        await handleUpdateSettings({ customer_pays_fee: newValue })
        setCustomerPaysFee(newValue)
    }

    const handleUpdateWebhook = async () => {
        await handleUpdateSettings({ webhook_url: webhookUrl })
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

                {/* Fee Preferences Section */}
                <section className={styles.section}>
                    <h2>Fee Preferences</h2>
                    <p>Configure who covers the platform processing fees.</p>

                    <div className={styles.toggleGroup}>
                        <div className={styles.toggleLabel}>
                            <h4>Pass Fee to Customer</h4>
                            <span>When enabled, the processing fee is added to the customer's total.</span>
                        </div>

                        <button
                            className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 ${customerPaysFee ? 'bg-blue-600' : 'bg-gray-200'
                                }`}
                            onClick={handleUpdateFeeSetting}
                            disabled={loading}
                        >
                            <span
                                className={`${customerPaysFee ? 'translate-x-6' : 'translate-x-1'
                                    } inline-block h-4 w-4 transform rounded-full bg-white transition-transform`}
                            />
                        </button>
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
