const fs = require('fs');
const path = require('path');

// Target paths
const completePath = path.join(__dirname, '../docs/postman/FidduPay-Complete-API.postman_collection.json');
const merchantPath = path.join(__dirname, '../fiddupay-node-sdk/postman/FidduPay-Merchant-API.postman_collection.json');

// 1. Define the base structure (based on the original 773-line version)
const baseCollection = {
    "info": {
        "name": "FidduPay Complete API v2.3.8",
        "description": "Complete API collection for FidduPay cryptocurrency payment gateway. Supports 10 cryptocurrencies across 5 major blockchains with enterprise-grade security and comprehensive admin functionality.",
        "version": "2.3.8",
        "schema": "https://schema.getpostman.com/json/collection/v2.1.0/collection.json"
    },
    "variable": [
        { "key": "baseUrl", "value": "http://localhost:8080", "type": "string", "description": "Local development server" },
        { "key": "sandboxUrl", "value": "http://localhost:3001", "type": "string", "description": "Local sandbox server" },
        { "key": "productionUrl", "value": "https://api.fiddupay.com", "type": "string", "description": "Production server" },
        { "key": "apiKey", "value": "sandbox_test_key_12345", "type": "string", "description": "Merchant API key" },
        { "key": "adminUsername", "value": "admin", "type": "string", "description": "Admin username" },
        { "key": "adminPassword", "value": "admin_password", "type": "string", "description": "Admin password" }
    ],
    "auth": {
        "type": "bearer",
        "bearer": [{ "key": "token", "value": "{{apiKey}}", "type": "string" }]
    },
    "item": []
};

// --- Helper to add folders and requests ---
function addRequest(collection, folderName, name, method, urlPath, body = null) {
    let folder = collection.item.find(i => i.name === folderName);
    if (!folder) {
        folder = { name: folderName, item: [] };
        collection.item.push(folder);
    }

    const requestItem = {
        name: name,
        request: {
            method: method,
            header: [],
            url: {
                raw: `{{baseUrl}}/${urlPath}`,
                host: ["{{baseUrl}}"],
                path: urlPath.split('/')
            }
        }
    };

    if (['POST', 'PUT'].includes(method)) {
        requestItem.request.header.push({ key: "Content-Type", value: "application/json" });
        if (body) {
            requestItem.request.body = { mode: "raw", raw: JSON.stringify(body, null, 2) };
        }
    }

    if (!urlPath.startsWith('health') && !urlPath.includes('login') && !urlPath.includes('register') && !urlPath.includes('status')) {
        requestItem.request.header.push({ key: "Authorization", value: "Bearer {{apiKey}}" });
    }

    folder.item.push(requestItem);
}

// 2. Build the COMPLETE collection
const complete = JSON.parse(JSON.stringify(baseCollection));

// --- Health & Status ---
addRequest(complete, 'Health & Status', 'Health Check', 'GET', 'health');
addRequest(complete, 'Health & Status', 'System Status', 'GET', 'api/v1/status');

// --- Admin Endpoints (Restore all from routes.rs) ---
addRequest(complete, 'Admin Authentication', 'Admin Login', 'POST', 'api/v1/admin/login', { username: "{{adminUsername}}", password: "{{adminPassword}}" });
addRequest(complete, 'Admin Authentication', 'Admin Logout', 'POST', 'api/v1/admin/logout');
addRequest(complete, 'Admin Authentication', 'Admin Dashboard', 'GET', 'api/v1/admin/dashboard');

addRequest(complete, 'Admin Merchant Management', 'Get All Merchants', 'GET', 'api/v1/admin/merchants');
addRequest(complete, 'Admin Merchant Management', 'Get Merchant Details', 'GET', 'api/v1/admin/merchants/123');
addRequest(complete, 'Admin Merchant Management', 'Suspend Merchant', 'POST', 'api/v1/admin/merchants/123/suspend', { reason: "Suspicious activity" });
addRequest(complete, 'Admin Merchant Management', 'Activate Merchant', 'POST', 'api/v1/admin/merchants/123/activate');
addRequest(complete, 'Admin Merchant Management', 'Delete Merchant', 'DELETE', 'api/v1/admin/merchants/123');

addRequest(complete, 'Admin Security Management', 'Get Security Events', 'GET', 'api/v1/admin/security/events');
addRequest(complete, 'Admin Security Management', 'Get Security Alerts', 'GET', 'api/v1/admin/security/alerts');
addRequest(complete, 'Admin Security Management', 'Acknowledge Alert', 'POST', 'api/v1/admin/security/alerts/456/acknowledge');
addRequest(complete, 'Admin Security Management', 'Get Security Settings', 'GET', 'api/v1/admin/security/settings');
addRequest(complete, 'Admin Security Management', 'Update Security Settings', 'PUT', 'api/v1/admin/security/settings', { max_login_attempts: 5, lockout_duration: 900, require_2fa: true });

