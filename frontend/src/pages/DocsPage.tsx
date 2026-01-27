import React, { useState } from 'react'
import styles from './DocsPage.module.css'

const DocsPage: React.FC = () => {
  const [activeTab, setActiveTab] = useState('overview')
  const [expandedEndpoint, setExpandedEndpoint] = useState<string | null>(null)

  const toggleEndpoint = (endpointId: string) => {
    setExpandedEndpoint(expandedEndpoint === endpointId ? null : endpointId)
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
              Quick Start
            </a>
            <a href="#sandbox" className={activeTab === 'sandbox' ? styles.active : ''} onClick={() => setActiveTab('sandbox')}>
              Sandbox & Testing
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
            <a href="#refunds" className={activeTab === 'refunds' ? styles.active : ''} onClick={() => setActiveTab('refunds')}>
              Refunds
            </a>
            <a href="#analytics" className={activeTab === 'analytics' ? styles.active : ''} onClick={() => setActiveTab('analytics')}>
              Analytics
            </a>
            <a href="#merchants" className={activeTab === 'merchants' ? styles.active : ''} onClick={() => setActiveTab('merchants')}>
              Merchants
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
        {/* Quick Links Section */}
        <div className={styles.quickLinksSection}>
          <h2>Documentation Resources</h2>
          <div className={styles.quickLinksGrid}>
            <a href="https://github.com/CHToken/crypto-payment-gateway/blob/main/docs/API_REFERENCE.md" target="_blank" rel="noopener noreferrer" className={styles.quickLinkCard}>
              <i className="fas fa-book"></i>
              <h3>Complete API Reference</h3>
              <p>Full documentation of all merchant API endpoints with examples</p>
            </a>
            <a href="https://github.com/CHToken/crypto-payment-gateway/blob/main/docs/NODE_SDK.md" target="_blank" rel="noopener noreferrer" className={styles.quickLinkCard}>
              <i className="fas fa-code"></i>
              <h3>Node.js SDK</h3>
              <p>Official SDK for easy integration with Node.js applications</p>
            </a>
            <a href="https://github.com/CHToken/crypto-payment-gateway/blob/main/docs/MERCHANT_GUIDE.md" target="_blank" rel="noopener noreferrer" className={styles.quickLinkCard}>
              <i className="fas fa-user-tie"></i>
              <h3>Merchant Integration Guide</h3>
              <p>Step-by-step guide for integrating FidduPay into your application</p>
            </a>
          </div>
        </div>

        {activeTab === 'overview' && (
          <div className={styles.section}>
            <h1>FidduPay API Documentation</h1>
            <p className={styles.lead}>
              The FidduPay API is organized around REST. Our API has predictable resource-oriented URLs, 
              accepts JSON request bodies, returns JSON-encoded responses, and uses standard HTTP response codes.
            </p>

            <div className={styles.infoBox}>
              <h3>Base URLs</h3>
              <p><strong>Sandbox:</strong> <code>https://api-sandbox.fiddupay.com/v1</code></p>
              <p><strong>Production:</strong> <code>https://api.fiddupay.com/v1</code></p>
            </div>

            <h2 className={styles.bigTitle}>Supported Cryptocurrencies & Networks</h2>
            <div className={styles.cryptoGrid}>
              <div className={styles.cryptoCard}>
                <div className={styles.cryptoIcon}>
                  <img src="https://raw.githubusercontent.com/spothq/cryptocurrency-icons/master/128/color/sol.png" alt="Solana" />
                </div>
                <h4>SOL</h4>
                <p>Solana Network</p>
                <small>32 confirmations (~13 seconds)</small>
              </div>
              <div className={styles.cryptoCard}>
                <div className={styles.cryptoIcon}>
                  <img src="https://raw.githubusercontent.com/spothq/cryptocurrency-icons/master/128/color/eth.png" alt="Ethereum" />
                </div>
                <h4>ETH</h4>
                <p>Ethereum Network</p>
                <small>12 confirmations (~3 minutes)</small>
              </div>
              <div className={styles.cryptoCard}>
                <div className={styles.cryptoIcon}>
                  <img src="https://raw.githubusercontent.com/spothq/cryptocurrency-icons/master/128/color/usdt.png" alt="USDT" />
                </div>
                <h4>USDT</h4>
                <p>Ethereum Network</p>
                <small>12 confirmations (~3 minutes)</small>
              </div>
              <div className={styles.cryptoCard}>
                <div className={styles.cryptoIcon}>
                  <img src="https://raw.githubusercontent.com/spothq/cryptocurrency-icons/master/128/color/bnb.png" alt="BNB" />
                </div>
                <h4>BNB</h4>
                <p>BSC Network</p>
                <small>15 confirmations (~45 seconds)</small>
              </div>
              <div className={styles.cryptoCard}>
                <div className={styles.cryptoIcon}>
                  <img src="https://raw.githubusercontent.com/spothq/cryptocurrency-icons/master/128/color/usdt.png" alt="USDT BSC" />
                </div>
                <h4>USDT</h4>
                <p>BSC Network</p>
                <small>15 confirmations (~45 seconds)</small>
              </div>
              <div className={styles.cryptoCard}>
                <div className={styles.cryptoIcon}>
                  <img src="https://raw.githubusercontent.com/spothq/cryptocurrency-icons/master/128/color/matic.png" alt="Polygon" />
                </div>
                <h4>MATIC</h4>
                <p>Polygon Network</p>
                <small>30 confirmations (~1 minute)</small>
              </div>
              <div className={styles.cryptoCard}>
                <div className={styles.cryptoIcon}>
                  <img src="https://raw.githubusercontent.com/spothq/cryptocurrency-icons/master/128/color/usdt.png" alt="USDT Polygon" />
                </div>
                <h4>USDT</h4>
                <p>Polygon Network</p>
                <small>128 confirmations (~4 minutes)</small>
              </div>
              <div className={styles.cryptoCard}>
                <div className={styles.cryptoIcon}>
                  <img src="https://cryptologos.cc/logos/arbitrum-arb-logo.png" alt="Arbitrum" />
                </div>
                <h4>ARB</h4>
                <p>Arbitrum Network</p>
                <small>1 confirmation (~1 second)</small>
              </div>
              <div className={styles.cryptoCard}>
                <div className={styles.cryptoIcon}>
                  <img src="https://raw.githubusercontent.com/spothq/cryptocurrency-icons/master/128/color/usdt.png" alt="USDT Arbitrum" />
                </div>
                <h4>USDT</h4>
                <p>Arbitrum Network</p>
                <small>1 confirmation (~250ms)</small>
              </div>
              <div className={styles.cryptoCard}>
                <div className={styles.cryptoIcon}>
                  <img src="https://raw.githubusercontent.com/spothq/cryptocurrency-icons/master/128/color/usdt.png" alt="USDT Solana" />
                </div>
                <h4>USDT</h4>
                <p>Solana SPL</p>
                <small>32 confirmations (~13 seconds)</small>
              </div>
            </div>

            <h2>Payment Status Flow</h2>
            <div className={styles.statusFlow}>
              <div className={styles.statusItem}>
                <code>PENDING</code>
                <p>Payment created, waiting for transaction</p>
              </div>
              <div className={styles.statusArrow}>→</div>
              <div className={styles.statusItem}>
                <code>CONFIRMING</code>
                <p>Transaction detected, waiting for confirmations</p>
              </div>
              <div className={styles.statusArrow}>→</div>
              <div className={styles.statusItem}>
                <code>CONFIRMED</code>
                <p>Payment confirmed and verified</p>
              </div>
            </div>
          </div>
        )}

        {activeTab === 'auth' && (
          <div className={styles.section}>
            <h1>Authentication</h1>
            <p className={styles.lead}>
              FidduPay uses API keys to authenticate requests. You can view and manage your API keys in the Dashboard.
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

        {activeTab === 'quickstart' && (
          <div className={styles.section}>
            <h1>Quick Start Guide</h1>
            <p className={styles.lead}>
              Get started with FidduPay in minutes. This guide will walk you through creating your first payment.
            </p>

            <h2 className={styles.bigTitle}>Step 1: Create Account</h2>
            <div className={styles.step}>
              <div className={styles.stepNumber}>1</div>
              <div className={styles.stepContent}>
                <h3>Sign Up</h3>
                <p>Create your FidduPay merchant account and verify your email address.</p>
                <div className={styles.codeExample}>
                  <pre><code>{`curl -X POST https://api.fiddupay.com/v1/merchants/register \\
  -H "Content-Type: application/json" \\
  -d '{
    "email": "merchant@example.com",
    "business_name": "Your Business Name"
  }'`}</code></pre>
                </div>
              </div>
            </div>

            <h2 className={styles.bigTitle}>Step 2: Get API Keys</h2>
            <div className={styles.step}>
              <div className={styles.stepNumber}>2</div>
              <div className={styles.stepContent}>
                <h3>Generate API Keys</h3>
                <p>After registration, you'll receive your API keys. Start with test keys for development.</p>
                <div className={styles.keyExample}>
                  <strong>Test Keys:</strong><br/>
                  <code>sk_test_...</code> (Secret Key)<br/>
                  <code>pk_test_...</code> (Publishable Key)
                </div>
              </div>
            </div>

            <h2 className={styles.bigTitle}>Step 3: Create Your First Payment</h2>
            <div className={styles.step}>
              <div className={styles.stepNumber}>3</div>
              <div className={styles.stepContent}>
                <h3>Make API Call</h3>
                <p>Create a payment request for $100 USDT on Ethereum:</p>
                <div className={styles.codeExample}>
                  <pre><code>{`curl -X POST https://api.fiddupay.com/v1/payments \\
  -H "Authorization: Bearer sk_test_..." \\
  -H "Content-Type: application/json" \\
  -d '{
    "amount_usd": "100.00",
    "crypto_type": "USDT_ETH",
    "description": "Test Payment",
    "expiration_minutes": 30
  }'`}</code></pre>
                </div>
              </div>
            </div>

            <h2 className={styles.bigTitle}>Step 4: Handle the Response</h2>
            <div className={styles.step}>
              <div className={styles.stepNumber}>4</div>
              <div className={styles.stepContent}>
                <h3>Payment Created</h3>
                <p>You'll receive a payment object with deposit address and QR code:</p>
                <div className={styles.codeExample}>
                  <pre><code>{`{
  "payment_id": "pay_1234567890",
  "status": "PENDING",
  "amount": "100.00000000",
  "amount_usd": "100.00",
  "crypto_type": "USDT_ETH",
  "deposit_address": "0x742d35Cc6634C0532925a3b8D4C0532925a3b8D4",
  "payment_link": "https://pay.fiddupay.com/pay_1234567890",
  "qr_code_data": "ethereum:0x742d35Cc...",
  "expires_at": "2026-01-25T00:30:52Z"
}`}</code></pre>
                </div>
              </div>
            </div>

            <div className={styles.nextSteps}>
              <h2 className={styles.bigTitle}>Next Steps</h2>
              <div className={styles.nextStepGrid}>
                <div className={styles.nextStep}>
                  <i className="fas fa-cog"></i>
                  <h3>Configure Wallets</h3>
                  <p>Set up your wallet addresses for automatic forwarding</p>
                </div>
                <div className={styles.nextStep}>
                  <i className="fas fa-flask"></i>
                  <h3>Test in Sandbox</h3>
                  <p>Use sandbox mode to test without real transactions</p>
                </div>
                <div className={styles.nextStep}>
                  <i className="fas fa-rocket"></i>
                  <h3>Go Live</h3>
                  <p>Switch to production keys when ready</p>
                </div>
              </div>
            </div>
          </div>
        )}

        {activeTab === 'sandbox' && (
          <div className={styles.section}>
            <h1>Sandbox & Testing</h1>
            <p className={styles.lead}>
              FidduPay provides a comprehensive sandbox environment for testing your integration without real transactions.
            </p>

            <h2 className={styles.bigTitle}>Sandbox Environment</h2>
            <div className={styles.infoBox}>
              <h3>Test Safely</h3>
              <p>Use sandbox mode to test your integration without real cryptocurrency transactions or fees.</p>
            </div>

            <div className={styles.endpoint}>
              <div className={styles.endpointHeader}>
                <span className={styles.method}>POST</span>
                <span className={styles.url}>/v1/sandbox/enable</span>
                <span className={styles.description}>Enable sandbox mode</span>
              </div>

              <h3>Response</h3>
              <div className={styles.codeExample}>
                <pre><code>{`{
  "sandbox_mode": true,
  "message": "Sandbox mode enabled. All transactions will be simulated."
}`}</code></pre>
              </div>
            </div>

            <div className={styles.endpoint}>
              <div className={styles.endpointHeader}>
                <span className={styles.method}>POST</span>
                <span className={styles.url}>/v1/sandbox/payments/:payment_id/simulate</span>
                <span className={styles.description}>Simulate payment completion</span>
              </div>

              <h3>Parameters</h3>
              <div className={styles.paramTable}>
                <div className={styles.param}>
                  <code>status</code>
                  <span className={styles.required}>required</span>
                  <p>Simulation status: "confirmed", "failed", "expired"</p>
                </div>
                <div className={styles.param}>
                  <code>transaction_hash</code>
                  <span className={styles.optional}>optional</span>
                  <p>Mock transaction hash for testing</p>
                </div>
              </div>
            </div>

            <h2 className={styles.bigTitle}>Environment Differences</h2>
            <div className={styles.environmentGrid}>
              <div className={styles.environmentCard}>
                <h3><i className="fas fa-flask"></i> Sandbox Mode</h3>
                <ul>
                  <li>API keys start with <code>sk_test_</code></li>
                  <li>No real blockchain transactions</li>
                  <li>Simulated payment confirmations</li>
                  <li>Test webhook deliveries</li>
                  <li>No actual fees charged</li>
                  <li>Instant payment simulation</li>
                </ul>
              </div>
              <div className={styles.environmentCard}>
                <h3> Production Mode</h3>
                <ul>
                  <li>API keys start with <code>sk_live_</code></li>
                  <li>Real blockchain transactions</li>
                  <li>Actual cryptocurrency transfers</li>
                  <li>Real webhook notifications</li>
                  <li>Transaction fees apply</li>
                  <li>Network confirmation times</li>
                </ul>
              </div>
            </div>

            <h2 className={styles.bigTitle}>Testing Best Practices</h2>
            <div className={styles.testingTips}>
              <div className={styles.tip}>
                <h3>1. Start with Sandbox</h3>
                <p>Always test your integration in sandbox mode before going live.</p>
              </div>
              <div className={styles.tip}>
                <h3>2. Test All Scenarios</h3>
                <p>Simulate successful payments, failures, and expirations.</p>
              </div>
              <div className={styles.tip}>
                <h3>3. Webhook Testing</h3>
                <p>Verify your webhook endpoints handle all event types correctly.</p>
              </div>
              <div className={styles.tip}>
                <h3>4. Error Handling</h3>
                <p>Test how your application handles API errors and network issues.</p>
              </div>
            </div>
          </div>
        )}

        {activeTab === 'payments' && (
          <div className={styles.section}>
            <h1><i className="fas fa-credit-card"></i> Payments API</h1>
            <p className={styles.lead}>
              Create and manage cryptocurrency payment requests with automatic forwarding and real-time notifications.
            </p>

            <div className={styles.endpointCard}>
              <div 
                className={styles.endpointHeader}
                onClick={() => toggleEndpoint('create-payment')}
              >
                <div className={styles.endpointMethod}>
                  <span className={styles.methodPost}>POST</span>
                  <code>/v1/payments</code>
                </div>
                <div className={styles.endpointTitle}>Create Payment</div>
                <i className={`fas fa-chevron-${expandedEndpoint === 'create-payment' ? 'up' : 'down'}`}></i>
              </div>
              
              {expandedEndpoint === 'create-payment' && (
                <div className={styles.endpointContent}>
                  <h3><i className="fas fa-cog"></i> Request Parameters</h3>
                  <div className={styles.paramTable}>
                    <div className={styles.paramRow}>
                      <code>amount_usd</code>
                      <span className={styles.required}>required</span>
                      <span>Payment amount in USD (string)</span>
                    </div>
                    <div className={styles.paramRow}>
                      <code>crypto_type</code>
                      <span className={styles.required}>required</span>
                      <span>Cryptocurrency: SOL, USDT_ETH, USDT_BEP20, USDT_POLYGON, USDT_ARBITRUM, USDT_SPL</span>
                    </div>
                    <div className={styles.paramRow}>
                      <code>description</code>
                      <span className={styles.optional}>optional</span>
                      <span>Payment description for reference</span>
                    </div>
                    <div className={styles.paramRow}>
                      <code>metadata</code>
                      <span className={styles.optional}>optional</span>
                      <span>Custom metadata object</span>
                    </div>
                    <div className={styles.paramRow}>
                      <code>expiration_minutes</code>
                      <span className={styles.optional}>optional</span>
                      <span>Payment expiration time (default: 20)</span>
                    </div>
                  </div>

                  <h3><i className="fas fa-arrow-left"></i> Response</h3>
                  <div className={styles.codeExample}>
                    <pre><code>{`{
  "payment_id": "pay_1234567890",
  "status": "PENDING",
  "amount": "0.00234567",
  "amount_usd": "100.00",
  "crypto_type": "USDT_ETH",
  "network": "ETHEREUM",
  "deposit_address": "0x742d35Cc6634C0532925a3b8D4C0532925a3b8D4",
  "payment_link": "https://pay.fiddupay.com/pay_1234567890",
  "qr_code_data": "ethereum:0x742d35Cc6634C0532925a3b8D4C0532925a3b8D4?value=234567000000000000",
  "fee_amount": "0.00001756",
  "fee_amount_usd": "0.75",
  "expires_at": "2026-01-24T23:35:37Z",
  "created_at": "2026-01-24T22:35:37Z"
}`}</code></pre>
                  </div>
                </div>
              )}
            </div>

            <div className={styles.endpointCard}>
              <div 
                className={styles.endpointHeader}
                onClick={() => toggleEndpoint('get-payment')}
              >
                <div className={styles.endpointMethod}>
                  <span className={styles.methodGet}>GET</span>
                  <code>/v1/payments/:id</code>
                </div>
                <div className={styles.endpointTitle}>Get Payment</div>
                <i className={`fas fa-chevron-${expandedEndpoint === 'get-payment' ? 'up' : 'down'}`}></i>
              </div>
              
              {expandedEndpoint === 'get-payment' && (
                <div className={styles.endpointContent}>
                  <h3><i className="fas fa-route"></i> Path Parameters</h3>
                  <div className={styles.paramTable}>
                    <div className={styles.paramRow}>
                      <code>id</code>
                      <span className={styles.required}>required</span>
                      <span>Payment ID</span>
                    </div>
                  </div>

                  <h3><i className="fas fa-arrow-left"></i> Response</h3>
                  <div className={styles.codeExample}>
                    <pre><code>{`{
  "payment_id": "pay_1234567890",
  "status": "CONFIRMED",
  "amount": "0.00234567",
  "amount_usd": "100.00",
  "crypto_type": "USDT_ETH",
  "network": "ETHEREUM",
  "deposit_address": "0x742d35Cc6634C0532925a3b8D4C0532925a3b8D4",
  "transaction_hash": "0xabc123def456...",
  "confirmations": 12,
  "confirmed_at": "2026-01-24T22:40:15Z",
  "created_at": "2026-01-24T22:35:37Z"
}`}</code></pre>
                  </div>
                </div>
              )}
            </div>

            <div className={styles.endpointCard}>
              <div 
                className={styles.endpointHeader}
                onClick={() => toggleEndpoint('list-payments')}
              >
                <div className={styles.endpointMethod}>
                  <span className={styles.methodGet}>GET</span>
                  <code>/v1/payments</code>
                </div>
                <div className={styles.endpointTitle}>List Payments</div>
                <i className={`fas fa-chevron-${expandedEndpoint === 'list-payments' ? 'up' : 'down'}`}></i>
              </div>
              
              {expandedEndpoint === 'list-payments' && (
                <div className={styles.endpointContent}>
                  <h3><i className="fas fa-filter"></i> Query Parameters</h3>
                  <div className={styles.paramTable}>
                    <div className={styles.paramRow}>
                      <code>page</code>
                      <span className={styles.optional}>optional</span>
                      <span>Page number (default: 1)</span>
                    </div>
                    <div className={styles.paramRow}>
                      <code>limit</code>
                      <span className={styles.optional}>optional</span>
                      <span>Items per page (default: 20, max: 100)</span>
                    </div>
                    <div className={styles.paramRow}>
                      <code>status</code>
                      <span className={styles.optional}>optional</span>
                      <span>Filter by status: PENDING, CONFIRMING, CONFIRMED, FAILED, REFUNDED</span>
                    </div>
                    <div className={styles.paramRow}>
                      <code>crypto_type</code>
                      <span className={styles.optional}>optional</span>
                      <span>Filter by cryptocurrency type</span>
                    </div>
                  </div>

                  <h3><i className="fas fa-arrow-left"></i> Response</h3>
                  <div className={styles.codeExample}>
                    <pre><code>{`{
  "payments": [
    {
      "payment_id": "pay_1234567890",
      "status": "CONFIRMED",
      "amount_usd": "100.00",
      "crypto_type": "USDT_ETH",
      "created_at": "2026-01-24T22:35:37Z"
    }
  ],
  "pagination": {
    "page": 1,
    "limit": 20,
    "total": 1,
    "has_more": false
  }
}`}</code></pre>
                  </div>
                </div>
              )}
            </div>

            <div className={styles.infoBox}>
              <h3><i className="fas fa-percentage"></i> Fee Toggle System</h3>
              <p>
                FidduPay supports flexible fee payment models. Merchants can choose who pays the processing fee:
              </p>
              <ul>
                <li><strong>Customer Pays Fee (Default):</strong> Customer pays requested amount + processing fee</li>
                <li><strong>Merchant Pays Fee:</strong> Customer pays requested amount, merchant receives requested amount - processing fee</li>
              </ul>
            </div>

            <div className={styles.endpointCard}>
              <div 
                className={styles.endpointHeader}
                onClick={() => toggleEndpoint('update-fee-setting')}
              >
                <div className={styles.endpointMethod}>
                  <span className={styles.methodPost}>POST</span>
                  <code>/v1/fee-setting</code>
                </div>
                <div className={styles.endpointTitle}>Update Fee Setting</div>
                <i className={`fas fa-chevron-${expandedEndpoint === 'update-fee-setting' ? 'up' : 'down'}`}></i>
              </div>
              
              {expandedEndpoint === 'update-fee-setting' && (
                <div className={styles.endpointContent}>
                  <h3><i className="fas fa-cog"></i> Request Parameters</h3>
                  <div className={styles.paramTable}>
                    <div className={styles.paramRow}>
                      <code>customer_pays_fee</code>
                      <span className={styles.required}>required</span>
                      <span>Boolean: true for customer pays fee, false for merchant pays fee</span>
                    </div>
                  </div>

                  <h3><i className="fas fa-arrow-left"></i> Response</h3>
                  <div className={styles.codeExample}>
                    <pre><code>{`{
  "success": true,
  "customer_pays_fee": true,
  "message": "Fee payment setting updated: Customer pays fee"
}`}</code></pre>
                  </div>
                </div>
              )}
            </div>
          </div>
        )}

        {activeTab === 'webhooks' && (
          <div className={styles.section}>
            <h1>Webhooks</h1>
            <p className={styles.lead}>
              FidduPay sends webhooks to notify your application when events happen in your account.
              Webhooks are delivered with automatic retry logic and delivery tracking.
            </p>

            <div className={styles.infoBox}>
              <h3>Webhook Configuration</h3>
              <p>Configure your webhook URL in the merchant dashboard or via API:</p>
              <code>PUT /v1/merchants/webhook</code>
            </div>

            <h2>Webhook Events</h2>
            <div className={styles.eventList}>
              <div className={styles.event}>
                <code>payment.confirmed</code>
                <p>Payment has been confirmed on the blockchain with required confirmations</p>
              </div>
              <div className={styles.event}>
                <code>payment.expired</code>
                <p>Payment request has expired without receiving payment</p>
              </div>
              <div className={styles.event}>
                <code>payment.failed</code>
                <p>Payment has failed due to insufficient amount or other issues</p>
              </div>
              <div className={styles.event}>
                <code>refund.completed</code>
                <p>Refund has been successfully processed and sent</p>
              </div>
              <div className={styles.event}>
                <code>refund.failed</code>
                <p>Refund processing has failed</p>
              </div>
            </div>

            <h2>Webhook Payload</h2>
            <div className={styles.codeExample}>
              <pre><code>{`{
  "event_type": "payment.confirmed",
  "payment_id": "pay_1234567890",
  "merchant_id": 12345,
  "status": "CONFIRMED",
  "amount": "0.00234567",
  "crypto_type": "USDT_ETH",
  "transaction_hash": "0x1234...abcd",
  "timestamp": 1706135777
}`}</code></pre>
            </div>

            <h2>Webhook Security</h2>
            <div className={styles.warningBox}>
              <strong>Verify webhook signatures!</strong> Always verify the webhook signature to ensure the request is from FidduPay.
            </div>

            <h2>Delivery & Retries</h2>
            <ul>
              <li>Webhooks are delivered with a 10-second timeout</li>
              <li>Failed deliveries are retried up to 5 times with exponential backoff</li>
              <li>Delivery status is tracked and available in your dashboard</li>
              <li>Webhooks expect a 2xx HTTP response to be considered successful</li>
            </ul>
          </div>
        )}

        {activeTab === 'refunds' && (
          <div className={styles.section}>
            <h1>Refunds API</h1>
            <p className={styles.lead}>
              Process refunds for confirmed payments. Refunds are processed automatically to the original payment address.
            </p>

            <div className={styles.endpoint}>
              <div className={styles.endpointHeader}>
                <span className={styles.method}>POST</span>
                <span className={styles.url}>/v1/refunds</span>
                <span className={styles.description}>Create a refund</span>
              </div>

              <h3>Parameters</h3>
              <div className={styles.paramTable}>
                <div className={styles.param}>
                  <code>payment_id</code>
                  <span className={styles.required}>required</span>
                  <p>ID of the payment to refund</p>
                </div>
                <div className={styles.param}>
                  <code>amount</code>
                  <span className={styles.optional}>optional</span>
                  <p>Partial refund amount (defaults to full payment amount)</p>
                </div>
                <div className={styles.param}>
                  <code>reason</code>
                  <span className={styles.optional}>optional</span>
                  <p>Reason for the refund</p>
                </div>
              </div>

              <h3>Response</h3>
              <div className={styles.codeExample}>
                <pre><code>{`{
  "refund_id": "ref_1234567890",
  "payment_id": "pay_1234567890",
  "amount": "0.00234567",
  "amount_usd": "100.00",
  "status": "pending",
  "reason": "Customer requested refund",
  "transaction_hash": null,
  "created_at": "2026-01-24T22:35:37Z",
  "completed_at": null
}`}</code></pre>
              </div>
            </div>

            <div className={styles.endpoint}>
              <div className={styles.endpointHeader}>
                <span className={styles.method}>GET</span>
                <span className={styles.url}>/v1/refunds/:refund_id</span>
                <span className={styles.description}>Get refund details</span>
              </div>
            </div>

            <div className={styles.endpoint}>
              <div className={styles.endpointHeader}>
                <span className={styles.method}>POST</span>
                <span className={styles.url}>/v1/refunds/:refund_id/complete</span>
                <span className={styles.description}>Complete pending refund</span>
              </div>
            </div>
          </div>
        )}

        {activeTab === 'analytics' && (
          <div className={styles.section}>
            <h1><i className="fas fa-chart-bar"></i> Analytics API</h1>
            <p className={styles.lead}>
              Access detailed analytics and reporting data for your payments and business metrics.
            </p>

            <div className={styles.endpointCard}>
              <div 
                className={styles.endpointHeader}
                onClick={() => toggleEndpoint('get-analytics')}
              >
                <div className={styles.endpointMethod}>
                  <span className={styles.methodGet}>GET</span>
                  <code>/v1/analytics</code>
                </div>
                <div className={styles.endpointTitle}>Get Analytics Data</div>
                <i className={`fas fa-chevron-${expandedEndpoint === 'get-analytics' ? 'up' : 'down'}`}></i>
              </div>
              
              {expandedEndpoint === 'get-analytics' && (
                <div className={styles.endpointContent}>
                  <h3><i className="fas fa-filter"></i> Query Parameters</h3>
                  <div className={styles.paramTable}>
                    <div className={styles.paramRow}>
                      <code>start_date</code>
                      <span className={styles.optional}>optional</span>
                      <span>Start date for analytics (ISO 8601 format)</span>
                    </div>
                    <div className={styles.paramRow}>
                      <code>end_date</code>
                      <span className={styles.optional}>optional</span>
                      <span>End date for analytics (ISO 8601 format)</span>
                    </div>
                    <div className={styles.paramRow}>
                      <code>granularity</code>
                      <span className={styles.optional}>optional</span>
                      <span>Data granularity: day, week, month</span>
                    </div>
                    <div className={styles.paramRow}>
                      <code>crypto_type</code>
                      <span className={styles.optional}>optional</span>
                      <span>Filter by cryptocurrency type</span>
                    </div>
                  </div>

                  <h3><i className="fas fa-arrow-left"></i> Response</h3>
                  <div className={styles.codeExample}>
                    <pre><code>{`{
  "period": {
    "start_date": "2026-01-01T00:00:00Z",
    "end_date": "2026-01-31T23:59:59Z",
    "granularity": "day"
  },
  "summary": {
    "total_payments": 1250,
    "total_volume_usd": "125000.00",
    "successful_payments": 1200,
    "failed_payments": 50,
    "success_rate": 96.0,
    "average_payment_usd": "100.00"
  },
  "data": [
    {
      "date": "2026-01-01",
      "payments": 45,
      "volume_usd": "4500.00",
      "success_rate": 95.6
    }
  ]
}`}</code></pre>
                  </div>
                </div>
              )}
            </div>

            <div className={styles.endpointCard}>
              <div 
                className={styles.endpointHeader}
                onClick={() => toggleEndpoint('export-analytics')}
              >
                <div className={styles.endpointMethod}>
                  <span className={styles.methodGet}>GET</span>
                  <code>/v1/analytics/export</code>
                </div>
                <div className={styles.endpointTitle}>Export Analytics Data</div>
                <i className={`fas fa-chevron-${expandedEndpoint === 'export-analytics' ? 'up' : 'down'}`}></i>
              </div>
              
              {expandedEndpoint === 'export-analytics' && (
                <div className={styles.endpointContent}>
                  <h3><i className="fas fa-filter"></i> Query Parameters</h3>
                  <div className={styles.paramTable}>
                    <div className={styles.paramRow}>
                      <code>format</code>
                      <span className={styles.optional}>optional</span>
                      <span>Export format: csv, json, xlsx (default: csv)</span>
                    </div>
                    <div className={styles.paramRow}>
                      <code>start_date</code>
                      <span className={styles.required}>required</span>
                      <span>Start date for export</span>
                    </div>
                    <div className={styles.paramRow}>
                      <code>end_date</code>
                      <span className={styles.required}>required</span>
                      <span>End date for export</span>
                    </div>
                  </div>

                  <h3><i className="fas fa-arrow-left"></i> Response</h3>
                  <div className={styles.codeExample}>
                    <pre><code>{`{
  "export_id": "exp_1234567890",
  "status": "processing",
  "format": "csv",
  "download_url": null,
  "expires_at": "2026-01-25T01:00:00Z",
  "created_at": "2026-01-25T00:30:00Z"
}`}</code></pre>
                  </div>
                </div>
              )}
            </div>
          </div>
        )}

        {activeTab === 'merchants' && (
          <div className={styles.section}>
            <h1>Merchants API</h1>
            <p className={styles.lead}>
              Manage merchant account settings, wallets, and configuration.
            </p>

            <div className={styles.endpoint}>
              <div className={styles.endpointHeader}>
                <span className={styles.method}>POST</span>
                <span className={styles.url}>/v1/merchants/register</span>
                <span className={styles.description}>Register new merchant</span>
              </div>

              <h3>Parameters</h3>
              <div className={styles.paramTable}>
                <div className={styles.param}>
                  <code>email</code>
                  <span className={styles.required}>required</span>
                  <p>Merchant email address</p>
                </div>
                <div className={styles.param}>
                  <code>business_name</code>
                  <span className={styles.required}>required</span>
                  <p>Business or company name</p>
                </div>
              </div>
            </div>

            <div className={styles.endpoint}>
              <div className={styles.endpointHeader}>
                <span className={styles.method}>GET</span>
                <span className={styles.url}>/v1/merchants/profile</span>
                <span className={styles.description}>Get merchant profile</span>
              </div>
            </div>

            <div className={styles.endpointCard}>
              <div 
                className={styles.endpointHeader}
                onClick={() => toggleEndpoint('set-wallets')}
              >
                <div className={styles.endpointMethod}>
                  <span className={styles.methodPut}>PUT</span>
                  <code>/v1/merchants/wallets</code>
                </div>
                <div className={styles.endpointTitle}>Set Wallet Addresses</div>
                <i className={`fas fa-chevron-${expandedEndpoint === 'set-wallets' ? 'up' : 'down'}`}></i>
              </div>
              
              {expandedEndpoint === 'set-wallets' && (
                <div className={styles.endpointContent}>
                  <h3><i className="fas fa-cog"></i> Request Parameters</h3>
                  <div className={styles.paramTable}>
                    <div className={styles.paramRow}>
                      <code>wallets</code>
                      <span className={styles.required}>required</span>
                      <span>Object containing wallet addresses for each crypto type</span>
                    </div>
                  </div>

                  <h3><i className="fas fa-arrow-left"></i> Response</h3>
                  <div className={styles.codeExample}>
                    <pre><code>{`{
  "message": "Wallet addresses updated successfully",
  "wallets": {
    "USDT_ETH": "0x742d35Cc6634C0532925a3b8D4C0532925a3b8D4",
    "SOL": "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM"
  }
}`}</code></pre>
                  </div>
                </div>
              )}
            </div>

            <div className={styles.endpointCard}>
              <div 
                className={styles.endpointHeader}
                onClick={() => toggleEndpoint('set-webhook')}
              >
                <div className={styles.endpointMethod}>
                  <span className={styles.methodPut}>PUT</span>
                  <code>/v1/merchants/webhook</code>
                </div>
                <div className={styles.endpointTitle}>Configure Webhook URL</div>
                <i className={`fas fa-chevron-${expandedEndpoint === 'set-webhook' ? 'up' : 'down'}`}></i>
              </div>
              
              {expandedEndpoint === 'set-webhook' && (
                <div className={styles.endpointContent}>
                  <h3><i className="fas fa-cog"></i> Request Parameters</h3>
                  <div className={styles.paramTable}>
                    <div className={styles.paramRow}>
                      <code>url</code>
                      <span className={styles.required}>required</span>
                      <span>Webhook URL to receive payment notifications</span>
                    </div>
                  </div>

                  <h3><i className="fas fa-arrow-left"></i> Response</h3>
                  <div className={styles.codeExample}>
                    <pre><code>{`{
  "message": "Webhook URL updated successfully",
  "webhook_url": "https://example.com/webhooks/fiddupay"
}`}</code></pre>
                  </div>
                </div>
              )}
            </div>
          </div>
        )}

        {activeTab === 'errors' && (
          <div className={styles.section}>
            <h1>Error Codes</h1>
            <p className={styles.lead}>
              FidduPay uses conventional HTTP response codes to indicate the success or failure of an API request.
            </p>

            <h2 className={styles.bigTitle}>HTTP Status Codes</h2>
            <div className={styles.errorList}>
              <div className={styles.errorItem}>
                <code className={styles.successCode}>200</code>
                <div>
                  <h3>OK</h3>
                  <p>Everything worked as expected.</p>
                </div>
              </div>
              <div className={styles.errorItem}>
                <code className={styles.successCode}>201</code>
                <div>
                  <h3>Created</h3>
                  <p>Resource was successfully created.</p>
                </div>
              </div>
              <div className={styles.errorItem}>
                <code className={styles.errorCode}>400</code>
                <div>
                  <h3>Bad Request</h3>
                  <p>The request was unacceptable, often due to missing a required parameter.</p>
                </div>
              </div>
              <div className={styles.errorItem}>
                <code className={styles.errorCode}>401</code>
                <div>
                  <h3>Unauthorized</h3>
                  <p>No valid API key provided.</p>
                </div>
              </div>
              <div className={styles.errorItem}>
                <code className={styles.errorCode}>403</code>
                <div>
                  <h3>Forbidden</h3>
                  <p>The API key doesn't have permissions to perform the request.</p>
                </div>
              </div>
              <div className={styles.errorItem}>
                <code className={styles.errorCode}>404</code>
                <div>
                  <h3>Not Found</h3>
                  <p>The requested resource doesn't exist.</p>
                </div>
              </div>
              <div className={styles.errorItem}>
                <code className={styles.errorCode}>429</code>
                <div>
                  <h3>Too Many Requests</h3>
                  <p>Too many requests hit the API too quickly. Rate limit exceeded.</p>
                </div>
              </div>
              <div className={styles.errorItem}>
                <code className={styles.errorCode}>500</code>
                <div>
                  <h3>Server Error</h3>
                  <p>Something went wrong on FidduPay's end.</p>
                </div>
              </div>
            </div>

            <h2 className={styles.bigTitle}>Error Response Format</h2>
            <div className={styles.codeExample}>
              <pre><code>{`{
  "error": {
    "type": "invalid_request_error",
    "code": "parameter_missing",
    "message": "Missing required parameter: amount_usd",
    "param": "amount_usd"
  }
}`}</code></pre>
            </div>

            <h2 className={styles.bigTitle}>Common Error Types</h2>
            <div className={styles.errorTypeList}>
              <div className={styles.errorType}>
                <code>api_error</code>
                <p>API errors cover any other type of problem (e.g., a temporary problem with FidduPay's servers)</p>
              </div>
              <div className={styles.errorType}>
                <code>authentication_error</code>
                <p>Failure to properly authenticate yourself in the request</p>
              </div>
              <div className={styles.errorType}>
                <code>invalid_request_error</code>
                <p>Invalid request errors arise when your request has invalid parameters</p>
              </div>
              <div className={styles.errorType}>
                <code>rate_limit_error</code>
                <p>Too many requests hit the API too quickly</p>
              </div>
              <div className={styles.errorType}>
                <code>daily_volume_exceeded</code>
                <p>Daily volume limit exceeded for non-KYC merchant</p>
              </div>
            </div>

            <h2>Daily Volume Limits</h2>
            <div className={styles.section}>
              <p>FidduPay enforces daily volume limits based on merchant KYC status:</p>
              
              <div className={styles.limitInfo}>
                <h3>Non-KYC Merchants</h3>
                <ul>
                  <li><strong>Daily Limit:</strong> $1,000 USD total volume</li>
                  <li><strong>Combined Tracking:</strong> All deposits + withdrawals count toward limit</li>
                  <li><strong>Reset:</strong> Daily at midnight UTC</li>
                  <li><strong>No Per-Transaction Limits:</strong> Individual transactions can be any amount up to remaining daily volume</li>
                </ul>
              </div>

              <div className={styles.limitInfo}>
                <h3>KYC Verified Merchants</h3>
                <ul>
                  <li><strong>No Limits:</strong> Unlimited daily volume</li>
                  <li><strong>Full Access:</strong> No restrictions on transaction amounts or frequency</li>
                </ul>
              </div>

              <div className={styles.codeExample}>
                <h4>Check Remaining Volume</h4>
                <pre><code>{`GET /api/v1/merchants/profile
Authorization: Bearer sk_your_api_key

Response:
{
  "id": 123,
  "business_name": "My Business",
  "email": "merchant@example.com",
  "kyc_verified": false,
  "daily_volume_remaining": "750.00"
}`}</code></pre>
              </div>
            </div>
          </div>
        )}

        {activeTab === 'sdks' && (
          <div className={styles.section}>
            <h1>Official SDKs</h1>
            <p className={styles.lead}>
              Use our official libraries to integrate FidduPay into your application.
            </p>

            <h2 className={styles.bigTitle}>Node.js SDK</h2>
            <div className={styles.sdkHighlight}>
              <div className={styles.sdkCard}>
                <i className="fab fa-node-js"></i>
                <h3>FidduPay Node.js</h3>
                <p>Official Node.js library for FidduPay API integration</p>
                <div className={styles.installCode}>
                  <code>npm install fiddupay-node</code>
                </div>
                <div className={styles.codeExample}>
                  <pre><code>{`const FidduPay = require('fiddupay-node');
const fiddupay = new FidduPay('sk_test_...');

// Create a payment
const payment = await fiddupay.payments.create({
  amount_usd: '100.00',
  crypto_type: 'USDT_ETH',
  description: 'Order #12345'
});

console.log(payment.payment_id);`}</code></pre>
                </div>
              </div>
            </div>

            <h2 className={styles.bigTitle}>Coming Soon</h2>
            <div className={styles.comingSoonGrid}>
              <div className={styles.comingSoonCard}>
                <i className="fab fa-python"></i>
                <h3>Python SDK</h3>
                <p>Coming Q2 2026</p>
                <div className={styles.comingSoonBadge}>In Development</div>
              </div>
              <div className={styles.comingSoonCard}>
                <i className="fab fa-php"></i>
                <h3>PHP SDK</h3>
                <p>Coming Q2 2026</p>
                <div className={styles.comingSoonBadge}>Planned</div>
              </div>
              <div className={styles.comingSoonCard}>
                <i className="fab fa-rust"></i>
                <h3>Rust SDK</h3>
                <p>Coming Q3 2026</p>
                <div className={styles.comingSoonBadge}>Planned</div>
              </div>
              <div className={styles.comingSoonCard}>
                <i className="fab fa-golang"></i>
                <h3>Go SDK</h3>
                <p>Coming Q3 2026</p>
                <div className={styles.comingSoonBadge}>Planned</div>
              </div>
            </div>

            <div className={styles.infoBox}>
              <h3>Request an SDK</h3>
              <p>Need an SDK for a specific language? Let us know at <strong>sdk@fiddupay.com</strong></p>
            </div>
          </div>
        )}
      </div>
    </div>
  )
}

export default DocsPage
