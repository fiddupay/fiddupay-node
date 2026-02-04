export interface Parameter {
    name: string;
    type: string;
    required: boolean;
    description: string;
}

export interface Endpoint {
    id: string;
    method: 'GET' | 'POST' | 'PUT' | 'DELETE';
    path: string;
    title: string;
    description: string;
    parameters?: Parameter[];
    body?: Parameter[];
    request?: {
        curl: string;
        node: string;
    };
    response?: string;
}

export interface DocSection {
    id: string;
    title: string;
    description?: string;
    endpoints: Endpoint[];
}

export const API_DATA: DocSection[] = [
    {
        id: 'getting-started',
        title: 'Getting Started',
        description: 'Welcome to the FidduPay API. Our API is built on REST principles and uses JSON for all communication. This guide will help you integrate cryptocurrency payments into your application.',
        endpoints: [
            {
                id: 'authentication',
                method: 'GET',
                path: 'Header: Authorization',
                title: 'Authentication',
                description: 'FidduPay uses API keys to authenticate requests. You can view and manage your API keys in the Merchant Dashboard. All requests must be made over HTTPS.',
                request: {
                    curl: 'curl https://api.fiddupay.com/api/v1/merchants/profile \\\n  -H "Authorization: Bearer sk_live_your_api_key"',
                    node: 'const profile = await fiddupay.merchants.getProfile();'
                },
                response: JSON.stringify({
                    id: 123,
                    business_name: "My Crypto Store",
                    email: "merchant@example.com",
                    created_at: "2026-01-26T06:00:00Z"
                }, null, 2)
            },
            {
                id: 'api-keys',
                method: 'POST',
                path: '/api/v1/merchants/api-keys/rotate',
                title: 'Rotate API Key',
                description: 'Generate a new API key and immediately invalidate the current one. Use this if your key has been compromised.',
                request: {
                    curl: 'curl -X POST https://api.fiddupay.com/api/v1/merchants/api-keys/rotate \\\n  -H "Authorization: Bearer sk_live_..."',
                    node: 'const newKey = await fiddupay.merchants.rotateApiKey();'
                },
                response: JSON.stringify({
                    api_key: "sk_live_new_rotated_key_..."
                }, null, 2)
            },
            {
                id: 'environment',
                method: 'POST',
                path: '/api/v1/merchants/environment/switch',
                title: 'Switch Environment',
                description: 'Toggle your account between Live and Sandbox environments.',
                body: [
                    { name: 'to_live', type: 'boolean', required: true, description: 'True for Live, false for Sandbox' }
                ],
                request: {
                    curl: 'curl -X POST https://api.fiddupay.com/api/v1/merchants/environment/switch \\\n  -H "Authorization: Bearer sk_live_..." \\\n  -d \'{"to_live": false}\'',
                    node: 'await fiddupay.merchants.switchEnvironment(false);'
                },
                response: JSON.stringify({
                    api_key: "sk_test_sandbox_key_...",
                    environment: "sandbox"
                }, null, 2)
            }
        ]
    },
    {
        id: 'payments',
        title: 'Payments',
        description: 'Manage lifecycle of cryptocurrency payments, from creation to settlement.',
        endpoints: [
            {
                id: 'create-payment',
                method: 'POST',
                path: '/api/v1/merchants/payments',
                title: 'Create Payment',
                description: 'Initialize a new multi-chain payment request.',
                body: [
                    { name: 'amount_usd', type: 'string', required: false, description: 'USD amount (e.g. "99.99")' },
                    { name: 'amount', type: 'string', required: false, description: 'Crypto amount (e.g. "0.5")' },
                    { name: 'crypto_type', type: 'string', required: true, description: 'SOL, USDT_ETH, USDT_SPL, etc.' },
                    { name: 'webhook_url', type: 'string', required: false, description: 'Override default webhook' },
                    { name: 'expiration_minutes', type: 'integer', required: false, description: 'Defaults to 20 mins' }
                ],
                request: {
                    curl: 'curl -X POST https://api.fiddupay.com/api/v1/merchants/payments \\\n  -H "Authorization: Bearer sk_live_..." \\\n  -d \'{\n    "amount_usd": "100.00",\n    "crypto_type": "USDT_ETH"\n  }\'',
                    node: 'const payment = await fiddupay.payments.create({\n  amount_usd: "100.00",\n  crypto_type: "USDT_ETH"\n});'
                },
                response: JSON.stringify({
                    payment_id: "pay_123",
                    status: "PENDING",
                    deposit_address: "0x...",
                    payment_link: "https://pay.fiddupay.com/pay_123"
                }, null, 2)
            },
            {
                id: 'retrieve-payment',
                method: 'GET',
                path: '/api/v1/merchants/payments/:payment_id',
                title: 'Retrieve Payment',
                description: 'Get the current status and details of a specific payment.',
                request: {
                    curl: 'curl https://api.fiddupay.com/api/v1/merchants/payments/pay_123 \\\n  -H "Authorization: Bearer sk_live_..."',
                    node: 'const payment = await fiddupay.payments.get("pay_123");'
                },
                response: JSON.stringify({
                    payment_id: "pay_123",
                    status: "CONFIRMED",
                    amount: "100.0",
                    transaction_hash: "0x..."
                }, null, 2)
            },
            {
                id: 'verify-payment',
                method: 'POST',
                path: '/api/v1/merchants/payments/:payment_id/verify',
                title: 'Verify Payment',
                description: 'Manually trigger verification of a payment using a transaction hash.',
                body: [
                    { name: 'transaction_hash', type: 'string', required: true, description: 'On-chain TRX hash' }
                ],
                request: {
                    curl: 'curl -X POST https://api.fiddupay.com/api/v1/merchants/payments/pay_123/verify \\\n  -H "Authorization: Bearer sk_live_..." \\\n  -d \'{"transaction_hash": "0x..."}\'',
                    node: 'await fiddupay.payments.verify("pay_123", "0x...");'
                },
                response: JSON.stringify({
                    confirmed: true,
                    status: "CONFIRMED"
                }, null, 2)
            }
        ]
    },
    {
        id: 'wallets',
        title: 'Wallets & Infrastructure',
        description: 'Configure how you receive funds. Support for both platform-managed and self-custodied addresses.',
        endpoints: [
            {
                id: 'list-wallets',
                method: 'GET',
                path: '/api/v1/merchants/wallets',
                title: 'List Configured Wallets',
                description: 'Retrieve all configured settlement addresses across all supported chains.',
                request: {
                    curl: 'curl https://api.fiddupay.com/api/v1/merchants/wallets \\\n  -H "Authorization: Bearer sk_live_..."',
                    node: 'const wallets = await fiddupay.wallets.list();'
                },
                response: JSON.stringify([
                    { crypto_type: "USDT_ETH", address: "0x...", wallet_type: "managed" }
                ], null, 2)
            },
            {
                id: 'export-key',
                method: 'POST',
                path: '/api/v1/merchants/wallets/export-key',
                title: 'Export Private Key',
                description: 'Export the private key for a platform-managed wallet. Warning: Use extreme caution.',
                body: [
                    { name: 'crypto_type', type: 'string', required: true, description: 'SOL, USDT_ETH, etc.' }
                ],
                request: {
                    curl: 'curl -X POST https://api.fiddupay.com/api/v1/merchants/wallets/export-key \\\n  -H "Authorization: Bearer sk_live_..." \\\n  -d \'{"crypto_type": "SOL"}\'',
                    node: 'const key = await fiddupay.wallets.exportKey("SOL");'
                },
                response: JSON.stringify({
                    private_key: "...",
                    warning: "Key exported. Ensure you have backed it up."
                }, null, 2)
            },
            {
                id: 'withdrawal-capability',
                method: 'GET',
                path: '/api/v1/merchants/wallets/withdrawal-capability/:crypto_type',
                title: 'Check Withdrawal Capability',
                description: 'Verify if a specific protocol is currently available for automated withdrawals.',
                request: {
                    curl: 'curl https://api.fiddupay.com/api/v1/merchants/wallets/withdrawal-capability/SOL \\\n  -H "Authorization: Bearer sk_live_..."',
                    node: 'const capable = await fiddupay.wallets.checkCapability("SOL");'
                },
                response: JSON.stringify({
                    capable: true,
                    network: "SOLANA",
                    min_withdrawal: "0.1"
                }, null, 2)
            }
        ]
    },
    {
        id: 'security',
        title: 'Security & Monitoring',
        description: 'Real-time monitoring of account security and system health.',
        endpoints: [
            {
                id: 'get-security-settings',
                method: 'GET',
                path: '/api/v1/merchants/security/settings',
                title: 'Get Security Settings',
                description: 'Retrieve your account\'s current security parameters and notification preferences.',
                request: {
                    curl: 'curl https://api.fiddupay.com/api/v1/merchants/security/settings \\\n  -H "Authorization: Bearer sk_live_..."',
                    node: 'const settings = await fiddupay.security.getSettings();'
                },
                response: JSON.stringify({
                    two_factor_enabled: true,
                    ip_whitelist_enforced: false,
                    alert_email: "security@example.com"
                }, null, 2)
            },
            {
                id: 'get-alerts',
                method: 'GET',
                path: '/api/v1/merchants/security/alerts',
                title: 'List Security Alerts',
                description: 'Retrieve active security alerts or system warnings (e.g. low gas, suspicious login).',
                request: {
                    curl: 'curl https://api.fiddupay.com/api/v1/merchants/security/alerts \\\n  -H "Authorization: Bearer sk_live_..."',
                    node: 'const alerts = await fiddupay.security.getAlerts();'
                },
                response: JSON.stringify([
                    { id: "alt_123", type: "LOW_GAS", severity: "high", message: "ETH balance low on mainnet" }
                ], null, 2)
            }
        ]
    },
    {
        id: 'sandbox-tools',
        title: 'Sandbox Simulation',
        description: 'Tools for testing your integration without using real digital assets.',
        endpoints: [
            {
                id: 'enable-sandbox',
                method: 'POST',
                path: '/api/v1/merchants/sandbox/enable',
                title: 'Enable Sandbox Environment',
                description: 'Generate sandbox credentials for comprehensive testing.',
                request: {
                    curl: 'curl -X POST https://api.fiddupay.com/api/v1/merchants/sandbox/enable \\\n  -H "Authorization: Bearer sk_live_..."',
                    node: 'await fiddupay.sandbox.enable();'
                },
                response: JSON.stringify({
                    sandbox_api_key: "sk_test_sandbox_...",
                    message: "Sandbox environment enabled"
                }, null, 2)
            },
            {
                id: 'simulate-payment',
                method: 'POST',
                path: '/api/v1/merchants/sandbox/payments/:payment_id/simulate',
                title: 'Simulate Payment Success',
                description: 'Force a sandbox payment to "CONFIRMED" status to test your webhooks.',
                body: [
                    { name: 'success', type: 'boolean', required: true, description: 'True to simulate success, false for failure' }
                ],
                request: {
                    curl: 'curl -X POST https://api.fiddupay.com/api/v1/merchants/sandbox/payments/pay_123/simulate \\\n  -H "Authorization: Bearer sk_test_..." \\\n  -d \'{"success": true}\'',
                    node: 'await fiddupay.sandbox.simulate("pay_123", true);'
                },
                response: JSON.stringify({
                    success: true,
                    message: "Payment simulated successfully"
                }, null, 2)
            }
        ]
    },
    {
        id: 'analytics-balance',
        title: 'Analytics & Financials',
        description: 'Monitor your transaction volume, fees, and real-time account balances.',
        endpoints: [
            {
                id: 'get-analytics',
                method: 'GET',
                path: '/api/v1/merchants/analytics',
                title: 'Get Performance Analytics',
                description: 'Retrieve high-level metrics including total volume, transaction count, and fee summaries.',
                request: {
                    curl: 'curl https://api.fiddupay.com/api/v1/merchants/analytics \\\n  -H "Authorization: Bearer sk_live_..."',
                    node: 'const stats = await fiddupay.analytics.get();'
                },
                response: JSON.stringify({
                    total_volume: "125000.50",
                    payment_count: 1450,
                    total_fees: "312.45"
                }, null, 2)
            },
            {
                id: 'get-balance',
                method: 'GET',
                path: '/api/v1/merchants/balance',
                title: 'Get Account Balance',
                description: 'Retrieve current settled balances across all supported cryptocurrencies.',
                request: {
                    curl: 'curl https://api.fiddupay.com/api/v1/merchants/balance \\\n  -H "Authorization: Bearer sk_live_..."',
                    node: 'const balance = await fiddupay.balance.get();'
                },
                response: JSON.stringify([
                    { crypto_type: "SOL", amount: "45.5", amount_usd: "4550.00" }
                ], null, 2)
            },
            {
                id: 'fee-setting',
                method: 'GET',
                path: '/api/v1/merchants/fee-setting',
                title: 'Get Fee Configuration',
                description: 'View your current fee tier and per-transaction rates.',
                request: {
                    curl: 'curl https://api.fiddupay.com/api/v1/merchants/fee-setting \\\n  -H "Authorization: Bearer sk_live_..."',
                    node: 'const fees = await fiddupay.merchants.getFees();'
                },
                response: JSON.stringify({
                    tier: "Business Prime",
                    base_fee_pct: "2.4",
                    min_fee_usd: "0.50"
                }, null, 2)
            }
        ]
    },
    {
        id: 'withdrawals',
        title: 'Withdrawals',
        description: 'Transfer accumulated funds from your gateway account to your private cold storage.',
        endpoints: [
            {
                id: 'create-withdrawal',
                method: 'POST',
                path: '/api/v1/merchants/withdrawals',
                title: 'Initiate Withdrawal',
                description: 'Request a withdrawal of funds to your configured external address.',
                body: [
                    { name: 'amount', type: 'string', required: true, description: 'Amount to withdraw' },
                    { name: 'crypto_type', type: 'string', required: true, description: 'Protocol (SOL, ETH, etc.)' }
                ],
                request: {
                    curl: 'curl -X POST https://api.fiddupay.com/api/v1/merchants/withdrawals \\\n  -H "Authorization: Bearer sk_live_..." \\\n  -d \'{"amount": "10.0", "crypto_type": "SOL"}\'',
                    node: 'const tx = await fiddupay.withdrawals.create({ amount: "10.0", crypto_type: "SOL" });'
                },
                response: JSON.stringify({
                    id: "wth_123",
                    status: "pending",
                    destination: "0x..."
                }, null, 2)
            },
            {
                id: 'list-withdrawals',
                method: 'GET',
                path: '/api/v1/merchants/withdrawals',
                title: 'List Withdrawals',
                description: 'Retrieve historical withdrawal requests and their statuses.',
                request: {
                    curl: 'curl https://api.fiddupay.com/api/v1/merchants/withdrawals \\\n  -H "Authorization: Bearer sk_live_..."',
                    node: 'const history = await fiddupay.withdrawals.list();'
                },
                response: JSON.stringify([
                    { id: "wth_122", status: "completed", tx_hash: "0x..." }
                ], null, 2)
            }
        ]
    },
    {
        id: 'compliance-audits',
        title: 'Audit & Compliance',
        description: 'Maintain regulatory compliance and operational transparency with detailed event logging.',
        endpoints: [
            {
                id: 'get-audit-logs',
                method: 'GET',
                path: '/api/v1/merchants/audit-logs',
                title: 'Retrieve Audit Logs',
                description: 'Get a detailed chronological log of all administrative actions and system events for your account.',
                request: {
                    curl: 'curl https://api.fiddupay.com/api/v1/merchants/audit-logs \\\n  -H "Authorization: Bearer sk_live_..."',
                    node: 'const logs = await fiddupay.audits.list();'
                },
                response: JSON.stringify([
                    { timestamp: "2026-02-04T12:00:00Z", action: "API_KEY_ROTATED", actor: "merchant_user_1" }
                ], null, 2)
            },
            {
                id: 'get-balance-history',
                method: 'GET',
                path: '/api/v1/merchants/balance/history',
                title: 'Get Balance History',
                description: 'Retrieve historical balance records for reconciliation and accounting purposes.',
                request: {
                    curl: 'curl https://api.fiddupay.com/api/v1/merchants/balance/history \\\n  -H "Authorization: Bearer sk_live_..."',
                    node: 'const history = await fiddupay.balance.getHistory();'
                },
                response: JSON.stringify([
                    { date: "2026-02-03", crypto_type: "USDT_ETH", balance: "1500.00" }
                ], null, 2)
            }
        ]
    },
    {
        id: 'advanced-security',
        title: 'Granular Alerts',
        description: 'Fine-tuned control over system alerts and security responses.',
        endpoints: [
            {
                id: 'acknowledge-alert',
                method: 'POST',
                path: '/api/v1/merchants/security/alerts/:alert_id/acknowledge',
                title: 'Acknowledge Security Alert',
                description: 'Mark a high-severity security alert as reviewed to clear it from your dashboard.',
                request: {
                    curl: 'curl -X POST https://api.fiddupay.com/api/v1/merchants/security/alerts/alt_123/acknowledge \\\n  -H "Authorization: Bearer sk_live_..."',
                    node: 'await fiddupay.security.acknowledgeAlert("alt_123");'
                },
                response: JSON.stringify({ message: "Alert acknowledged" }, null, 2)
            },
            {
                id: 'set-global-webhook',
                method: 'PUT',
                path: '/api/v1/merchants/webhook',
                title: 'Configure Global Webhook',
                description: 'Set a default destination for all system events and payment notifications.',
                body: [
                    { name: 'webhook_url', type: 'string', required: true, description: 'The absolute URL to receive POST events' }
                ],
                request: {
                    curl: 'curl -X PUT https://api.fiddupay.com/api/v1/merchants/webhook \\\n  -H "Authorization: Bearer sk_live_..." \\\n  -d \'{"webhook_url": "https://callback.com/fiddupay"}\'',
                    node: 'await fiddupay.merchants.setWebhook("https://callback.com/fiddupay");'
                },
                response: JSON.stringify({ message: "Webhook URL updated successfully" }, null, 2)
            }
        ]
    }
];
