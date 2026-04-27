import React, { useState } from 'react'
import { useAuthStore } from '@/stores/authStore'
import CustomSelect from '@/components/ui/CustomSelect'
import { MdVisibility } from 'react-icons/md'

interface WidgetTabProps {
    styles: any;
}

const WidgetTab: React.FC<WidgetTabProps> = ({ styles }) => {
    const { user } = useAuthStore()
    const [theme, setTheme] = useState('dark')

    const pubKey = user?.sandbox_mode ? user?.test_publishable_key : user?.live_publishable_key;
    const pubKeyDisplay = pubKey && pubKey !== 'PENDING' ? pubKey : 'pub_live_YOUR_KEY_HERE';

    const snippet = `<!-- 1. Include this early in your <head> or <body> -->
<script src="https://pay.fiddupay.com/widget.js"></script>

<!-- 2. Attach an onclick event triggering the Widget with your Public Key -->
<button onclick="FidduPay.open({ amount: 50, publicKey: '${pubKeyDisplay}', theme: '${theme}' })">
    Buy Now ($50)
</button>

<!-- 3. (Optional) Provide a callback listener -->
<script>
    window.addEventListener('message', (event) => {
        // Ensure you verify the origin!
        // if (event.origin !== "https://pay.fiddupay.com") return;
        
        if (event.data?.type === 'FIDDUPAY_SUCCESS') {
            console.log("Payment Confirmed!", event.data.payload.paymentId);
            // Example: Make a backend verification call to FidduPay
        }
    });
</script>`

    const handleCopy = () => {
        navigator.clipboard.writeText(snippet)
        alert('Widget snippet copied to clipboard!')
    }

    return (
        <div className={styles.sectionCard}>
            <div className={styles.sectionHeader}>
                <h3 className={styles.sectionTitle}>No-Code Checkout Widget</h3>
                <p className={styles.sectionDescription}>
                    Allow customers to seamlessly pay with Crypto on your website without redirecting them.
                </p>
            </div>

            <div className={styles.formGroup} style={{ marginTop: '20px', maxWidth: '300px' }}>
                <CustomSelect
                    label="Widget Theme"
                    options={[
                        { value: 'dark', label: 'Dark Mode' },
                        { value: 'light', label: 'Light Mode' }
                    ]}
                    value={theme}
                    onChange={(val) => setTheme(val)}
                />
            </div>

            <div style={{ marginTop: '20px', background: 'var(--bg-main)', padding: '16px', border: '1px solid var(--border)', borderRadius: '8px', color: 'var(--text-main)', fontSize: '13px', overflowX: 'auto', fontFamily: 'monospace' }}>
                <pre style={{ margin: 0, whiteSpace: 'pre-wrap', wordBreak: 'break-all' }}><code>{snippet}</code></pre>
            </div>

            <button
                className={styles.primaryBtn}
                onClick={handleCopy}
                style={{ marginTop: '16px' }}
            >
                Copy Code Snippet
            </button>

            <div style={{ marginTop: '24px', padding: '16px', background: 'var(--surface-hover)', borderRadius: '12px', border: '1px solid var(--border)' }}>
                <h4 style={{ fontWeight: 600, fontSize: '14px', marginBottom: '12px', color: 'var(--text-main)' }}>Integration Steps:</h4>
                <ol style={{ fontSize: '13px', color: 'var(--text-muted)', paddingLeft: '20px', display: 'flex', flexDirection: 'column', gap: '8px', margin: 0 }}>
                    <li><strong style={{ color: 'var(--text-main)' }}>Embed the Script:</strong> Place the FidduPay script onto your webpage so the widget loads seamlessly.</li>
                    <li><strong style={{ color: 'var(--text-main)' }}>Launch Widget:</strong> Use <code>FidduPay.open({'{'} amount: 50, publicKey: '${pubKeyDisplay}' {'}'})</code> on any button click.</li>
                    <li><strong style={{ color: 'var(--text-main)' }}>Secure Webhooks:</strong> Set up Webhooks in the Developer Dashboard so your pure-HTML site knows when items are securely paid for!</li>
                </ol>
            </div>

            {/* Live Preview */}
            <div style={{ marginTop: '32px' }}>
                <h4 style={{ fontWeight: 600, fontSize: '15px', marginBottom: '16px', color: 'var(--text-main)', display: 'flex', alignItems: 'center', gap: '8px' }}>
                    <MdVisibility style={{ color: 'var(--primary)' }} size={20} />
                    Live Widget Preview
                </h4>
                <div style={{ 
                    width: '100%', 
                    height: '650px', 
                    border: '1px solid var(--border)', 
                    borderRadius: '16px', 
                    overflow: 'hidden', 
                    background: 'rgba(0,0,0,0.02)',
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'center',
                    boxShadow: 'inset 0 2px 4px 0 rgba(0, 0, 0, 0.05)'
                }}>
                    <iframe 
                        src={`${import.meta.env.VITE_API_URL || ''}/preview/widget`} 
                        style={{ width: '100%', height: '100%', border: 'none' }}
                        title="Widget Preview"
                    />
                </div>
            </div>
        </div>
    )
}

export default WidgetTab
