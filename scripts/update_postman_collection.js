const fs = require('fs');
const path = require('path');

const collectionPath = path.join(__dirname, '../docs/postman/FidduPay-Complete-API.postman_collection.json');
const rawData = fs.readFileSync(collectionPath);
let collection = JSON.parse(rawData);

function findFolder(name) {
    return collection.item.find(i => i.name === name);
}

function createFolder(name) {
    const folder = { name: name, item: [] };
    collection.item.push(folder);
    return folder;
}

function getOrCreateFolder(name) {
    let folder = findFolder(name);
    if (!folder) {
        folder = createFolder(name);
    }
    return folder;
}

function addRequest(folderName, name, method, urlPath, body = null) {
    const folder = getOrCreateFolder(folderName);

    // Check if request already exists to avoid duplicates
    if (folder.item.find(i => i.name === name)) {
        console.log(`Skipping ${name} in ${folderName} - already exists`);
        return;
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
        requestItem.request.header.push({
            key: "Content-Type",
            value: "application/json"
        });

        if (body) {
            requestItem.request.body = {
                mode: "raw",
                raw: JSON.stringify(body, null, 2)
            };
        }
    }

    // Add Auth header default for most
    if (!urlPath.startsWith('health') && !urlPath.includes('login') && !urlPath.includes('register') && !urlPath.includes('status')) {
        requestItem.request.header.push({
            key: "Authorization",
            value: "Bearer {{apiKey}}"
        });
    }

    folder.item.push(requestItem);
    console.log(`Added ${name} to ${folderName}`);
}

// --- Merchant Refunds ---
addRequest('Merchant Refunds', 'Create Refund', 'POST', 'api/v1/merchants/refunds', {
    payment_id: "pay_123",
    amount: "50.00",
    reason: "Customer request"
});
addRequest('Merchant Refunds', 'Get Refund Details', 'GET', 'api/v1/merchants/refunds/ref_123');
addRequest('Merchant Refunds', 'Complete Refund', 'POST', 'api/v1/merchants/refunds/ref_123/complete', {
    transaction_hash: "0x..."
});

// --- Merchant Analytics ---
addRequest('Merchant Analytics', 'Get Analytics Stats', 'GET', 'api/v1/merchants/analytics');
addRequest('Merchant Analytics', 'Export Analytics', 'GET', 'api/v1/merchants/analytics/export');

// --- Merchant Security ---
addRequest('Merchant Security', 'Set IP Whitelist', 'PUT', 'api/v1/merchants/ip-whitelist', {
    ips: ["192.168.1.1", "10.0.0.1"]
});
addRequest('Merchant Security', 'Get IP Whitelist', 'GET', 'api/v1/merchants/ip-whitelist');
addRequest('Merchant Security', 'Get Audit Logs', 'GET', 'api/v1/merchants/audit-logs');
addRequest('Merchant Security', 'Get Security Events', 'GET', 'api/v1/merchants/security/events');
addRequest('Merchant Security', 'Get Security Alerts', 'GET', 'api/v1/merchants/security/alerts');
addRequest('Merchant Security', 'Acknowledge Alert', 'POST', 'api/v1/merchants/security/alerts/alert_123/acknowledge');
addRequest('Merchant Security', 'Get Balance Alerts', 'GET', 'api/v1/merchants/security/balance-alerts');
addRequest('Merchant Security', 'Resolve Balance Alert', 'POST', 'api/v1/merchants/security/balance-alerts/alert_123/resolve');
addRequest('Merchant Security', 'Check Gas Balances', 'GET', 'api/v1/merchants/security/gas-check');

// --- Merchant Balances ---
addRequest('Merchant Balances', 'Get Current Balance', 'GET', 'api/v1/merchants/balance');
addRequest('Merchant Balances', 'Get Balance History', 'GET', 'api/v1/merchants/balance/history');

