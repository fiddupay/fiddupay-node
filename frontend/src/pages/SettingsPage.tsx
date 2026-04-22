import React, { useState, useEffect } from 'react'
import {
    MdAccountBalanceWallet,
    MdCode,
    MdPayment,
    MdNotificationsActive,
    MdLock,
    MdShield
} from 'react-icons/md'
import { useLocation } from 'react-router-dom'
import { useAuthStore } from '@/stores/authStore'
import { SettingsSkeleton } from '@/components/layout/PageSkeletons'
import styles from '@/styles/pages/SettingsPage.module.css'
import SEO from '@/components/ui/SEO'

// Modular Tabs
import SettlementTab from '@/components/settings/tabs/SettlementTab'
import FeesTab from '@/components/settings/tabs/FeesTab'
import ApiSettingsTab from '@/components/settings/tabs/ApiSettingsTab'
import WebhooksTab from '@/components/settings/tabs/WebhooksTab'
import SecurityTab from '@/components/settings/tabs/SecurityTab'
import WidgetTab from '@/components/settings/tabs/WidgetTab'
import VerificationTab from '@/components/settings/tabs/VerificationTab'

type TabType = 'settlement' | 'fees' | 'api' | 'webhooks' | 'security' | 'widget' | 'verification'

const SettingsPage: React.FC = () => {
    const { user, loadUser, loading: authLoading } = useAuthStore()
    const [activeTab, setActiveTab] = useState<TabType>('settlement')
    const location = useLocation()

    useEffect(() => {
        // Initial load
        loadUser(true)
        
        // Handle tab deep-linking
        const params = new URLSearchParams(location.search)
        const tab = params.get('tab') as TabType
        if (tab && ['settlement', 'fees', 'api', 'webhooks', 'security', 'widget', 'verification'].includes(tab)) {
            setActiveTab(tab)
        }
    }, [location.search])

    if (authLoading && !user) {
        return <SettingsSkeleton />
    }

    return (
        <div className={styles.page}>
            <SEO 
                title="Merchant Settings" 
                description="Configure your settlement modes, fee preferences, API keys, and notification webhooks."
            />
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
                <button
                    className={`${styles.tabBtn} ${activeTab === 'verification' ? styles.activeTab : ''}`}
                    onClick={() => setActiveTab('verification')}
                >
                    <MdShield /> Verification
                </button>
            </div>

            <div className={styles.content}>
                {activeTab === 'settlement' && (
                    <SettlementTab user={user} styles={styles} />
                )}

                {activeTab === 'fees' && (
                    <FeesTab user={user} styles={styles} />
                )}

                {activeTab === 'api' && (
                    <ApiSettingsTab user={user} styles={styles} />
                )}

                {activeTab === 'webhooks' && (
                    <WebhooksTab user={user} styles={styles} />
                )}

                {activeTab === 'widget' && (
                    <WidgetTab styles={styles} />
                )}

                {activeTab === 'security' && (
                    <SecurityTab user={user} styles={styles} />
                )}

                {activeTab === 'verification' && (
                    <VerificationTab user={user} loading={false} styles={styles} />
                )}
            </div>
        </div>
    )
}

export default SettingsPage
