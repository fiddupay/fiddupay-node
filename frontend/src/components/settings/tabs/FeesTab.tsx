import React, { useState, useEffect } from 'react';
import { merchantAPI } from '@/services/apiService';
import { useToast } from '@/contexts/ToastContext';
import { useAuthStore } from '@/stores/authStore';

interface FeesTabProps {
    user: any;
    styles: any;
}

const FeesTab: React.FC<FeesTabProps> = ({ user, styles }) => {
    const { showToast } = useToast();
    const { loadUser } = useAuthStore();
    const [loading, setLoading] = useState(false);
    const [customerPaysFee, setCustomerPaysFee] = useState(user?.customer_pays_fee || false);

    useEffect(() => {
        if (user) {
            setCustomerPaysFee(user.customer_pays_fee || false);
        }
    }, [user]);

    const handleUpdateFeeSetting = async (newValue: boolean) => {
        try {
            setLoading(true);
            await merchantAPI.updateSettings({ customer_pays_fee: newValue });
            setCustomerPaysFee(newValue);
            await loadUser(true);
            showToast(`Fees will now be paid by ${newValue ? 'customers' : 'you'}`, 'success');
        } catch (error: any) {
            showToast('Failed to update fee preferences', 'error');
        } finally {
            setLoading(false);
        }
    };

    return (
        <section className={styles.section}>
            <h2>Fee Preferences</h2>
            <p>Control who pays the transaction fees for your payments.</p>

            <div className={styles.toggleGroup}>
                <div className={styles.toggleLabel}>
                    <h4>Pass Fee to Customer</h4>
                    <span>If enabled, the transaction fee will be added to the customer's total amount.</span>
                </div>
                <label className={styles.switch}>
                    <input 
                        type="checkbox" 
                        checked={customerPaysFee}
                        onChange={(e) => handleUpdateFeeSetting(e.target.checked)}
                        disabled={loading}
                    />
                    <span className={`${styles.slider} ${styles.round}`}></span>
                </label>
            </div>
        </section>
    );
};

export default FeesTab;
