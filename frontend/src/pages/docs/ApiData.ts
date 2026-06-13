export interface Parameter {
    name: string;
    type: string;
    required: boolean;
    description: string;
}

export interface SubSection {
    title: string;
    items: {
        key: string;
        description: string;
    }[];
}

export interface Endpoint {
    id: string;
    method: 'GET' | 'POST' | 'PUT' | 'DELETE' | 'PATCH';
    path: string;
    title: string;
    description: string;
    parameters?: Parameter[];
    body?: Parameter[];
    subSections?: SubSection[];
    request?: {
        curl: string;
        node: string;
    };
    response?: string;
    deprecated?: boolean;
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
                    node: 'const profile = await fiddupay.merchants.retrieve();'
                },
                response: JSON.stringify({
                    id: 123,
                    business_name: "My Crypto Store",
                    email: "merchant@example.com",
                    role: "merchant",
                    created_at: "2026-01-26T06:00:00Z",
                    sandbox_mode: true,
                    settlement_mode: "managed",
                    low_balance_alerts_enabled: true,
                    low_balance_threshold_usd: "50.00"
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
                    api_key: "sk_sandbox_sandbox_key_...",
                    environment: "sandbox"
                }, null, 2)
            },
            {
                id: 'update-settings',
                method: 'PATCH',
                path: '/api/v1/merchants/settings',
                title: 'Update Settings (Unified)',
                description: 'Consolidated endpoint to update all merchant settings atomically, including webhook URL, settlement mode, fee settings, and IP whitelist.',
                body: [
                    { name: 'webhook_url', type: 'string', required: false, description: 'New webhook destination' },
                    { name: 'settlement_mode', type: 'string', required: false, description: 'forwarding or managed' },
                    { name: 'customer_pays_fee', type: 'boolean', required: false, description: 'Toggle who pays network fees' },
                    { name: 'fee_percentage', type: 'number', required: false, description: 'Custom fee percentage override' },
                    { name: 'ip_whitelist', type: 'string[]', required: false, description: 'Array of allowed IP addresses' },
                    { name: 'sandbox_mode', type: 'boolean', required: false, description: 'Toggle sandbox environment' },
                    { name: 'low_balance_threshold_usd', type: 'string', required: false, description: 'USD threshold for balance alerts (e.g. "50.00")' },
                    { name: 'low_balance_alerts_enabled', type: 'boolean', required: false, description: 'Toggle real-time low balance notifications' }
                ],
                request: {
                    curl: 'curl -X PATCH https://api.fiddupay.com/api/v1/merchants/settings \\\n  -H "Authorization: Bearer sk_live_..." \\\n  -d \'{\n    "webhook_url": "https://example.com/webhook",\n    "low_balance_alerts_enabled": true,\n    "low_balance_threshold_usd": "100.00"\n  }\'',
                    node: 'await fiddupay.merchants.updateSettings({\n  webhook_url: "https://example.com/webhook",\n  low_balance_alerts_enabled: true,\n  low_balance_threshold_usd: "100.00"\n});'
                },
                response: JSON.stringify({
                    status: "success",
                    message: "Settings updated successfully"
                }, null, 2)
            },
            {
                id: 'get-status',
                method: 'GET',
                path: '/api/v1/merchants/status',
                title: 'Get Readiness Status',
                description: 'Assess merchant readiness, network coverage, and active security alerts to ensure the account is fully operational.',
                request: {
                    curl: 'curl https://api.fiddupay.com/api/v1/merchants/status \\\n  -H "Authorization: Bearer sk_live_..."',
                    node: 'const readiness = await fiddupay.merchants.getReadiness();'
                },
                response: JSON.stringify({
                    is_ready: true,
                    environment: "production",
                    settlement_mode: "forwarding",
                    network_coverage: ["SOLANA", "ETHEREUM"],
                    security: {
                        active_alerts: 0,
                        critical_alerts: 0
                    },
                    verification_status: "verified",
                    issues: []
                }, null, 2)
            },
            {
                id: 'generate-api-key',
                method: 'POST',
                path: '/api/v1/merchants/api-keys/generate',
                title: 'Generate API Key',
                description: 'Generate a new API key for the specified environment (live or sandbox).',
                body: [
                    { name: 'is_live', type: 'boolean', required: true, description: 'True for live, false for sandbox' }
                ],
                request: {
                    curl: 'curl -X POST https://api.fiddupay.com/api/v1/merchants/api-keys/generate \\\n  -H "Authorization: Bearer sk_live_..." \\\n  -d \'{"is_live": true}\'',
                    node: 'const newKey = await fiddupay.merchants.generateApiKey(true);'
                },
                response: JSON.stringify({
                    api_key: "sk_live_new_generated_key_..."
                }, null, 2)
            },
            {
                id: 'test-webhook',
                method: 'POST',
                path: '/api/v1/merchants/webhook/test',
                title: 'Test Webhook Configuration',
                description: 'Trigger a test webhook event to verify your endpoint is correctly receiving and acknowledging notifications.',
                request: {
                    curl: 'curl -X POST https://api.fiddupay.com/api/v1/merchants/webhook/test \\\n  -H "Authorization: Bearer sk_live_..."',
                    node: 'await fiddupay.merchants.sendTestWebhook();'
                },
                response: JSON.stringify({
                    status: "success",
                    message: "Test webhook sent to https://your-site.com/webhook",
                    delivery_id: "del_123"
                }, null, 2)
            },
            {
                id: 'logout',
                method: 'POST',
                path: '/api/v1/merchants/logout',
                title: 'Logout Merchant',
                description: 'Invalidate the current merchant session and tokens.',
                request: {
                    curl: 'curl -X POST https://api.fiddupay.com/api/v1/merchants/logout \\\n  -H "Authorization: Bearer sk_live_..."',
                    node: 'await fiddupay.merchants.logout();'
                },
                response: JSON.stringify({
                    message: "Logged out successfully"
                }, null, 2)
            },
            {
                id: 'get-settings',
                method: 'GET',
                path: '/api/v1/merchants/settings',
                title: 'Get Settings',
                description: 'Retrieve current configurations including webhook endpoint, settlement mode, fee coverage, IP whitelist, and alerts.',
                request: {
                    curl: 'curl https://api.fiddupay.com/api/v1/merchants/settings \\\n  -H "Authorization: Bearer sk_live_..."',
                    node: 'const settings = await fiddupay.merchants.getSettings();'
                },
                response: JSON.stringify({
                    webhook_url: "https://example.com/webhook",
                    webhook_format: "standard",
                    webhook_signing_secret: "whsec_abcd**********",
                    settlement_mode: "managed",
                    customer_pays_fee: false,
                    sandbox_mode: false,
                    redirect_url: "https://example.com/success",
                    low_balance_alerts_enabled: true,
                    ip_whitelist: ["12.34.56.78", "98.76.54.32"]
                }, null, 2)
            },
            {
                id: 'claim-username',
                method: 'POST',
                path: '/api/v1/merchants/claim-username',
                title: 'Claim Username',
                description: 'Reserve a unique username (PayID) for the merchant profile.',
                body: [
                    { name: 'username', type: 'string', required: true, description: 'Desired unique username to register' }
                ],
                request: {
                    curl: 'curl -X POST https://api.fiddupay.com/api/v1/merchants/claim-username \\\n  -H "Authorization: Bearer sk_live_..." \\\n  -d \'{"username": "crypto_merchant"}\'',
                    node: 'await fiddupay.merchants.claimUsername("crypto_merchant");'
                },
                response: JSON.stringify({
                    status: "success",
                    message: "Username claimed"
                }, null, 2)
            },
            {
                id: 'update-kyc-draft',
                method: 'POST',
                path: '/api/v1/merchants/kyc-draft',
                title: 'Update KYC Draft',
                description: 'Save or update raw merchant identity and compliance information draft details.',
                body: [
                    { name: 'first_name', type: 'string', required: false, description: 'Legal first name' },
                    { name: 'last_name', type: 'string', required: false, description: 'Legal last name' },
                    { name: 'gender', type: 'string', required: false, description: 'Identity gender' },
                    { name: 'phone_number', type: 'string', required: false, description: 'Contact phone' },
                    { name: 'country', type: 'string', required: false, description: 'Merchant resident country' },
                    { name: 'social_handles', type: 'object', required: false, description: 'Twitter, Instagram links' },
                    { name: 'business_country', type: 'string', required: false, description: 'Business registration country' },
                    { name: 'business_license_number', type: 'string', required: false, description: 'Official license/reg number' },
                    { name: 'business_certificate_url', type: 'string', required: false, description: 'Certificate scan URL' },
                    { name: 'nin_bvn', type: 'string', required: false, description: 'National identity hash input' }
                ],
                request: {
                    curl: 'curl -X POST https://api.fiddupay.com/api/v1/merchants/kyc-draft \\\n  -H "Authorization: Bearer sk_live_..." \\\n  -d \'{"first_name": "Jane", "last_name": "Doe"}\'',
                    node: 'await fiddupay.merchants.updateKycDraft({\n  first_name: "Jane",\n  last_name: "Doe"\n});'
                },
                response: JSON.stringify({
                    status: "success",
                    message: "KYC draft updated"
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
                description: 'Initialize a new payment request. For fixed single-currency links, providing `.amount` is mandatory. For multi-currency checkout links, provide `.amount_usd` instead.',
                body: [
                    { name: 'amount', type: 'string', required: false, description: 'Required for all fixed single-currency links (e.g. SOL, USDT)' },
                    { name: 'amount_usd', type: 'string', required: false, description: 'Required for multi-currency checkout links only (where crypto_type is omitted)' },
                    { name: 'crypto_type', type: 'string', required: false, description: 'SOL, BTC, USDT_ETH, USDT_SPL, etc. (Omit for multi-currency link)' },
                    { name: 'webhook_url', type: 'string', required: false, description: 'Override default webhook' },
                    { name: 'expiration_minutes', type: 'integer', required: false, description: 'Defaults to 20 mins' }
                ],
                request: {
                    curl: '# Create Fixed Currency Payment (Supports any coin including USDT)\ncurl -X POST https://api.fiddupay.com/api/v1/merchants/payments \\\n  -H "Authorization: Bearer sk_live_..." \\\n  -d \'{"amount": "100.00", "crypto_type": "USDT_ETH"}\'\n\n# Create Multi-currency Checkout link\ncurl -X POST https://api.fiddupay.com/api/v1/merchants/payments \\\n  -H "Authorization: Bearer sk_live_..." \\\n  -d \'{"amount_usd": "100.00"}\'',
                    node: '// Create Fixed Currency Payment\nconst p1 = await fiddupay.payments.create({ amount: "100.00", crypto_type: "USDT_ETH" });\n\n// Create Multi-currency Checkout\nconst p2 = await fiddupay.payments.create({ amount_usd: "100.00" });'
                },
                response: JSON.stringify({
                    payment_id: "pay_123",
                    crypto_type: "USDT_ETH",
                    amount: "100.00",
                    amount_usd: "100.00",
                    to_address: "0x742d35Cc6634C0532925a3b8D4C9db96590c6C87",
                    status: "PENDING",
                    confirmations: 0,
                    required_confirmations: 12,
                    expires_at: "2026-02-04T10:20:00Z",
                    created_at: "2026-02-04T10:00:00Z",
                    confirmed_at: null,
                    description: "Stablecoin Order",
                    metadata: { order_id: "12345" },
                    network: "ETHEREUM",
                    deposit_address: "0x742d35Cc6634C0532925a3b8D4C9db96590c6C87",
                    payment_link: "https://pay.fiddupay.com/lnk_xyz789",
                    qr_code_data: "ethereum:0x742d35Cc6634C0532925a3b8D4C9db96590c6C87?amount=100.00",
                    fee_amount: "1.50",
                    fee_amount_usd: "1.50",
                    transaction_hash: null,
                    from_address: null,
                    partial_payments: null,
                    last_verification_at: null
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
                    node: 'const payment = await fiddupay.payments.retrieve("pay_123");'
                },
                response: JSON.stringify({
                    payment_id: "pay_123",
                    crypto_type: "USDT_ETH",
                    amount: "100.00",
                    amount_usd: "100.00",
                    to_address: "0x742d35Cc6634C0532925a3b8D4C9db96590c6C87",
                    status: "CONFIRMED",
                    confirmations: 12,
                    required_confirmations: 12,
                    expires_at: "2026-02-04T10:20:00Z",
                    created_at: "2026-02-04T10:00:00Z",
                    confirmed_at: "2026-02-04T10:05:00Z",
                    description: "Stablecoin Order",
                    metadata: { order_id: "12345" },
                    network: "ETHEREUM",
                    deposit_address: "0x742d35Cc6634C0532925a3b8D4C9db96590c6C87",
                    payment_link: "https://pay.fiddupay.com/lnk_xyz789",
                    qr_code_data: "ethereum:0x742d35Cc6634C0532925a3b8D4C9db96590c6C87?amount=100.00",
                    fee_amount: "1.50",
                    fee_amount_usd: "1.50",
                    transaction_hash: "0x123abc...",
                    from_address: "0xsender...",
                    partial_payments: null,
                    last_verification_at: "2026-02-04T10:05:00Z"
                }, null, 2)
            },
            {
                id: 'list-payments',
                method: 'GET',
                path: '/api/v1/merchants/payments',
                title: 'List Payments',
                description: 'Retrieve a list of payments with optional filters.',
                parameters: [
                    { name: 'limit', type: 'integer', required: false, description: 'Number of records to return' },
                    { name: 'offset', type: 'integer', required: false, description: 'Pagination offset' },
                    { name: 'status', type: 'string', required: false, description: 'Filter by status (PENDING, CONFIRMED, FAILED, EXPIRED)' }
                ],
                request: {
                    curl: 'curl https://api.fiddupay.com/api/v1/merchants/payments?status=CONFIRMED \\\n  -H "Authorization: Bearer sk_live_..."',
                    node: 'const payments = await fiddupay.payments.list({ status: "CONFIRMED" });'
                },
                response: JSON.stringify([
                    {
                        payment_id: "pay_123",
                        crypto_type: "USDT_ETH",
                        amount: "100.00",
                        amount_usd: "100.00",
                        to_address: "0x742d35Cc6634C0532925a3b8D4C9db96590c6C87",
                        status: "CONFIRMED",
                        confirmations: 12,
                        required_confirmations: 12,
                        expires_at: "2026-02-04T10:20:00Z",
                        created_at: "2026-02-04T10:00:00Z",
                        confirmed_at: "2026-02-04T10:05:00Z",
                        description: "Stablecoin Order",
                        metadata: { order_id: "12345" },
                        network: "ETHEREUM",
                        deposit_address: "0x742d35Cc6634C0532925a3b8D4C9db96590c6C87",
                        payment_link: "https://pay.fiddupay.com/lnk_xyz789",
                        qr_code_data: "ethereum:0x742d35Cc6634C0532925a3b8D4C9db96590c6C87?amount=100.00",
                        fee_amount: "1.50",
                        fee_amount_usd: "1.50",
                        transaction_hash: "0x123abc...",
                        from_address: "0xsender...",
                        partial_payments: null,
                        last_verification_at: "2026-02-04T10:05:00Z"
                    }
                ], null, 2)
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
                    status: "CONFIRMED",
                    payment: {
                        payment_id: "pay_123",
                        crypto_type: "USDT_ETH",
                        amount: "100.00",
                        amount_usd: "100.00",
                        to_address: "0x742d35Cc6634C0532925a3b8D4C9db96590c6C87",
                        status: "CONFIRMED",
                        confirmations: 12,
                        required_confirmations: 12,
                        expires_at: "2026-02-04T10:20:00Z",
                        created_at: "2026-02-04T10:00:00Z",
                        confirmed_at: "2026-02-04T10:05:00Z",
                        description: "Stablecoin Order",
                        metadata: { order_id: "12345" },
                        network: "ETHEREUM",
                        deposit_address: "0x742d35Cc6634C0532925a3b8D4C9db96590c6C87",
                        payment_link: "https://pay.fiddupay.com/lnk_xyz789",
                        qr_code_data: "ethereum:0x742d35Cc6634C0532925a3b8D4C9db96590c6C87?amount=100.00",
                        fee_amount: "1.50",
                        fee_amount_usd: "1.50",
                        transaction_hash: "0x123abc...",
                        from_address: "0xsender...",
                        partial_payments: null,
                        last_verification_at: "2026-02-04T10:05:00Z"
                    }
                }, null, 2)
            },
            {
                id: 'list-transactions',
                method: 'GET',
                path: '/api/v1/merchants/transactions',
                title: 'Unified Transaction Feed',
                description: 'Get a unified chronological feed of all payments, refunds, and withdrawals associated with your account.',
                parameters: [
                    { name: 'limit', type: 'integer', required: false, description: 'Number of records to return' },
                    { name: 'offset', type: 'integer', required: false, description: 'Pagination offset' },
                    { name: 'from_date', type: 'string', required: false, description: 'ISO-8601 start date' },
                    { name: 'to_date', type: 'string', required: false, description: 'ISO-8601 end date' },
                    { name: 'txn_type', type: 'string', required: false, description: 'payment, refund, or withdrawal' }
                ],
                request: {
                    curl: 'curl https://api.fiddupay.com/api/v1/merchants/transactions \\\n  -H "Authorization: Bearer sk_live_..."',
                    node: 'const transactions = await fiddupay.transactions.list();'
                },
                response: JSON.stringify({
                    transactions: [
                        { type: "payment", id: "pay_123", amount: "10.0", status: "completed", created_at: "2026-02-04T10:00:00Z" },
                        { type: "refund", id: "ref_456", amount: "5.0", status: "processed", created_at: "2026-02-04T11:00:00Z" }
                    ]
                }, null, 2)
            },
            {
                id: 'cancel-payment',
                method: 'POST',
                path: '/api/v1/merchants/payments/:payment_id/cancel',
                title: 'Cancel Payment',
                description: 'Explicitly cancel a pending payment request before it expires.',
                request: {
                    curl: 'curl -X POST https://api.fiddupay.com/api/v1/merchants/payments/pay_123/cancel \\\n  -H "Authorization: Bearer sk_live_..."',
                    node: 'await fiddupay.payments.cancel("pay_123");'
                },
                response: JSON.stringify({
                    status: "CANCELLED",
                    message: "Payment pay_123 has been cancelled"
                }, null, 2)
            },
            {
                id: 'finalize-payment-selection',
                method: 'POST',
                path: '/api/v1/merchants/payments/:payment_id/select',
                title: 'Select Payment Currency',
                description: 'Finalize cryptocurrency token choice path for a checkout payment link.',
                body: [
                    { name: 'crypto_type', type: 'string', required: true, description: 'Target token code (e.g. SOL, USDT_ETH)' }
                ],
                request: {
                    curl: 'curl -X POST https://api.fiddupay.com/api/v1/merchants/payments/pay_123/select \\\n  -H "Authorization: Bearer sk_live_..." \\\n  -d \'{"crypto_type": "SOL"}\'',
                    node: 'await fiddupay.payments.finalizeSelection("pay_123", "SOL");'
                },
                response: JSON.stringify({
                    success: true,
                    deposit_address: "9WzDX...",
                    amount: "1.234"
                }, null, 2)
            }
        ]
    },
    {
        id: 'address-only',
        title: 'Address-Only Mode (WIP)',
        description: 'Native-only cryptocurrency payments where the customer sends funds directly to your wallet. (Exclusive to Forwarding mode - UNDER DEVELOPMENT)',
        endpoints: [
            {
                id: 'create-address-only',
                method: 'POST',
                path: '/api/v1/merchants/address-only/create',
                title: 'Create Address-Only Payment (Experimental)',
                description: 'Generate a deposit address for a native currency payment. (Forbidden in Managed mode - Feature in Beta)',
                body: [
                    { name: 'crypto_type', type: 'string', required: true, description: 'SOL, BTC, ETH, BNB, MATIC, ARB' },
                    { name: 'merchant_address', type: 'string', required: true, description: 'Your on-chain wallet address' },
                    { name: 'requested_amount', type: 'string', required: true, description: 'Native amount (e.g. "1.5")' }
                ],
                request: {
                    curl: 'curl -X POST https://api.fiddupay.com/api/v1/merchants/address-only/create \\\n  -H "Authorization: Bearer sk_live_..." \\\n  -d \'{\n    "crypto_type": "SOL",\n    "merchant_address": "your_address...",\n    "requested_amount": "1.5"\n  }\'',
                    node: 'const payment = await fiddupay.addressOnly.create({\n  crypto_type: "SOL",\n  merchant_address: "...",\n  requested_amount: "1.5"\n});'
                },
                response: JSON.stringify({
                    payment_id: "addr_123",
                    gateway_deposit_address: "0x...",
                    requested_amount: "1.5",
                    customer_amount: "1.51",
                    processing_fee: "0.01",
                    customer_pays_fee: true,
                    customer_instructions: "Send exactly 1.51 SOL to the deposit address."
                }, null, 2)
            },
            {
                id: 'get-address-only-status',
                method: 'GET',
                path: '/api/v1/merchants/address-only/status',
                title: 'Get Status',
                description: 'Retrieve the current status of an address-only payment.',
                parameters: [
                    { name: 'payment_id', type: 'string', required: true, description: 'The payment ID' }
                ],
                request: {
                    curl: 'curl https://api.fiddupay.com/api/v1/merchants/address-only/status?payment_id=addr_123 \\\n  -H "Authorization: Bearer sk_live_..."',
                    node: 'const status = await fiddupay.addressOnly.getStatus("addr_123");'
                },
                response: JSON.stringify({
                    payment_id: "addr_123",
                    status: "confirmed",
                    amount_received: "1.51"
                }, null, 2)
            },
            {
                id: 'get-address-only-currencies',
                method: 'GET',
                path: '/api/v1/merchants/address-only/currencies',
                title: 'List Supported Currencies',
                description: 'List supported native cryptocurrencies for address-only payments.',
                request: {
                    curl: 'curl https://api.fiddupay.com/api/v1/merchants/address-only/currencies \\\n  -H "Authorization: Bearer sk_live_..."',
                    node: 'const currencies = await fiddupay.addressOnly.getCurrencies();'
                },
                response: JSON.stringify(["ETH", "BNB", "MATIC", "ARB", "SOL"], null, 2)
            },
            {
                id: 'get-address-only-stats',
                method: 'GET',
                path: '/api/v1/merchants/address-only/stats',
                title: 'Get Stats',
                description: 'Retrieve address-only mode performance and aggregate payment statistics.',
                request: {
                    curl: 'curl https://api.fiddupay.com/api/v1/merchants/address-only/stats \\\n  -H "Authorization: Bearer sk_live_..."',
                    node: 'const stats = await fiddupay.addressOnly.getStats();'
                },
                response: JSON.stringify({
                    total_payments: 45,
                    completed_payments: 42,
                    pending_payments: 3,
                    total_volume: "1250.75",
                    total_fees_collected: "12.50"
                }, null, 2)
            },
            {
                id: 'get-address-only-fee-setting',
                method: 'GET',
                path: '/api/v1/merchants/address-only/fee-setting',
                title: 'Get Fee Setting',
                description: 'Get details on who pays network processing fees in address-only mode.',
                request: {
                    curl: 'curl https://api.fiddupay.com/api/v1/merchants/address-only/fee-setting \\\n  -H "Authorization: Bearer sk_live_..."',
                    node: 'const feeSetting = await fiddupay.addressOnly.getFeeSetting();'
                },
                response: JSON.stringify({
                    customer_pays_fee: true,
                    description: "Customer pays processing fee"
                }, null, 2)
            },
            {
                id: 'update-address-only-fee-setting',
                method: 'PUT',
                path: '/api/v1/merchants/address-only/fee-setting',
                title: 'Update Fee Setting',
                description: 'Configure fee delegation options for native transfer gateway.',
                body: [
                    { name: 'customer_pays_fee', type: 'boolean', required: true, description: 'True if customer pays network transfer costs' }
                ],
                request: {
                    curl: 'curl -X PUT https://api.fiddupay.com/api/v1/merchants/address-only/fee-setting \\\n  -H "Authorization: Bearer sk_live_..." \\\n  -d \'{"customer_pays_fee": false}\'',
                    node: 'await fiddupay.addressOnly.updateFeeSetting(false);'
                },
                response: JSON.stringify({
                    success: true,
                    message: "Fee payment setting updated: Merchant pays fee",
                    customer_pays_fee: false
                }, null, 2)
            },
            {
                id: 'get-address-only-health',
                method: 'GET',
                path: '/api/v1/merchants/address-only/health',
                title: 'Get Health Status',
                description: 'Validate processing coverage health parameters for address-only payments.',
                request: {
                    curl: 'curl https://api.fiddupay.com/api/v1/merchants/address-only/health \\\n  -H "Authorization: Bearer sk_live_..."',
                    node: 'const health = await fiddupay.addressOnly.getHealth();'
                },
                response: JSON.stringify({
                    status: "healthy",
                    details: {
                        solana_rpc: "connected",
                        ethereum_rpc: "connected"
                    }
                }, null, 2)
            }
        ]
    },
    {
        id: 'refunds',
        title: 'Refunds',
        description: 'Manage returns and fund reversals for your customers.',
        endpoints: [
            {
                id: 'create-refund',
                method: 'POST',
                path: '/api/v1/merchants/refunds',
                title: 'Create Refund',
                description: 'Initialize a refund for a previously confirmed payment.',
                body: [
                    { name: 'payment_id', type: 'string', required: true, description: 'ID of original payment' },
                    { name: 'amount', type: 'string', required: true, description: 'Amount to refund' },
                    { name: 'reason', type: 'string', required: true, description: 'Reason for refund' }
                ],
                request: {
                    curl: 'curl -X POST https://api.fiddupay.com/api/v1/merchants/refunds \\\n  -H "Authorization: Bearer sk_live_..." \\\n  -d \'{\n    "payment_id": "pay_123",\n    "amount": "10.0",\n    "reason": "Customer request"\n  }\'',
                    node: 'const refund = await fiddupay.refunds.create({\n  payment_id: "pay_123",\n  amount: "10.0",\n  reason: "Customer request"\n});'
                },
                response: JSON.stringify({
                    refund_id: "ref_456",
                    status: "PENDING",
                    amount: "10.0"
                }, null, 2)
            },
            {
                id: 'list-refunds',
                method: 'GET',
                path: '/api/v1/merchants/refunds',
                title: 'List Refunds',
                description: 'Retrieve all refund requests for the merchant.',
                parameters: [
                    { name: 'limit', type: 'integer', required: false, description: 'Number of records to return' },
                    { name: 'offset', type: 'integer', required: false, description: 'Pagination offset' }
                ],
                request: {
                    curl: 'curl https://api.fiddupay.com/api/v1/merchants/refunds \\\n  -H "Authorization: Bearer sk_live_..."',
                    node: 'const refunds = await fiddupay.refunds.list();'
                },
                response: JSON.stringify([
                    {
                        refund_id: "ref_456",
                        payment_id: "pay_123",
                        status: "COMPLETED",
                        amount: "10.0",
                        reason: "Customer request",
                        created_at: "2026-02-04T11:00:00Z"
                    }
                ], null, 2)
            },
            {
                id: 'get-refund',
                method: 'GET',
                path: '/api/v1/merchants/refunds/:refund_id',
                title: 'Retrieve Refund',
                description: 'Get status and details of a specific refund request.',
                request: {
                    curl: 'curl https://api.fiddupay.com/api/v1/merchants/refunds/ref_456 \\\n  -H "Authorization: Bearer sk_live_..."',
                    node: 'const refund = await fiddupay.refunds.retrieve("ref_456");'
                },
                response: JSON.stringify({
                    refund_id: "ref_456",
                    status: "COMPLETED",
                    amount: "10.0"
                }, null, 2)
            },
            {
                id: 'complete-refund',
                method: 'POST',
                path: '/api/v1/merchants/refunds/:refund_id/complete',
                title: 'Complete Refund',
                description: 'Broadcast or confirm completion of a pending refund on-chain.',
                request: {
                    curl: 'curl -X POST https://api.fiddupay.com/api/v1/merchants/refunds/ref_456/complete \\\n  -H "Authorization: Bearer sk_live_..."',
                    node: 'await fiddupay.refunds.complete("ref_456");'
                },
                response: JSON.stringify({
                    status: "COMPLETED",
                    message: "Refund completed successfully"
                }, null, 2)
            }
        ]
    },
    {
        id: 'customers',
        title: 'Merchant Customers (Sub-Accounts)',
        description: 'Manage individual customer sub-accounts with dedicated deposit wallets.',
        endpoints: [
            {
                id: 'get-customers-summary',
                method: 'GET',
                path: '/api/v1/merchants/customers/summary',
                title: 'Get Customer Summary',
                description: 'Retrieve aggregate statistics across all platform customers, including total counts (active, flagged, recent) and the total aggregate balance converted to USD.',
                request: {
                    curl: 'curl https://api.fiddupay.com/api/v1/merchants/customers/summary \\\n  -H "Authorization: Bearer sk_live_..."',
                    node: 'const summary = await fiddupay.customers.getSummary();'
                },
                response: JSON.stringify({
                    total_customers: 150,
                    active_customers: 142,
                    flagged_customers: 3,
                    recent_customers: 12,
                    total_balance_usd: 12500.50
                }, null, 2)
            },
            {
                id: 'register-customer',
                method: 'POST',
                path: '/api/v1/merchants/customers',
                title: 'Register Customer',
                description: 'Create a new customer profile within your merchant account for dedicated wallet management.',
                body: [
                    { name: 'external_id', type: 'string', required: true, description: 'Your internal user ID' },
                    { name: 'email', type: 'string', required: false, description: 'Customer email' },
                    { name: 'first_name', type: 'string', required: false, description: 'Customer legal first name' },
                    { name: 'last_name', type: 'string', required: false, description: 'Customer legal last name' },
                    { name: 'metadata', type: 'object', required: false, description: 'Custom mapping data' }
                ],
                request: {
                    curl: 'curl -X POST https://api.fiddupay.com/api/v1/merchants/customers \\\n  -H "Authorization: Bearer sk_live_..." \\\n  -d \'{\n    "external_id": "user_1234",\n    "email": "cust@example.com",\n    "first_name": "John",\n    "last_name": "Doe"\n  }\'',
                    node: 'const customer = await fiddupay.customers.register({\n  external_id: "user_1234",\n  email: "cust@example.com",\n  first_name: "John",\n  last_name: "Doe"\n});'
                },
                response: JSON.stringify({
                    id: "mc_789",
                    external_id: "user_1234",
                    status: "ACTIVE"
                }, null, 2)
            },
            {
                id: 'list-customers',
                method: 'GET',
                path: '/api/v1/merchants/customers',
                title: 'List Customers',
                description: 'Retrieve registered customer sub-profiles with pagination filters.',
                parameters: [
                    { name: 'limit', type: 'integer', required: false, description: 'Number of records to return' },
                    { name: 'offset', type: 'integer', required: false, description: 'Pagination offset' }
                ],
                request: {
                    curl: 'curl https://api.fiddupay.com/api/v1/merchants/customers \\\n  -H "Authorization: Bearer sk_live_..."',
                    node: 'const customers = await fiddupay.customers.list();'
                },
                response: JSON.stringify([
                    {
                        id: "mc_789",
                        external_id: "user_1234",
                        email: "cust@example.com",
                        status: "ACTIVE",
                        created_at: "2026-02-04T10:00:00Z"
                    }
                ], null, 2)
            },
            {
                id: 'create-wallets',
                method: 'POST',
                path: '/api/v1/merchants/customers/:id/wallets',
                title: 'Provision Wallets',
                description: 'Automatically generate or assign deposit addresses for a customer across selected networks.',
                body: [
                    { name: 'networks', type: 'string[]', required: true, description: 'SOL, BTC, ETH, etc. (Empty for all)' }
                ],
                request: {
                    curl: 'curl -X POST https://api.fiddupay.com/api/v1/merchants/customers/user_1234/wallets \\\n  -H "Authorization: Bearer sk_live_..." \\\n  -d \'{"networks": ["SOL", "ETH"]}\'',
                    node: 'await fiddupay.customers.provisionCustomerWallets("user_1234", ["SOL", "ETH"]);'
                },
                response: JSON.stringify({
                    success: true,
                    wallets: [
                        { network: "SOL", address: "7x..." },
                        { network: "ETH", address: "0x..." }
                    ]
                }, null, 2)
            },
            {
                id: 'retrieve-customer-wallets',
                method: 'GET',
                path: '/api/v1/merchants/customers/:id/wallets',
                title: 'Retrieve Customer Wallets',
                description: 'Fetch the active configured deposit addresses assigned to a customer sub-account.',
                request: {
                    curl: 'curl https://api.fiddupay.com/api/v1/merchants/customers/user_1234/wallets \\\n  -H "Authorization: Bearer sk_live_..."',
                    node: 'const wallets = await fiddupay.customers.getCustomerWallets("user_1234");'
                },
                response: JSON.stringify([
                    { network: "SOL", address: "7x...", is_active: true },
                    { network: "ETH", address: "0x...", is_active: true }
                ], null, 2)
            },
            {
                id: 'get-customer-balances',
                method: 'GET',
                path: '/api/v1/merchants/customers/:id/balances',
                title: 'Get Customer Balances',
                description: 'Read the token balances across all provisioned chains for a customer.',
                request: {
                    curl: 'curl https://api.fiddupay.com/api/v1/merchants/customers/user_1234/balances \\\n  -H "Authorization: Bearer sk_live_..."',
                    node: 'const balances = await fiddupay.customers.getBalances("user_1234");'
                },
                response: JSON.stringify([
                    { crypto_type: "SOL", balance: "1.5", balance_usd: "150.00" },
                    { crypto_type: "USDT_ETH", balance: "10.0", balance_usd: "10.00" }
                ], null, 2)
            },
            {
                id: 'sweep-customer',
                method: 'POST',
                path: '/api/v1/merchants/customers/:id/sweep',
                title: 'Sweep Funds',
                description: 'Trigger a transfer of funds from a customer\'s sub-account directly into your main merchant balance.',
                body: [
                    { name: 'crypto_type', type: 'string', required: true, description: 'SOL, WSOL, USDT, etc.' },
                    { name: 'amount', type: 'string', required: false, description: 'Amount to sweep (omitted for entire balance)' }
                ],
                request: {
                    curl: 'curl -X POST https://api.fiddupay.com/api/v1/merchants/customers/user_1234/sweep \\\n  -H "Authorization: Bearer sk_live_..." \\\n  -d \'{\n    "crypto_type": "USDT",\n    "amount": "100.0"\n  }\'',
                    node: 'await fiddupay.customers.sweep("user_1234", {\n  crypto_type: "USDT",\n  amount: "100.0"\n});'
                },
                response: JSON.stringify({
                    success: true,
                    swept_amount: "100.0",
                    message: "Funds swept successfully"
                }, null, 2)
            },
            {
                id: 'customer-pay-merchant',
                method: 'POST',
                path: '/api/v1/merchants/customers/:id/pay-merchant',
                title: 'Internal Payment',
                description: 'Initiate an internal payment from a customer\'s sub-account balance to your master balance.',
                body: [
                    { name: 'crypto_type', type: 'string', required: true, description: 'SOL, USDT, etc.' },
                    { name: 'amount', type: 'string', required: true, description: 'Amount to pay' },
                    { name: 'reference_id', type: 'string', required: false, description: 'Your internal reference' }
                ],
                request: {
                    curl: 'curl -X POST https://api.fiddupay.com/api/v1/merchants/customers/user_1234/pay-merchant \\\n  -H "Authorization: Bearer sk_live_..." \\\n  -d \'{\n    "crypto_type": "USDT",\n    "amount": "25.0",\n    "reference_id": "order_789"\n  }\'',
                    node: 'await fiddupay.customers.payMerchant("user_1234", {\n  crypto_type: "USDT",\n  amount: "25.0",\n  reference_id: "order_789"\n});'
                },
                response: JSON.stringify({
                    success: true,
                    transaction: { id: "tx_999", amount: "25.0" },
                    message: "Payment processed internally"
                }, null, 2)
            },
            {
                id: 'get-customer-transactions',
                method: 'GET',
                path: '/api/v1/merchants/customers/:id/transactions',
                title: 'List Customer Transactions',
                description: 'Retrieve a chronological list of transactions associated with a customer sub-account.',
                parameters: [
                    { name: 'limit', type: 'integer', required: false, description: 'Number of records to return' },
                    { name: 'offset', type: 'integer', required: false, description: 'Pagination offset' }
                ],
                request: {
                    curl: 'curl https://api.fiddupay.com/api/v1/merchants/customers/user_1234/transactions \\\n  -H "Authorization: Bearer sk_live_..."',
                    node: 'const txs = await fiddupay.customers.getTransactions("user_1234");'
                },
                response: JSON.stringify([
                    { id: "tx_999", amount: "25.0", crypto_type: "USDT", status: "completed", type: "pay-merchant", created_at: "2026-02-04T12:00:00Z" }
                ], null, 2)
            },
            {
                id: 'customer-withdraw',
                method: 'POST',
                path: '/api/v1/merchants/customers/:id/withdraw',
                title: 'Withdraw for Customer',
                description: 'Request a withdrawal from a customer\'s sub-account balance to an external address.',
                body: [
                    { name: 'crypto_type', type: 'string', required: true, description: 'SOL, BTC, ETH, etc.' },
                    { name: 'amount', type: 'string', required: true, description: 'Amount to withdraw' },
                    { name: 'destination_address', type: 'string', required: true, description: 'External wallet address' }
                ],
                request: {
                    curl: 'curl -X POST https://api.fiddupay.com/api/v1/merchants/customers/user_1234/withdraw \\\n  -H "Authorization: Bearer sk_live_..." \\\n  -d \'{\n    "crypto_type": "SOL",\n    "amount": "1.5",\n    "destination_address": "8x..."\n  }\'',
                    node: 'await fiddupay.customers.withdraw("user_1234", {\n  crypto_type: "SOL",\n  amount: "1.5",\n  destination_address: "8x..."\n});'
                },
                response: JSON.stringify({
                    withdrawal: { id: "wth_001", status: "pending" },
                    message: "Withdrawal initiated"
                }, null, 2)
            },
            {
                id: 'customer-status',
                method: 'PATCH',
                path: '/api/v1/merchants/customers/:id/status',
                title: 'Update Customer Status',
                description: 'Change a customer\'s operational status (active, suspended, or inactive).',
                body: [
                    { name: 'status', type: 'string', required: true, description: 'active, suspended, or inactive' }
                ],
                request: {
                    curl: 'curl -X PATCH https://api.fiddupay.com/api/v1/merchants/customers/user_1234/status \\\n  -H "Authorization: Bearer sk_live_..." \\\n  -d \'{"status": "suspended"}\'',
                    node: 'await fiddupay.customers.updateStatus("user_1234", { status: "suspended" });'
                },
                response: JSON.stringify({
                    message: "Customer status updated successfully"
                }, null, 2)
            },
            {
                id: 'customer-permissions',
                method: 'PATCH',
                path: '/api/v1/merchants/customers/:id/permissions',
                title: 'Update Permissions',
                description: 'Configure granular withdrawal permissions and transaction limits for a customer.',
                body: [
                    { name: 'can_withdraw', type: 'boolean', required: false, description: 'Allow/block customer withdrawals' },
                    { name: 'withdrawal_limit', type: 'string', required: false, description: 'Custom volume allowance limit' }
                ],
                request: {
                    curl: 'curl -X PATCH https://api.fiddupay.com/api/v1/merchants/customers/user_1234/permissions \\\n  -H "Authorization: Bearer sk_live_..." \\\n  -d \'{"can_withdraw": false}\'',
                    node: 'await fiddupay.customers.updatePermissions("user_1234", { can_withdraw: false });'
                },
                response: JSON.stringify({
                    message: "Customer permissions updated successfully"
                }, null, 2)
            },
            {
                id: 'customer-deposit-address',
                method: 'GET',
                path: '/api/v1/merchants/customers/:id/deposit-address/:crypto_type',
                title: 'Get Deposit Address',
                description: 'Retrieve the specific on-chain deposit address for a customer and asset.',
                request: {
                    curl: 'curl https://api.fiddupay.com/api/v1/merchants/customers/user_1234/deposit-address/SOL \\\n  -H "Authorization: Bearer sk_live_..."',
                    node: 'const { address } = await fiddupay.customers.getDepositAddress("user_1234", "SOL");'
                },
                response: JSON.stringify({
                    address: "7x...",
                    crypto_type: "SOL"
                }, null, 2)
            },
            {
                id: 'bulk-provision-wallets',
                method: 'POST',
                path: '/api/v1/merchants/customers/bulk-provision',
                title: 'Bulk Provision Wallets',
                description: 'Generate deposit addresses for multiple customers at once or for your entire customer base.',
                body: [
                    { name: 'customer_ids', type: 'string[]', required: false, description: 'Specific IDs to process' },
                    { name: 'all_customers', type: 'boolean', required: false, description: 'Process ALL registered customers' }
                ],
                request: {
                    curl: 'curl -X POST https://api.fiddupay.com/api/v1/merchants/customers/bulk-provision \\\n  -H "Authorization: Bearer sk_live_..." \\\n  -d \'{"all_customers": true}\'',
                    node: 'await fiddupay.customers.bulkProvision({ all_customers: true });'
                },
                response: JSON.stringify({
                    count: 15,
                    message: "15 customers successfully provisioned"
                }, null, 2)
            },
            {
                id: 'deactivate-customer',
                method: 'POST',
                path: '/api/v1/merchants/customers/:id/deactivate',
                title: 'Deactivate Customer',
                description: 'Permanently deactivate a customer, preventing any future activity while preserving history.',
                request: {
                    curl: 'curl -X POST https://api.fiddupay.com/api/v1/merchants/customers/user_1234/deactivate \\\n  -H "Authorization: Bearer sk_live_..."',
                    node: 'await fiddupay.customers.deactivate("user_1234");'
                },
                response: JSON.stringify({
                    message: "Customer deactivated successfully"
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
                id: 'setup-wallet',
                method: 'POST',
                path: '/api/v1/merchants/wallets',
                title: 'Unified Wallet Setup',
                description: 'A single endpoint to configure or generate wallets for any supported cryptocurrency.',
                body: [
                    { name: 'crypto_type', type: 'string', required: true, description: 'SOL, BTC, ETH, WSOL, USDT_SPL, etc.' },
                    { name: 'mode', type: 'string', required: true, description: 'address or generate' },
                    { name: 'address', type: 'string', required: false, description: 'Required for mode "address"' },
                    { name: 'is_active', type: 'boolean', required: false, description: 'Set as primary wallet' }
                ],
                request: {
                    curl: 'curl -X POST https://api.fiddupay.com/api/v1/merchants/wallets \\\n  -H "Authorization: Bearer sk_live_..." \\\n  -d \'{\n    "crypto_type": "SOL",\n    "mode": "generate"\n  }\'',
                    node: 'const wallet = await fiddupay.wallets.setup({\n  crypto_type: "SOL",\n  mode: "generate"\n});'
                },
                response: JSON.stringify({
                    wallet: { crypto_type: "SOL", address: "your_new_address...", network: "SOLANA", is_active: true },
                    mode: "generate",
                    message: "Wallet generated successfully"
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
                    node: 'const capable = await fiddupay.withdrawals.checkCapability("SOL");'
                },
                response: JSON.stringify({
                    capable: true,
                    network: "SOLANA",
                    min_withdrawal: "0.1"
                }, null, 2)
            },
            {
                id: 'gas-check',
                method: 'GET',
                path: '/api/v1/merchants/wallets/gas-check',
                title: 'Check Wallet Gas Status',
                description: 'Verifies if managed wallets have sufficient gas for immediate withdrawals.',
                request: {
                    curl: 'curl https://api.fiddupay.com/api/v1/merchants/wallets/gas-check \\\n  -H "Authorization: Bearer sk_live_..."',
                    node: 'const status = await fiddupay.withdrawals.validateGas("SOL", "1.0");'
                },
                response: JSON.stringify({
                    sufficient: true,
                    wallets: [
                        { crypto_type: "SOL", status: "OK", balance: "0.1" }
                    ]
                }, null, 2)
            },
            {
                id: 'gas-estimates',
                method: 'GET',
                path: '/api/v1/merchants/wallets/gas-estimates',
                title: 'Get Gas Estimates',
                description: 'Retrieve real-time gas fee estimates for supported networks natively.',
                request: {
                    curl: 'curl https://api.fiddupay.com/api/v1/merchants/wallets/gas-estimates \\\n  -H "Authorization: Bearer sk_live_..."',
                    node: 'const estimates = await fiddupay.withdrawals.getGasEstimates();'
                },
                response: JSON.stringify({
                    "ETH": { "low": 15, "average": 20, "high": 30 },
                    "SOL": { "average": 0.000005 }
                }, null, 2)
            },
            {
                id: 'revoke-wallet',
                method: 'DELETE',
                path: '/api/v1/merchants/wallets/:crypto_type',
                title: 'Revoke Wallet',
                description: 'Remove a specific wallet configuration or deposit address from your account.',
                request: {
                    curl: 'curl -X DELETE https://api.fiddupay.com/api/v1/merchants/wallets/SOL \\\n  -H "Authorization: Bearer sk_live_..."',
                    node: 'await fiddupay.wallets.revoke("SOL");'
                },
                response: JSON.stringify({
                    success: true,
                    message: "Wallet configuration removed successfully"
                }, null, 2)
            },
            {
                id: 'wallet-balances',
                method: 'GET',
                path: '/api/v1/merchants/wallets/balances',
                title: 'Get Wallet Balances',
                description: 'Retrieve real-time on-chain balances for all your configured settlement wallets.',
                request: {
                    curl: 'curl https://api.fiddupay.com/api/v1/merchants/wallets/balances \\\n  -H "Authorization: Bearer sk_live_..."',
                    node: 'const balances = await fiddupay.wallets.getBalances();'
                },
                response: JSON.stringify({
                    wallets: [
                        { crypto_type: "SOL", available_balance: "5.234", available_usd: "523.40", total_balance: "5.234", total_usd: "523.40" },
                        { crypto_type: "USDT_ETH", available_balance: "100.0", available_usd: "100.00", total_balance: "100.0", total_usd: "100.00" }
                    ]
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
                id: 'update-security-settings',
                method: 'PUT',
                path: '/api/v1/merchants/security/settings',
                title: 'Update Security Settings',
                description: 'Configure and update security configurations and preferences.',
                body: [
                    { name: 'two_factor_enabled', type: 'boolean', required: false, description: 'Toggle 2FA requirements' },
                    { name: 'ip_whitelist_enforced', type: 'boolean', required: false, description: 'Force check inbound calls against whitelists' },
                    { name: 'alert_email', type: 'string', required: false, description: 'Security alert delivery email' }
                ],
                request: {
                    curl: 'curl -X PUT https://api.fiddupay.com/api/v1/merchants/security/settings \\\n  -H "Authorization: Bearer sk_live_..." \\\n  -d \'{"two_factor_enabled": true}\'',
                    node: 'await fiddupay.security.updateSettings({ two_factor_enabled: true });'
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
            },
            {
                id: 'acknowledge-alert',
                method: 'POST',
                path: '/api/v1/merchants/security/alerts/:alert_id/acknowledge',
                title: 'Acknowledge Security Alert',
                description: 'Mark a security alert as acknowledged to clear it from the active dashboard view.',
                request: {
                    curl: 'curl -X POST https://api.fiddupay.com/api/v1/merchants/security/alerts/alt_123/acknowledge \\\n  -H "Authorization: Bearer sk_live_..."',
                    node: 'await fiddupay.security.acknowledgeAlert("alt_123");'
                },
                response: JSON.stringify({
                    success: true,
                    message: "Alert alt_123 acknowledged."
                }, null, 2)
            },
            {
                id: 'get-security-events',
                method: 'GET',
                path: '/api/v1/merchants/security/events',
                title: 'List Security Events',
                description: 'Retrieve a log of automated security actions and access events.',
                request: {
                    curl: 'curl https://api.fiddupay.com/api/v1/merchants/security/events \\\n  -H "Authorization: Bearer sk_live_..."',
                    node: 'const events = await fiddupay.security.getEvents();'
                },
                response: JSON.stringify([
                    { id: "evt_345", event_type: "LOGIN_SUCCESS", ip_address: "192.168.1.1", created_at: "2024-01-01T12:00:00Z" }
                ], null, 2)
            },
            {
                id: 'get-balance-alerts',
                method: 'GET',
                path: '/api/v1/merchants/security/balance-alerts',
                title: 'List Balance Alerts',
                description: 'Retrieve alerts specifically triggered by unexpectedly high or low treasury balances.',
                request: {
                    curl: 'curl https://api.fiddupay.com/api/v1/merchants/security/balance-alerts \\\n  -H "Authorization: Bearer sk_live_..."',
                    node: 'const balanceAlerts = await fiddupay.security.getBalanceAlerts();'
                },
                response: JSON.stringify([
                    { id: "bal_alt_67", threshold_usd: "50000.00", current_usd: "65000.00", message: "Accumulated volume exceeds threshold" }
                ], null, 2)
            },
            {
                id: 'resolve-balance-alert',
                method: 'POST',
                path: '/api/v1/merchants/security/balance-alerts/:alert_id/resolve',
                title: 'Resolve Balance Alert',
                description: 'Mark a liquidity or balance threshold alert as resolved.',
                request: {
                    curl: 'curl -X POST https://api.fiddupay.com/api/v1/merchants/security/balance-alerts/bal_alt_67/resolve \\\n  -H "Authorization: Bearer sk_live_..."',
                    node: 'await fiddupay.security.resolveBalanceAlert("bal_alt_67");'
                },
                response: JSON.stringify({
                    success: true,
                    message: "Balance alert resolved."
                }, null, 2)
            },
            {
                id: 'security-gas-check',
                method: 'GET',
                path: '/api/v1/merchants/security/gas-check',
                title: 'Monitor Gas Status',
                description: 'Perform a centralized security check on the gas levels across all active managed wallets.',
                request: {
                    curl: 'curl https://api.fiddupay.com/api/v1/merchants/security/gas-check \\\n  -H "Authorization: Bearer sk_live_..."',
                    node: 'const check = await fiddupay.security.gasCheck();'
                },
                response: JSON.stringify({
                    status: "healthy",
                    warnings: []
                }, null, 2)
            },
            {
                id: 'toggle-master-wallet-lock',
                method: 'POST',
                path: '/api/v1/merchants/security/wallets/lock',
                title: 'Toggle Master Wallet Lock',
                description: 'Enable or disable the global lock on merchant-owned settlement wallets.',
                body: [
                    { name: 'locked', type: 'boolean', required: true, description: 'True to lock, false to unlock' },
                    { name: 'password', type: 'string', required: true, description: 'Merchant account password for confirmation' }
                ],
                request: {
                    curl: 'curl -X POST https://api.fiddupay.com/api/v1/merchants/security/wallets/lock \\\n  -H "Authorization: Bearer sk_live_..." \\\n  -d \'{"locked": true, "password": "your_password"}\'',
                    node: 'await fiddupay.security.toggleWalletLock(true, "your_password");'
                },
                response: JSON.stringify({
                    locked: true,
                    message: "Master wallet security lock enabled"
                }, null, 2)
            },
            {
                id: 'toggle-customer-wallet-lock',
                method: 'POST',
                path: '/api/v1/merchants/security/customers/wallets/lock',
                title: 'Toggle Customer Wallet Lock',
                description: 'Enable or disable the protection lock for all provisioned customer deposit wallets.',
                body: [
                    { name: 'locked', type: 'boolean', required: true, description: 'True to lock, false to unlock' },
                    { name: 'password', type: 'string', required: true, description: 'Merchant account password for confirmation' }
                ],
                request: {
                    curl: 'curl -X POST https://api.fiddupay.com/api/v1/merchants/security/customers/wallets/lock \\\n  -H "Authorization: Bearer sk_live_..." \\\n  -d \'{"locked": true, "password": "your_password"}\'',
                    node: 'await fiddupay.security.toggleCustomerWalletLock(true, "your_password");'
                },
                response: JSON.stringify({
                    locked: true,
                    message: "Customer wallet security lock enabled"
                }, null, 2)
            },
            {
                id: 'set-transaction-pin',
                method: 'POST',
                path: '/api/v1/merchants/security/transaction-pin',
                title: 'Set Transaction PIN',
                description: 'Set or update your 4-digit transaction PIN required for sensitive operations like withdrawals.',
                body: [
                    { name: 'pin', type: 'string', required: true, description: '4-digit numeric PIN' }
                ],
                request: {
                    curl: 'curl -X POST https://api.fiddupay.com/api/v1/merchants/security/transaction-pin \\\n  -H "Authorization: Bearer sk_live_..." \\\n  -d \'{"pin": "1234"}\'',
                    node: 'await fiddupay.security.setTransactionPin("1234");'
                },
                response: JSON.stringify({
                    message: "Transaction PIN updated successfully"
                }, null, 2)
            },
            {
                id: 'verify-transaction-pin',
                method: 'POST',
                path: '/api/v1/merchants/security/transaction-pin/verify',
                title: 'Verify Transaction PIN',
                description: 'Validate a transaction PIN for verification purposes.',
                body: [
                    { name: 'pin', type: 'string', required: true, description: '4-digit numeric PIN' }
                ],
                request: {
                    curl: 'curl -X POST https://api.fiddupay.com/api/v1/merchants/security/transaction-pin/verify \\\n  -H "Authorization: Bearer sk_live_..." \\\n  -d \'{"pin": "1234"}\'',
                    node: 'const { valid } = await fiddupay.security.verifyTransactionPin("1234");'
                },
                response: JSON.stringify({
                    valid: true
                }, null, 2)
            },
            {
                id: 'update-password',
                method: 'POST',
                path: '/api/v1/merchants/security/password',
                title: 'Update Password',
                description: 'Update the merchant account password by confirming the current password.',
                body: [
                    { name: 'current_password', type: 'string', required: true, description: 'The existing account password' },
                    { name: 'new_password', type: 'string', required: true, description: 'The new password to set' }
                ],
                request: {
                    curl: 'curl -X POST https://api.fiddupay.com/api/v1/merchants/security/password \\\n  -H "Authorization: Bearer sk_live_..." \\\n  -d \'{"current_password": "old_pass", "new_password": "new_pass"}\'',
                    node: 'await fiddupay.security.updatePassword({\n  current_password: "old_pass",\n  new_password: "new_pass"\n});'
                },
                response: JSON.stringify({
                    message: "Password updated successfully"
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
                    node: 'const stats = await fiddupay.analytics.retrieve();'
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
                    node: 'const balance = await fiddupay.merchants.getBalance();'
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
                    node: 'const fees = await fiddupay.merchants.getFeeSetting();'
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
            },
            {
                id: 'get-withdrawal',
                method: 'GET',
                path: '/api/v1/merchants/withdrawals/:withdrawal_id',
                title: 'Get Withdrawal Details',
                description: 'Retrieve status, network, amount, and transaction hash of a specific withdrawal.',
                request: {
                    curl: 'curl https://api.fiddupay.com/api/v1/merchants/withdrawals/wth_123 \\\n  -H "Authorization: Bearer sk_live_..."',
                    node: 'const details = await fiddupay.withdrawals.get("wth_123");'
                },
                response: JSON.stringify({
                    id: "wth_123",
                    status: "completed",
                    amount: "10.0",
                    crypto_type: "SOL",
                    destination: "0x...",
                    tx_hash: "0x...",
                    created_at: "2026-02-04T10:00:00Z"
                }, null, 2)
            },
            {
                id: 'process-withdrawal',
                method: 'POST',
                path: '/api/v1/merchants/withdrawals/:id/process',
                title: 'Process (Broadcast) Withdrawal',
                description: 'Explicitly trigger the broadcasting of a pending withdrawal to the blockchain. Requires your encryption password.',
                body: [
                    { name: 'encryption_password', type: 'string', required: true, description: 'Merchant-set password' }
                ],
                request: {
                    curl: 'curl -X POST https://api.fiddupay.com/api/v1/merchants/withdrawals/wth_123/process \\\n  -H "Authorization: Bearer sk_live_..." \\\n  -d \'{"encryption_password": "..."}\'',
                    node: 'await fiddupay.withdrawals.process("wth_123", "your_password");'
                },
                response: JSON.stringify({
                    success: true,
                    tx_hash: "0x...",
                    message: "Withdrawal broadcasted successfully"
                }, null, 2)
            },
            {
                id: 'cancel-withdrawal',
                method: 'POST',
                path: '/api/v1/merchants/withdrawals/:id/cancel',
                title: 'Cancel Withdrawal',
                description: 'Abort a pending withdrawal request before it is broadcasted.',
                request: {
                    curl: 'curl -X POST https://api.fiddupay.com/api/v1/merchants/withdrawals/wth_123/cancel \\\n  -H "Authorization: Bearer sk_live_..."',
                    node: 'await fiddupay.withdrawals.cancel("wth_123");'
                },
                response: JSON.stringify({
                    status: "CANCELLED",
                    message: "Withdrawal wth_123 has been cancelled"
                }, null, 2)
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
                subSections: [
                    {
                        title: 'Settings & Authentication',
                        items: [
                            { key: 'merchant_registration', description: 'Merchant account creation' },
                            { key: 'login', description: 'Successful logins' },
                            { key: 'api_key_generation', description: 'API key modifications' },
                            { key: 'api_key_rotation', description: 'API key rotation' },
                            { key: 'environment_switch', description: 'Sandbox/Live switches' },
                            { key: 'merchant_settings_update', description: 'Settings modifications' },
                            { key: 'wallet_lock_toggle', description: 'Master wallet security status change' },
                            { key: 'test_webhook_trigger', description: 'Triggering test delivery' },
                            { key: 'transaction_pin_set', description: 'PIN configuration' }
                        ]
                    },
                    {
                        title: 'Customer Operations',
                        items: [
                            { key: 'customer_registration', description: 'Customer sub-account setup' },
                            { key: 'wallet_provisioning', description: 'Pre-generating customer keys' },
                            { key: 'bulk_wallet_provisioning', description: 'Batch provisioning trigger' },
                            { key: 'customer_status_updated', description: 'Banning or flagging customers' },
                            { key: 'customer_permissions_updated', description: 'Adjusting feature flags' },
                            { key: 'wallet_sweep', description: 'Emptying specific customer wallets' },
                            { key: 'customer_withdrawal', description: 'Customer withdrawal request' },
                            { key: 'customer_deactivation', description: 'Customer exclusion' }
                        ]
                    },
                    {
                        title: 'Payments & Transactions',
                        items: [
                            { key: 'payment_creation', description: 'Inbound multi-chain payment trigger' },
                            { key: 'payment_cancellation', description: 'Aborting payment setup' },
                            { key: 'payment_verification', description: 'Manual hash verification' },
                            { key: 'payment_simulation', description: 'Testing sandbox flows' },
                            { key: 'payment_selection_finalized', description: 'Asset path set on payment' },
                            { key: 'address_only_payment_creation', description: 'Native-mode payment trigger' },
                            { key: 'address_only_fee_setting_update', description: 'Fee toggle changed' }
                        ]
                    }
                ],
                request: {
                    curl: 'curl https://api.fiddupay.com/api/v1/merchants/audit-logs \\\n  -H "Authorization: Bearer sk_live_..."',
                    node: 'const logs = await fiddupay.merchants.getAuditLogs();'
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
                    node: 'const history = await fiddupay.merchants.getBalanceHistory();'
                },
                response: JSON.stringify([
                    { date: "2026-02-03", crypto_type: "USDT_ETH", balance: "1500.00" }
                ], null, 2)
            }
        ]
    },
    {
        id: 'invoices',
        title: 'Invoice Management',
        description: 'Generate and manage crypto payment invoices for your customers.',
        endpoints: [
            {
                id: 'create-invoice',
                method: 'POST',
                path: '/api/v1/merchants/invoices',
                title: 'Create Invoice',
                description: 'Generate a new invoice linking an external transaction or payment requirement.',
                body: [
                    { name: 'items', type: 'array', required: true, description: 'Array of invoice items: {description, quantity, unit_price, amount}' },
                    { name: 'customer_email', type: 'string', required: false, description: 'Email address of client' },
                    { name: 'customer_name', type: 'string', required: false, description: 'Full name of recipient' },
                    { name: 'tax', type: 'string', required: false, description: 'Tax override value' },
                    { name: 'due_date', type: 'string', required: false, description: 'YYYY-MM-DD completion date' }
                ],
                request: {
                    curl: 'curl -X POST https://api.fiddupay.com/api/v1/merchants/invoices \\\n  -H "Authorization: Bearer sk_live_..." \\\n  -d \'{\n    "customer_email": "hello@fiddupay.com",\n    "items": [\n      {"description": "Consulting", "quantity": 1, "unit_price": "50.0", "amount": "50.0"}\n    ]\n  }\'',
                    node: 'const invoice = await fiddupay.invoices.create({\n  customer_email: "hello@fiddupay.com",\n  items: [\n    { description: "Consulting", quantity: 1, unit_price: "50.0", amount: "50.0" }\n  ]\n});'
                },
                response: JSON.stringify({
                    id: "inv_123",
                    url: "https://pay.fiddupay.com/inv_123",
                    status: "PENDING"
                }, null, 2)
            },
            {
                id: 'list-invoices',
                method: 'GET',
                path: '/api/v1/merchants/invoices',
                title: 'List Invoices',
                description: 'Retrieve your invoice history.',
                request: {
                    curl: 'curl https://api.fiddupay.com/api/v1/merchants/invoices \\\n  -H "Authorization: Bearer sk_live_..."',
                    node: 'const invoices = await fiddupay.invoices.list();'
                },
                response: JSON.stringify([
                    { id: "inv_123", status: "PAID", amount_usd: "50.0" }
                ], null, 2)
            },
            {
                id: 'get-invoice',
                method: 'GET',
                path: '/api/v1/merchants/invoices/:invoice_id',
                title: 'Get Invoice Details',
                description: 'Look up specific details for an invoice by its ID.',
                request: {
                    curl: 'curl https://api.fiddupay.com/api/v1/merchants/invoices/inv_123 \\\n  -H "Authorization: Bearer sk_live_..."',
                    node: 'const invoice = await fiddupay.invoices.get("inv_123");'
                },
                response: JSON.stringify({
                    id: "inv_123",
                    status: "PAID",
                    created_at: "2024-01-01T12:00:00Z"
                }, null, 2)
            }
        ]
    },
    {
        id: 'public-api',
        title: 'Public API',
        description: 'Publishable Key endpoints for pure frontend/no-code integrations. These do not require your Secret API key.',
        endpoints: [
            {
                id: 'create-public-payment',
                method: 'POST',
                path: '/api/v1/public/payments/create',
                title: 'Create Public Payment',
                description: 'Initialize a payment via Publishable Key (for pure no-code frontend widgets). Returns a payment ID and specialized payment URL.',
                body: [
                    { name: 'publishable_key', type: 'string', required: true, description: 'Your account publishable key (pub_...)' },
                    { name: 'amount_usd', type: 'string', required: false, description: 'Required if amount is omitted' },
                    { name: 'amount', type: 'string', required: false, description: 'Required if amount_usd is omitted' },
                    { name: 'crypto_type', type: 'string', required: false, description: 'Specific asset (e.g., SOL, USDT_ETH)' },
                    { name: 'description', type: 'string', required: false, description: 'Payment description' }
                ],
                request: {
                    curl: 'curl -X POST https://api.fiddupay.com/api/v1/public/payments/create \\\n  -H "Content-Type: application/json" \\\n  -d \'{\n    "publishable_key": "pub_live_...",\n    "amount_usd": "100.00"\n  }\'',
                    node: 'const { payment_url } = await fiddupay.public.createPayment({\n  publishable_key: "pub_live_...",\n  amount_usd: "100.00"\n});'
                },
                response: JSON.stringify({
                    payment_id: "pay_xyz",
                    payment_url: "https://pay.fiddupay.com/pay_xyz"
                }, null, 2)
            }
        ]
    },
    {
        id: 'notifications',
        title: 'Notifications',
        description: 'Manage account-level notifications and system alerts for the merchant dashboard.',
        endpoints: [
            {
                id: 'list-notifications',
                method: 'GET',
                path: '/api/v1/merchants/notifications',
                title: 'List Notifications',
                description: 'Retrieve latest account notifications, filtered by environment.',
                parameters: [
                    { name: 'limit', type: 'integer', required: false, description: 'Defaults to 20' },
                    { name: 'offset', type: 'integer', required: false, description: 'Pagination offset' }
                ],
                request: {
                    curl: 'curl https://api.fiddupay.com/api/v1/merchants/notifications \\\n  -H "Authorization: Bearer sk_live_..."',
                    node: 'const { notifications } = await fiddupay.notifications.list();'
                },
                response: JSON.stringify({
                    notifications: [
                        { id: "not_123", type: "low_balance", message: "Low balance on ETH", read: false, created_at: "2026-04-14T10:00:00Z" }
                    ],
                    total: 1,
                    unread_count: 1
                }, null, 2)
            },
            {
                id: 'mark-read',
                method: 'PATCH',
                path: '/api/v1/merchants/notifications/:id/read',
                title: 'Mark Read',
                description: 'Mark a specific notification (or all if ID is omitted) as read.',
                request: {
                    curl: 'curl -X PATCH https://api.fiddupay.com/api/v1/merchants/notifications/not_123/read \\\n  -H "Authorization: Bearer sk_live_..."',
                    node: 'await fiddupay.notifications.markRead("not_123");'
                },
                response: JSON.stringify({
                    status: "success",
                    affected: 1
                }, null, 2)
            },
            {
                id: 'delete-notification',
                method: 'DELETE',
                path: '/api/v1/merchants/notifications/:id',
                title: 'Delete Notification',
                description: 'Remove a specific notification (or all if ID is omitted) permanently.',
                request: {
                    curl: 'curl -X DELETE https://api.fiddupay.com/api/v1/merchants/notifications/not_123 \\\n  -H "Authorization: Bearer sk_live_..."',
                    node: 'await fiddupay.notifications.delete("not_123");'
                },
                response: JSON.stringify({
                    status: "success",
                    affected: 1
                }, null, 2)
            }
        ]
    },
    {
        id: 'webhooks',
        title: 'Webhooks',
        description: 'FidduPay sends real-time event notifications to your server via HTTP POST requests. Every event is wrapped in a structured envelope with a unique ID, type, and timestamp. You should verify the `X-Signature` header on every delivery using your webhook signing secret.',
        endpoints: [
            {
                id: 'webhook-event-format',
                method: 'POST',
                path: 'POST https://your-server.com/webhook',
                title: 'Event Envelope & Types',
                description: 'All webhook deliveries share the same envelope structure regardless of event type. The `type` field determines which event occurred. The `data` object contains event-specific payload fields.',
                subSections: [
                    {
                        title: 'Payment Events',
                        items: [
                            { key: 'payment.confirmed', description: 'A payment was confirmed on-chain' },
                            { key: 'payment.expired', description: 'A payment window closed without a confirmed deposit' },
                            { key: 'refund.completed', description: 'A refund was successfully processed' }
                        ]
                    },
                    {
                        title: 'Merchant & Balance Events',
                        items: [
                            { key: 'merchant.deposit', description: 'Funds were credited to your merchant balance' },
                            { key: 'customer.deposit', description: 'A customer sub-account received a deposit' },
                            { key: 'address_only_payment_status', description: 'An address-only payment status changed' }
                        ]
                    },
                    {
                        title: 'System Events',
                        items: [
                            { key: 'webhook.test', description: 'Triggered by the "Test Webhook" button in settings' }
                        ]
                    }
                ],
                request: {
                    curl: `# FidduPay delivers to your server:
POST https://your-server.com/webhook
Content-Type: application/json
X-Signature: t=1743004800,v1=abc123...

{
  "id": "evt_5f9a2c3b4",
  "type": "payment.confirmed",
  "created_at": "2026-03-24T15:00:00Z",
  "data": {
    "payment_id": "pay_5f9a2c3b4",
    "status": "CONFIRMED",
    "amount": "150.00",
    "crypto_type": "SOL",
    "transaction_hash": "3xKp..."
  }
}`,
                    node: `import { FidduPay, Webhooks } from '@fiddupay/node-sdk';

// Express example
app.post('/webhook', express.raw({ type: 'application/json' }), (req, res) => {
  const sig = req.headers['x-signature'];
  let event;

  try {
    event = Webhooks.constructEvent(req.body, sig, process.env.WEBHOOK_SECRET);
  } catch (err) {
    return res.status(400).send('Invalid signature');
  }

  switch (event.type) {
    case 'payment.confirmed':
      console.log('Payment confirmed:', event.data.payment_id);
      break;
    case 'payment.expired':
      console.log('Payment expired:', event.data.payment_id);
      break;
    case 'refund.completed':
      console.log('Refund done:', event.data.refund_id);
      break;
  }

  res.json({ received: true });
});`
                },
                response: JSON.stringify({
                    id: "evt_5f9a2c3b4",
                    type: "payment.confirmed",
                    created_at: "2026-03-24T15:00:00Z",
                    data: {
                        payment_id: "pay_5f9a2c3b4",
                        status: "CONFIRMED",
                        amount: "150.00",
                        crypto_type: "SOL",
                        transaction_hash: "3xKp..."
                    }
                }, null, 2)
            },
            {
                id: 'webhook-signature',
                method: 'GET',
                path: 'Header: X-Signature',
                title: 'Signature Verification',
                description: 'The `X-Signature` header contains `t=<unix_timestamp>,v1=<hmac_hex>`. To verify a webhook delivery:\n\n1. **Extract** `t` and `v1` from the header.\n2. **Construct** the signed string by concatenating the timestamp and the raw request body: `t.<raw_request_body>`.\n3. **Compute** the HMAC-SHA256 of the signed string using your secret key.\n4. **Compare** the result with `v1` using a constant-time comparison to prevent timing attacks.\n\n> [!IMPORTANT]\n> Reject any requests where the timestamp is older than 5 minutes to prevent replay attacks.',
                request: {
                    curl: `# Verify manually:
t=1743004800
v1=abc123...

signed_string="\${t}.\${raw_body}"
computed=$(echo -n "$signed_string" | openssl dgst -sha256 -hmac "$WEBHOOK_SECRET")
# Compare computed with v1`,
                    node: `// The SDK handles this automatically:
const event = Webhooks.constructEvent(
  req.body,               // raw Buffer or string
  req.headers['x-signature'],
  process.env.WEBHOOK_SECRET
  // Optional: 4th arg = tolerance in seconds (default 300)
);`
                },
                response: JSON.stringify({
                    received: true
                }, null, 2)
            }
        ]
    }
];
