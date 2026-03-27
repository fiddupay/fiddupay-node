import React from 'react';

interface FeesTabProps {
    customerPaysFee: boolean;
    handleUpdateFeeSetting: (value: boolean) => Promise<void>;
    loading: boolean;
    styles: any;
}

const FeesTab: React.FC<FeesTabProps> = ({
    customerPaysFee,
    handleUpdateFeeSetting,
    loading,
    styles
}) => {
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
