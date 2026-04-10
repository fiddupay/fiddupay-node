import React, { useState } from 'react'
import { useAuthStore } from '@/stores/authStore'

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

            <div className={styles.formGroup} style={{ marginTop: '20px' }}>
                <label className={styles.label}>Widget Theme</label>
                <select
                    className={styles.input}
                    value={theme}
                    onChange={(e) => setTheme(e.target.value)}
                    style={{ width: '200px' }}
                >
                    <option value="dark">Dark Mode</option>
                    <option value="light">Light Mode</option>
                </select>
            </div>

            <div style={{ marginTop: '20px', background: '#0f172a', padding: '16px', borderRadius: '8px', color: '#e2e8f0', fontSize: '13px', overflowX: 'auto', fontFamily: 'monospace' }}>
                <pre style={{ margin: 0 }}><code>{snippet}</code></pre>
            </div>

            <button
                className={styles.primaryBtn}
                onClick={handleCopy}
                style={{ marginTop: '16px' }}
            >
                Copy Code Snippet
            </button>

            <div style={{ marginTop: '24px', padding: '16px', background: '#f8fafc', borderRadius: '8px', border: '1px solid #e2e8f0' }}>
                <h4 style={{ fontWeight: 600, fontSize: '14px', marginBottom: '8px', color: '#1e293b' }}>Integration Steps:</h4>
                <ol style={{ fontSize: '13px', color: '#475569', paddingLeft: '20px', display: 'flex', flexDirection: 'column', gap: '8px', margin: 0 }}>
                    <li><strong style={{ color: '#0f172a' }}>Embed the Script:</strong> Place the FidduPay script onto your webpage so the widget loads seamlessly.</li>
                    <li><strong style={{ color: '#0f172a' }}>Launch Widget:</strong> Use <code>FidduPay.open({'{'} amount: 50, publicKey: '${pubKeyDisplay}' {'}'})</code> on any button click.</li>
                    <li><strong style={{ color: '#0f172a' }}>Secure Webhooks:</strong> Set up Webhooks in the Developer Dashboard so your pure-HTML site knows when items are securely paid for!</li>
                </ol>
            </div>
        </div>
    )
}

export default WidgetTab
