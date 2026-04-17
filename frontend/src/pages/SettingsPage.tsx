import React, { useState, useEffect } from 'react'
import {
    MdAccountBalanceWallet,
    MdRefresh,
    MdCode,
    MdPayment,
    MdNotificationsActive,
    MdLock,
    MdClose,
    MdWarning,
    MdError
} from 'react-icons/md'
import { useAuthStore } from '@/stores/authStore'
import { merchantAPI, securityAPI, addressOnlyAPI } from '@/services/apiService'
import { useToast } from '@/contexts/ToastContext'
import styles from '@/styles/pages/SettingsPage.module.css'

// Modular Tabs
import SettlementTab from '@/components/settings/tabs/SettlementTab'
import FeesTab from '@/components/settings/tabs/FeesTab'
import ApiSettingsTab from '@/components/settings/tabs/ApiSettingsTab'
import WebhooksTab from '@/components/settings/tabs/WebhooksTab'
import SecurityTab from '@/components/settings/tabs/SecurityTab'
import WidgetTab from '@/components/settings/tabs/WidgetTab'

type TabType = 'settlement' | 'fees' | 'api' | 'webhooks' | 'security' | 'widget'

const SettingsPage: React.FC = () => {
    const { user, loadUser } = useAuthStore()
    const { showToast } = useToast()
    const [loading, setLoading] = useState(false)
    const [activeTab, setActiveTab] = useState<TabType>('settlement')
    const [selectedMode, setSelectedMode] = useState<'forwarding' | 'managed'>('managed')
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
    const [ipWhitelist, setIpWhitelist] = useState<string[]>([])
    const [newIp, setNewIp] = useState('')
    const [pin, setPin] = useState('')
    const [confirmPin, setConfirmPin] = useState('')
    const [settingPin, setSettingPin] = useState(false)
    const [lowBalanceThreshold, setLowBalanceThreshold] = useState('0')
    const [lowBalanceAlertsEnabled, setLowBalanceAlertsEnabled] = useState(true)
    const [addressOnlyCustomerPaysFee, setAddressOnlyCustomerPaysFee] = useState(false)
    const [passwordConfirm, setPasswordConfirm] = useState<{
        show: boolean;
        target: 'wallet' | 'customer' | null;
        newLockState: boolean;
        password: '';
    }>({
        show: false,
        target: null,
        newLockState: false,
        password: ''
    })

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
            setIpWhitelist(user.ip_whitelist || [])
            setLowBalanceThreshold(user.low_balance_threshold_usd || '0')
            setLowBalanceAlertsEnabled(user.low_balance_alerts_enabled !== false) // Default to true if undefined
        }
    }, [user]) // Removed user?.sandbox_mode from deps to rely on the full user object check

    const handleSetPin = async (e: React.FormEvent) => {
        e.preventDefault()
        if (pin.length !== 4 || !/^\d+$/.test(pin)) {
            showToast('PIN must be exactly 4 digits', 'warning')
            return
        }
        if (pin !== confirmPin) {
            showToast('PINs do not match', 'error')
            return
        }

        try {
            setSettingPin(true)
            await merchantAPI.setTransactionPin(pin)
            showToast('Transaction PIN set successfully', 'success')
            setPin('')
            setConfirmPin('')
            await loadUser(true)
        } catch (error: any) {
            showToast(error.response?.data?.error || 'Failed to set PIN', 'error')
        } finally {
            setSettingPin(false)
        }
    }

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

            // Fetch Address-Only fee settings if reachable
            try {
                const aoFeeRes = await addressOnlyAPI.getFeeSetting()
                setAddressOnlyCustomerPaysFee(aoFeeRes.data.customer_pays_fee)
            } catch (err) {
                console.warn('Address-Only settings not available', err)
            }

        } catch (error) {
            console.error('Failed to fetch settings', error)
        }
    }

    const handleAddIp = async () => {
        if (!newIp) return
        if (ipWhitelist.includes(newIp)) {
            showToast('IP already in whitelist', 'warning')
            return
        }
        const updated = [...ipWhitelist, newIp]
        await handleUpdateSettings({ ip_whitelist: updated })
        setIpWhitelist(updated)
        setNewIp('')
    }

    const handleRemoveIp = async (ip: string) => {
        const updated = ipWhitelist.filter(i => i !== ip)
        await handleUpdateSettings({ ip_whitelist: updated })
        setIpWhitelist(updated)
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

    const handleUpdateSettlementMode = async (mode: 'forwarding' | 'managed') => {
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
        }
    }

    const handleUpdateAddressOnlyFeeSetting = async (customerPays: boolean) => {
        try {
            setLoading(true)
            await addressOnlyAPI.updateFeeSetting({ customer_pays_fee: customerPays })
            setAddressOnlyCustomerPaysFee(customerPays)
            showToast(`Forwarding fees updated: ${customerPays ? 'Customer' : 'Merchant'} pays`, 'success')
        } catch (error: any) {
            showToast('Failed to update forwarding fee preference', 'error')
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

    const handleToggleWalletLock = async () => {
        if (!user) return
        const newLockState = !user.wallets_locked
        setPasswordConfirm({
            show: true,
            target: 'wallet',
            newLockState,
            password: ''
        })
    }

    const handleToggleCustomerWalletLock = async () => {
        if (!user) return
        const newLockState = !user.customer_wallets_locked
        setPasswordConfirm({
            show: true,
            target: 'customer',
            newLockState,
            password: ''
        })
    }

    const confirmLockAction = async () => {
        if (!passwordConfirm.target || !passwordConfirm.password) {
            showToast('Please enter your password to confirm', 'error')
            return
        }

        try {
            setLoading(true)
            if (passwordConfirm.target === 'wallet') {
                await securityAPI.toggleWalletLock(passwordConfirm.newLockState, passwordConfirm.password)
                showToast(`Wallets ${passwordConfirm.newLockState ? 'locked' : 'unlocked'} successfully`, 'success')
            } else {
                await securityAPI.toggleCustomerWalletLock(passwordConfirm.newLockState, passwordConfirm.password)
                showToast(`Customer wallets ${passwordConfirm.newLockState ? 'locked' : 'unlocked'} successfully`, 'success')
            }
            await loadUser(true)
            setPasswordConfirm({ show: false, target: null, newLockState: false, password: '' })
        } catch (error: any) {
            showToast(error.response?.data?.error || 'Failed to verify password', 'error')
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
                <button
                    className={`${styles.tabBtn} ${activeTab === 'widget' ? styles.activeTab : ''}`}
                    onClick={() => setActiveTab('widget')}
                >
                    <MdCode /> Checkout Widget
                </button>
                <button
                    className={`${styles.tabBtn} ${activeTab === 'security' ? styles.activeTab : ''}`}
                    onClick={() => setActiveTab('security')}
                >
                    <MdLock /> Security
                </button>
            </div>

            <div className={styles.content}>
                {activeTab === 'settlement' && (
                    <SettlementTab 
                        user={user}
                        selectedMode={selectedMode}
                        handleUpdateSettlementMode={handleUpdateSettlementMode}
                        handleToggleWalletLock={handleToggleWalletLock}
                        handleToggleCustomerWalletLock={handleToggleCustomerWalletLock}
                        addressOnlyCustomerPaysFee={addressOnlyCustomerPaysFee}
                        handleUpdateAddressOnlyFeeSetting={handleUpdateAddressOnlyFeeSetting}
                        loading={loading}
                        styles={styles}
                    />
                )}

                {activeTab === 'fees' && (
                    <FeesTab 
                        customerPaysFee={customerPaysFee}
                        handleUpdateFeeSetting={handleUpdateFeeSetting}
                        loading={loading}
                        styles={styles}
                    />
                )}

                {activeTab === 'api' && (
                    <ApiSettingsTab 
                        user={user}
                        apiKey={apiKey}
                        showApiKey={showApiKey}
                        setShowApiKey={setShowApiKey}
                        handleRotateKey={handleRotateKey}
                        copyToClipboard={copyToClipboard}
                        redirectUrl={redirectUrl}
                        setRedirectUrl={setRedirectUrl}
                        handleUpdateRedirect={handleUpdateRedirect}
                        ipWhitelist={ipWhitelist}
                        newIp={newIp}
                        setNewIp={setNewIp}
                        handleAddIp={handleAddIp}
                        handleRemoveIp={handleRemoveIp}
                        loading={loading}
                        styles={styles}
                    />
                )}

                {activeTab === 'webhooks' && (
                    <WebhooksTab 
                        webhookUrls={webhookUrls}
                        setWebhookUrls={setWebhookUrls}
                        webhookFormat={webhookFormat}
                        setWebhookFormat={setWebhookFormat}
                        handleUpdateWebhook={handleUpdateWebhook}
                        handleSendTestWebhook={handleSendTestWebhook}
                        signingSecret={signingSecret}
                        showSecret={showSecret}
                        setShowSecret={setShowSecret}
                        handleRotateSecret={handleRotateSecret}
                        loading={loading}
                        styles={styles}
                    />
                )}

                {activeTab === 'widget' && (
                    <WidgetTab styles={styles} />
                )}

                {activeTab === 'security' && (
                    <SecurityTab 
                        user={user}
                        pin={pin}
                        setPin={setPin}
                        confirmPin={confirmPin}
                        setConfirmPin={setConfirmPin}
                        handleSetPin={handleSetPin}
                        settingPin={settingPin}
                        lowBalanceThreshold={lowBalanceThreshold}
                        setLowBalanceThreshold={setLowBalanceThreshold}
                        lowBalanceAlertsEnabled={lowBalanceAlertsEnabled}
                        setLowBalanceAlertsEnabled={setLowBalanceAlertsEnabled}
                        handleUpdateSettings={handleUpdateSettings}
                        styles={styles}
                    />
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
            {/* Password Confirmation Modal */}
            {passwordConfirm.show && (
                <div className={styles.modalOverlay}>
                    <div className={styles.modal}>
                        <div className={styles.modalHeader}>
                            <h2><MdLock /> Security Confirmation</h2>
                            <button
                                className={styles.closeBtn}
                                onClick={() => setPasswordConfirm({ ...passwordConfirm, show: false })}
                                disabled={loading}
                            >
                                <MdClose />
                            </button>
                        </div>
                        <div className={styles.modalBody}>
                            <p>
                                You are about to <strong>{passwordConfirm.newLockState ? 'lock' : 'unlock'}</strong> your 
                                {passwordConfirm.target === 'wallet' ? ' primary ' : ' customer '} 
                                wallets. This is a sensitive security action.
                            </p>
                            <div className={styles.inputGroup} style={{ marginTop: '20px' }}>
                                <label style={{ fontSize: '14px', fontWeight: 600, color: '#374151' }}>
                                    Enter Account Password
                                </label>
                                <input
                                    type="password"
                                    value={passwordConfirm.password}
                                    onChange={(e) => setPasswordConfirm({ ...passwordConfirm, password: e.target.value as any })}
                                    placeholder="Your account password"
                                    className={styles.urlInput}
                                    autoFocus
                                    onKeyDown={(e) => e.key === 'Enter' && confirmLockAction()}
                                />
                            </div>
                            {!passwordConfirm.newLockState && (
                                <div className={styles.warningBox} style={{ marginTop: '15px' }}>
                                    <MdWarning />
                                    <p>Unlocking wallets allows changing destination addresses. Ensure you know what you are doing.</p>
                                </div>
                            )}
                        </div>
                        <div className={styles.modalActions}>
                            <button
                                className={styles.cancelBtn}
                                onClick={() => setPasswordConfirm({ ...passwordConfirm, show: false })}
                                disabled={loading}
                            >
                                Cancel
                            </button>
                            <button
                                className={styles.confirmRotateBtn}
                                onClick={confirmLockAction}
                                disabled={loading || !passwordConfirm.password}
                                style={{ backgroundColor: passwordConfirm.newLockState ? '#10b981' : '#3b82f6' }}
                            >
                                {loading ? (
                                    <>
                                        <MdRefresh className="animate-spin" /> Verifying...
                                    </>
                                ) : (
                                    <>
                                        {passwordConfirm.newLockState ? <MdLock /> : <MdLockOpen />}
                                        Confirm {passwordConfirm.newLockState ? 'Lock' : 'Unlock'}
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

// Helper icons not in MD
const MdLockOpen = (props: any) => (
    <svg stroke="currentColor" fill="currentColor" strokeWidth="0" viewBox="0 0 24 24" height="1em" width="1em" xmlns="http://www.w3.org/2000/svg" {...props}>
        <path fill="none" d="M0 0h24v24H0V0z"></path>
        <path d="M18 8h-1V6c0-2.76-2.24-5-5-5S7 3.24 7 6h2c0-1.66 1.34-3 3-3s3 1.34 3 3v2H6c-1.1 0-2 .9-2 2v10c0 1.1.9 2 2 2h12c1.1 0 2-.9 2-2V10c0-1.1-.9-2-2-2zm0 12H6V10h12v10zm-6-3c1.1 0 2-.9 2-2s-.9-2-2-2-2 .9-2 2 .9 2 2 2z"></path>
    </svg>
)

export default SettingsPage