addRequest(complete, 'Admin System Configuration', 'Get Environment Config', 'GET', 'api/v1/admin/config/environment');
addRequest(complete, 'Admin System Configuration', 'Update Environment Config', 'PUT', 'api/v1/admin/config/environment', { environment: "production", debug_mode: false });
addRequest(complete, 'Admin System Configuration', 'Get Fee Config', 'GET', 'api/v1/admin/config/fees');
addRequest(complete, 'Admin System Configuration', 'Update Fee Config', 'PUT', 'api/v1/admin/config/fees', { platform_fee_percentage: 2.5, withdrawal_fee_fixed: 5.0 });
addRequest(complete, 'Admin System Configuration', 'Get System Limits', 'GET', 'api/v1/admin/config/limits');
addRequest(complete, 'Admin System Configuration', 'Update System Limits', 'PUT', 'api/v1/admin/config/limits', { max_payment_amount: 10000, daily_withdrawal_limit: 50000 });

addRequest(complete, 'Admin Payment Management', 'Get All Payments', 'GET', 'api/v1/admin/payments');
addRequest(complete, 'Admin Payment Management', 'Get Payment Details', 'GET', 'api/v1/admin/payments/pay_123');
addRequest(complete, 'Admin Payment Management', 'Force Confirm Payment', 'POST', 'api/v1/admin/payments/pay_123/force-confirm');
addRequest(complete, 'Admin Payment Management', 'Force Fail Payment', 'POST', 'api/v1/admin/payments/pay_123/force-fail');

addRequest(complete, 'Admin Withdrawal Management', 'Get All Withdrawals', 'GET', 'api/v1/admin/withdrawals');
addRequest(complete, 'Admin Withdrawal Management', 'Approve Withdrawal', 'POST', 'api/v1/admin/withdrawals/wd_123/approve');
addRequest(complete, 'Admin Withdrawal Management', 'Reject Withdrawal', 'POST', 'api/v1/admin/withdrawals/wd_123/reject', { reason: "Insufficient verification" });

addRequest(complete, 'Admin Analytics', 'Get Platform Analytics', 'GET', 'api/v1/admin/analytics/platform');
addRequest(complete, 'Admin Analytics', 'Get Revenue Analytics', 'GET', 'api/v1/admin/analytics/revenue');
addRequest(complete, 'Admin Analytics', 'Transaction Reports', 'GET', 'api/v1/admin/reports/transactions');
addRequest(complete, 'Admin Analytics', 'Merchant Reports', 'GET', 'api/v1/admin/reports/merchants');

addRequest(complete, 'Admin Wallet Management', 'Get Hot Wallets', 'GET', 'api/v1/admin/wallets/hot');
addRequest(complete, 'Admin Wallet Management', 'Get Cold Wallets', 'GET', 'api/v1/admin/wallets/cold');
addRequest(complete, 'Admin Wallet Management', 'Get Wallet Balances', 'GET', 'api/v1/admin/wallets/balances');
addRequest(complete, 'Admin Wallet Management', 'Transfer Funds', 'POST', 'api/v1/admin/wallets/transfer', { from_wallet: "hot_1", to_wallet: "cold_1", amount: "10.0", crypto_type: "ETH" });

addRequest(complete, 'Admin User Management', 'Get Admin Users', 'GET', 'api/v1/admin/users');
addRequest(complete, 'Admin User Management', 'Create Admin User', 'POST', 'api/v1/admin/users', { username: "subadmin", password: "password123", role: "support" });
addRequest(complete, 'Admin User Management', 'Delete Admin User', 'DELETE', 'api/v1/admin/users/1');
addRequest(complete, 'Admin User Management', 'Update Permissions', 'PUT', 'api/v1/admin/users/1/permissions', { permissions: ["read_payments"] });

addRequest(complete, 'Admin System Maintenance', 'Get System Health', 'GET', 'api/v1/admin/system/health');
addRequest(complete, 'Admin System Maintenance', 'Get System Logs', 'GET', 'api/v1/admin/system/logs');
addRequest(complete, 'Admin System Maintenance', 'Create Backup', 'POST', 'api/v1/admin/system/backup');
addRequest(complete, 'Admin System Maintenance', 'Toggle Maintenance Mode', 'POST', 'api/v1/admin/system/maintenance', { enabled: true });

// --- Merchant Endpoints ---
addRequest(complete, 'Merchant Authentication', 'Register Merchant', 'POST', 'api/v1/merchants/register', { email: "m@e.com", business_name: "Store", password: "p123" });
addRequest(complete, 'Merchant Authentication', 'Login Merchant', 'POST', 'api/v1/merchants/login', { email: "m@e.com", password: "p123" });
addRequest(complete, 'Merchant Authentication', 'Get Profile', 'GET', 'api/v1/merchants/profile');
addRequest(complete, 'Merchant Authentication', 'Switch Environment', 'POST', 'api/v1/merchants/environment/switch');
addRequest(complete, 'Merchant Authentication', 'Rotate API Key', 'POST', 'api/v1/merchants/api-keys/rotate');