// --- Merchant Wallets (Advanced) ---
addRequest('Merchant Wallets', 'Generate Wallet', 'POST', 'api/v1/merchants/wallets/generate', { crypto_type: "ETH" });
addRequest('Merchant Wallets', 'Import Wallet', 'POST', 'api/v1/merchants/wallets/import', {
    crypto_type: "ETH",
    private_key: "0x..."
});
addRequest('Merchant Wallets', 'Export Private Key', 'POST', 'api/v1/merchants/wallets/export-key', { crypto_type: "ETH" });
addRequest('Merchant Wallets', 'Check Gas Requirements', 'GET', 'api/v1/merchants/wallets/gas-check');
addRequest('Merchant Wallets', 'Get Gas Estimates', 'GET', 'api/v1/merchants/wallets/gas-estimates');
addRequest('Merchant Wallets', 'Check Withdrawal Capability', 'GET', 'api/v1/merchants/wallets/withdrawal-capability/ETH');

// --- Admin Analytics ---
addRequest('Admin Analytics & Reporting', 'Get Platform Analytics', 'GET', 'api/v1/admin/analytics/platform');
addRequest('Admin Analytics & Reporting', 'Get Revenue Analytics', 'GET', 'api/v1/admin/analytics/revenue');
addRequest('Admin Analytics & Reporting', 'Transaction Reports', 'GET', 'api/v1/admin/reports/transactions');
addRequest('Admin Analytics & Reporting', 'Merchant Reports', 'GET', 'api/v1/admin/reports/merchants');

// --- Admin Wallets ---
addRequest('Admin Wallet Management', 'Get Hot Wallets', 'GET', 'api/v1/admin/wallets/hot');
addRequest('Admin Wallet Management', 'Get Cold Wallets', 'GET', 'api/v1/admin/wallets/cold');
addRequest('Admin Wallet Management', 'Get All Wallet Balances', 'GET', 'api/v1/admin/wallets/balances');
addRequest('Admin Wallet Management', 'Transfer Funds', 'POST', 'api/v1/admin/wallets/transfer', {
    from_wallet: "hot_1",
    to_wallet: "cold_1",
    amount: "10.0",
    crypto_type: "ETH"
});

// --- Admin User Management ---
addRequest('Admin User Management', 'Get Admin Users', 'GET', 'api/v1/admin/users');
addRequest('Admin User Management', 'Create Admin User', 'POST', 'api/v1/admin/users', {
    username: "subadmin",
    password: "password123",
    role: "support"
});
addRequest('Admin User Management', 'Delete Admin User', 'DELETE', 'api/v1/admin/users/user_123');
addRequest('Admin User Management', 'Update Permissions', 'PUT', 'api/v1/admin/users/user_123/permissions', {
    permissions: ["read_payments", "read_merchants"]
});

// --- Admin System Maintenance ---
addRequest('Admin System Maintenance', 'Get System Health', 'GET', 'api/v1/admin/system/health');
addRequest('Admin System Maintenance', 'Get System Logs', 'GET', 'api/v1/admin/system/logs');
addRequest('Admin System Maintenance', 'Create System Backup', 'POST', 'api/v1/admin/system/backup');
addRequest('Admin System Maintenance', 'Toggle Maintenance Mode', 'POST', 'api/v1/admin/system/maintenance', { enabled: true });

// --- Public Routes ---
addRequest('Public API', 'Get Blog Posts', 'GET', 'api/v1/blog');
addRequest('Public API', 'Get Careers', 'GET', 'api/v1/careers');
addRequest('Public API', 'Get Pricing', 'GET', 'api/v1/pricing');
addRequest('Public API', 'Contact Form', 'POST', 'api/v1/contact', {
    name: "John Doe",
    email: "john@example.com",
    message: "Hello"
});
addRequest('Public API', 'Supported Currencies', 'GET', 'api/v1/currencies/supported');
addRequest('Public API', 'Get Payment Page', 'GET', 'pay/link_123');
addRequest('Public API', 'Get Payment Page Status', 'GET', 'pay/link_123/status');


// Save file
fs.writeFileSync(collectionPath, JSON.stringify(collection, null, 2));
console.log('Postman collection updated successfully.');
