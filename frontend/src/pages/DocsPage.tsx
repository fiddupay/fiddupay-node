import React, { useState } from 'react'
import styles from './DocsPage.module.css'

const DocsPage: React.FC = () => {
  const [activeTab, setActiveTab] = useState('overview')
  const [selectedLanguage, setSelectedLanguage] = useState('curl')

  const codeExamples = {
    curl: `curl -X POST https://api.payflow.com/v1/payments \\
  -H "Authorization: Bearer sk_test_..." \\
  -H "Content-Type: application/json" \\
  -d '{
    "amount": "100.00",
    "currency": "USDT",
    "network": "ethereum",
    "callback_url": "https://yoursite.com/webhook",
    "metadata": {
      "order_id": "order_123"
    }
  }'`,
    javascript: `const payflow = require('payflow-node');
payflow.apiKey = 'sk_test_...';

const payment = await payflow.payments.create({
  amount: '100.00',
  currency: 'USDT',
  network: 'ethereum',
  callback_url: 'https://yoursite.com/webhook',
  metadata: {
    order_id: 'order_123'
  }
});`,
    python: `import payflow
payflow.api_key = "sk_test_..."

payment = payflow.Payment.create(
    amount="100.00",
    currency="USDT",
    network="ethereum",
    callback_url="https://yoursite.com/webhook",
    metadata={
        "order_id": "order_123"
    }
)`,
    php: `<?php
require_once('vendor/autoload.php');
\\PayFlow\\PayFlow::setApiKey('sk_test_...');

$payment = \\PayFlow\\Payment::create([
    'amount' => '100.00',
    'currency' => 'USDT',
    'network' => 'ethereum',
    'callback_url' => 'https://yoursite.com/webhook',
    'metadata' => [
        'order_id' => 'order_123'
    ]
]);`
  }

  return (
    <div className={styles.docsPage}>
      <div className={styles.sidebar}>
        <div className={styles.sidebarHeader}>
          <h2>Documentation</h2>
        </div>
        <nav className={styles.sidebarNav}>
          <div className={styles.navSection}>
            <h3>Getting Started</h3>
            <a href="#overview" className={activeTab === 'overview' ? styles.active : ''} onClick={() => setActiveTab('overview')}>
              <i className="fas fa-play"></i> Overview
            </a>
            <a href="#authentication" className={activeTab === 'auth' ? styles.active : ''} onClick={() => setActiveTab('auth')}>
              <i className="fas fa-key"></i> Authentication
            </a>
            <a href="#quickstart" className={activeTab === 'quickstart' ? styles.active : ''} onClick={() => setActiveTab('quickstart')}>
              <i className="fas fa-rocket"></i> Quick Start
            </a>
          </div>
          <div className={styles.navSection}>
            <h3>API Reference</h3>
            <a href="#payments" className={activeTab === 'payments' ? styles.active : ''} onClick={() => setActiveTab('payments')}>
              <i className="fas fa-credit-card"></i> Payments
            </a>
            <a href="#webhooks" className={activeTab === 'webhooks' ? styles.active : ''} onClick={() => setActiveTab('webhooks')}>
              <i className="fas fa-webhook"></i> Webhooks
            </a>
            <a href="#wallets" className={activeTab === 'wallets' ? styles.active : ''} onClick={() => setActiveTab('wallets')}>
              <i className="fas fa-wallet"></i> Wallets
            </a>
          </div>
          <div className={styles.navSection}>
            <h3>Resources</h3>
            <a href="#sdks" className={activeTab === 'sdks' ? styles.active : ''} onClick={() => setActiveTab('sdks')}>
              <i className="fas fa-code"></i> SDKs
            </a>
            <a href="#errors" className={activeTab === 'errors' ? styles.active : ''} onClick={() => setActiveTab('errors')}>
              <i className="fas fa-exclamation-triangle"></i> Error Codes
            </a>
          </div>
        </nav>
      </div>

      <div className={styles.content}>
        {activeTab === 'overview' && (
          <div className={styles.section}>
            <h1>PayFlow API Documentation</h1>
            <p className={styles.lead}>
              The PayFlow API is organized around REST. Our API has predictable resource-oriented URLs, 
              accepts form-encoded request bodies, returns JSON-encoded responses, and uses standard HTTP response codes.
            </p>

            <div className={styles.infoBox}>
              <h3><i className="fas fa-info-circle"></i> Base URL</h3>
              <code>https://api.payflow.com/v1</code>
            </div>

            <h2>Supported Cryptocurrencies</h2>
            <div className={styles.cryptoGrid}>
              <div className={styles.cryptoCard}>
                <i className="fab fa-bitcoin"></i>
                <h4>SOL</h4>
                <p>Solana Network</p>
              </div>
              <div className={styles.cryptoCard}>
                <i className="fas fa-coins"></i>
                <h4>USDT</h4>
                <p>5 Networks: ETH, BSC, Polygon, Arbitrum, Solana</p>
              </div>
            </div>
          </div>
        )}

        {activeTab === 'auth' && (
          <div className={styles.section}>
            <h1>Authentication</h1>
            <p className={styles.lead}>
              PayFlow uses API keys to authenticate requests. You can view and manage your API keys in the Dashboard.
            </p>

            <div className={styles.warningBox}>
              <i className="fas fa-exclamation-triangle"></i>
              <strong>Keep your API keys secure!</strong> Do not share your secret API keys in publicly accessible areas.
            </div>

            <h2>API Key Types</h2>
            <div className={styles.keyTypes}>
              <div className={styles.keyType}>
                <h3>Publishable Keys</h3>
                <code>pk_test_...</code>
                <p>Used in client-side code. Safe to publish.</p>
              </div>
              <div className={styles.keyType}>
                <h3>Secret Keys</h3>
                <code>sk_test_...</code>
                <p>Used server-side. Must be kept confidential.</p>
              </div>
            </div>

            <h2>Making Authenticated Requests</h2>
            <div className={styles.codeExample}>
              <div className={styles.codeHeader}>
                <span>Authentication Header</span>
              </div>
              <pre><code>Authorization: Bearer sk_test_...</code></pre>
            </div>
          </div>
        )}

        {activeTab === 'payments' && (
          <div className={styles.section}>
            <h1>Payments API</h1>
            <p className={styles.lead}>
              Create and manage cryptocurrency payment requests.
            </p>

            <div className={styles.endpoint}>
              <div className={styles.endpointHeader}>
                <span className={styles.method}>POST</span>
                <span className={styles.url}>/v1/payments</span>
                <span className={styles.description}>Create a payment</span>
              </div>

              <h3>Parameters</h3>
              <div className={styles.paramTable}>
                <div className={styles.param}>
                  <code>amount</code>
                  <span className={styles.required}>required</span>
                  <p>Payment amount in USD (string)</p>
                </div>
                <div className={styles.param}>
                  <code>currency</code>
                  <span className={styles.required}>required</span>
                  <p>Cryptocurrency: "SOL" or "USDT"</p>
                </div>
                <div className={styles.param}>
                  <code>network</code>
                  <span className={styles.optional}>optional</span>
                  <p>Network for USDT: "ethereum", "bsc", "polygon", "arbitrum", "solana"</p>
                </div>
                <div className={styles.param}>
                  <code>callback_url</code>
                  <span className={styles.optional}>optional</span>
                  <p>Webhook URL for payment notifications</p>
                </div>
              </div>

              <h3>Example Request</h3>
              <div className={styles.codeExample}>
                <div className={styles.codeHeader}>
                  <div className={styles.languageTabs}>
                    {Object.keys(codeExamples).map(lang => (
                      <button
                        key={lang}
                        className={selectedLanguage === lang ? styles.active : ''}
                        onClick={() => setSelectedLanguage(lang)}
                      >
                        {lang.toUpperCase()}
                      </button>
                    ))}
                  </div>
                </div>
                <pre><code>{codeExamples[selectedLanguage as keyof typeof codeExamples]}</code></pre>
              </div>

              <h3>Response</h3>
              <div className={styles.codeExample}>
                <pre><code>{`{
  "id": "pay_1234567890",
  "amount": "100.00",
  "currency": "USDT",
  "network": "ethereum",
  "status": "pending",
  "payment_address": "0x742d35Cc6634C0532925a3b8D4C0532925a3b8D4",
  "qr_code": "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAA...",
  "expires_at": "2026-01-24T23:35:37Z",
  "created_at": "2026-01-24T22:35:37Z"
}`}</code></pre>
              </div>
            </div>
          </div>
        )}

        {activeTab === 'webhooks' && (
          <div className={styles.section}>
            <h1>Webhooks</h1>
            <p className={styles.lead}>
              PayFlow sends webhooks to notify your application when events happen in your account.
            </p>

            <div className={styles.infoBox}>
              <h3><i className="fas fa-bolt"></i> Real-time Notifications</h3>
              <p>Receive instant notifications when payments are confirmed, failed, or expired.</p>
            </div>

            <h2>Webhook Events</h2>
            <div className={styles.eventList}>
              <div className={styles.event}>
                <code>payment.confirmed</code>
                <p>Payment has been confirmed on the blockchain</p>
              </div>
              <div className={styles.event}>
                <code>payment.failed</code>
                <p>Payment has failed or been rejected</p>
              </div>
              <div className={styles.event}>
                <code>payment.expired</code>
                <p>Payment request has expired</p>
              </div>
            </div>

            <h2>Webhook Payload</h2>
            <div className={styles.codeExample}>
              <pre><code>{`{
  "event": "payment.confirmed",
  "data": {
    "id": "pay_1234567890",
    "amount": "100.00",
    "currency": "USDT",
    "network": "ethereum",
    "status": "confirmed",
    "transaction_hash": "0x1234...abcd",
    "confirmed_at": "2026-01-24T22:36:15Z"
  }
}`}</code></pre>
            </div>
          </div>
        )}

        {activeTab === 'sdks' && (
          <div className={styles.section}>
            <h1>Official SDKs</h1>
            <p className={styles.lead}>
              Use our official libraries to integrate PayFlow into your application.
            </p>

            <div className={styles.sdkGrid}>
              <div className={styles.sdkCard}>
                <i className="fab fa-node-js"></i>
                <h3>Node.js</h3>
                <code>npm install payflow-node</code>
                <a href="#" className={styles.sdkLink}>View Documentation</a>
              </div>
              <div className={styles.sdkCard}>
                <i className="fab fa-python"></i>
                <h3>Python</h3>
                <code>pip install payflow</code>
                <a href="#" className={styles.sdkLink}>View Documentation</a>
              </div>
              <div className={styles.sdkCard}>
                <i className="fab fa-php"></i>
                <h3>PHP</h3>
                <code>composer require payflow/payflow-php</code>
                <a href="#" className={styles.sdkLink}>View Documentation</a>
              </div>
              <div className={styles.sdkCard}>
                <i className="fab fa-java"></i>
                <h3>Java</h3>
                <code>implementation 'com.payflow:payflow-java'</code>
                <a href="#" className={styles.sdkLink}>View Documentation</a>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  )
}

export default DocsPage