addRequest(complete, 'Merchant Payments', 'Create Payment', 'POST', 'api/v1/merchants/payments', { amount_usd: "100.0", crypto_type: "SOL", description: "Test" });
addRequest(complete, 'Merchant Payments', 'List Payments', 'GET', 'api/v1/merchants/payments');
addRequest(complete, 'Merchant Payments', 'Get Payment', 'GET', 'api/v1/merchants/payments/p_123');
addRequest(complete, 'Merchant Payments', 'Verify Payment', 'POST', 'api/v1/merchants/payments/p_123/verify');

addRequest(complete, 'Merchant Refunds', 'Create Refund', 'POST', 'api/v1/merchants/refunds', { payment_id: "p_123", amount: "10.0", reason: "Refund" });
addRequest(complete, 'Merchant Refunds', 'Get Refund', 'GET', 'api/v1/merchants/refunds/r_123');
addRequest(complete, 'Merchant Refunds', 'Complete Refund', 'POST', 'api/v1/merchants/refunds/r_123/complete');

addRequest(complete, 'Merchant Wallets', 'Get Wallets', 'GET', 'api/v1/merchants/wallets');
addRequest(complete, 'Merchant Wallets', 'Update Wallets', 'PUT', 'api/v1/merchants/wallets', { solana_address: "..." });
addRequest(complete, 'Merchant Wallets', 'Configure Address Only', 'POST', 'api/v1/merchants/wallets/configure-address', { crypto_type: "ETH", address: "0x...", customer_pays_fee: true });
addRequest(complete, 'Merchant Wallets', 'Generate Wallet', 'POST', 'api/v1/merchants/wallets/generate');
addRequest(complete, 'Merchant Wallets', 'Import Wallet', 'POST', 'api/v1/merchants/wallets/import');

addRequest(complete, 'Merchant Withdrawals', 'Create Withdrawal', 'POST', 'api/v1/merchants/withdrawals', { crypto_type: "SOL", amount: "1.0", destination_address: "..." });
addRequest(complete, 'Merchant Withdrawals', 'List Withdrawals', 'GET', 'api/v1/merchants/withdrawals');
addRequest(complete, 'Merchant Withdrawals', 'Get Withdrawal', 'GET', 'api/v1/merchants/withdrawals/w_123');
addRequest(complete, 'Merchant Withdrawals', 'Cancel Withdrawal', 'POST', 'api/v1/merchants/withdrawals/w_123/cancel');

addRequest(complete, 'Merchant Analytics', 'Get Analytics', 'GET', 'api/v1/merchants/analytics');
addRequest(complete, 'Merchant Analytics', 'Export Analytics', 'GET', 'api/v1/merchants/analytics/export');

addRequest(complete, 'Merchant Security', 'Get IP Whitelist', 'GET', 'api/v1/merchants/ip-whitelist');
addRequest(complete, 'Merchant Security', 'Set IP Whitelist', 'PUT', 'api/v1/merchants/ip-whitelist');
addRequest(complete, 'Merchant Security', 'Get Audit Logs', 'GET', 'api/v1/merchants/audit-logs');
addRequest(complete, 'Merchant Security', 'Get Profile Alerts', 'GET', 'api/v1/merchants/security/alerts');
addRequest(complete, 'Merchant Security', 'Update Settings', 'PUT', 'api/v1/merchants/security/settings');

addRequest(complete, 'Merchant Balances', 'Get Balance', 'GET', 'api/v1/merchants/balance');
addRequest(complete, 'Merchant Balances', 'Get History', 'GET', 'api/v1/merchants/balance/history');

addRequest(complete, 'Sandbox Testing', 'Enable Sandbox', 'POST', 'api/v1/merchants/sandbox/enable');
addRequest(complete, 'Sandbox Testing', 'Simulate Confirmation', 'POST', 'api/v1/merchants/sandbox/payments/pay_123/simulate', { status: "completed", transaction_hash: "0x...", from_address: "0x..." });

addRequest(complete, 'Public API', 'Supported Currencies', 'GET', 'api/v1/currencies/supported');
addRequest(complete, 'Public API', 'Blog', 'GET', 'api/v1/blog');
addRequest(complete, 'Public API', 'Careers', 'GET', 'api/v1/careers');
addRequest(complete, 'Public API', 'Payment Page Status', 'GET', 'pay/pay_123/status');

// 3. Create the MERCHANT collection by filtering the complete one
const merchant = JSON.parse(JSON.stringify(complete));
merchant.info.name = "FidduPay Merchant API SDK";
merchant.info.description = "Official Merchant API documentation for FidduPay Node.js SDK users.";
merchant.item = merchant.item.filter(folder => !folder.name.includes('Admin'));

// 4. Save both
fs.writeFileSync(completePath, JSON.stringify(complete, null, 2));
fs.writeFileSync(merchantPath, JSON.stringify(merchant, null, 2));

console.log('Postman Collections Generated:');
console.log('- Complete: ' + completePath);
console.log('- Merchant: ' + merchantPath);
