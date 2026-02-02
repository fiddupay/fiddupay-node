const fs = require('fs');
const path = require('path');

const collectionPath = path.join(__dirname, '../docs/postman/FidduPay-Complete-API.postman_collection.json');
const rawData = fs.readFileSync(collectionPath);
let collection = JSON.parse(rawData);

// Filter out Admin folders
const keepFolders = [
    'Public API',
    'Merchant Authentication',
    'Merchant Payments',
    'Merchant Wallets',
    'Merchant Withdrawals',
    'Merchant Refunds',
    'Merchant Analytics',
    'Merchant Security',
    'Merchant Balances',
    'Sandbox Testing', // Keep Sandbox for merchants
    'Health & Status' // Keep Health
];

// Or filtering by exclusion
const excludeKeywords = ['Admin'];

console.log(`Original folders: ${collection.item.length}`);

collection.item = collection.item.filter(folder => {
    // Check if folder name contains "Admin"
    if (folder.name.includes('Admin')) {
        console.log(`Removing ${folder.name}`);
        return false;
    }
    return true;
});

console.log(`Remaining folders: ${collection.item.length}`);

// Rename collection to be specific
collection.info.name = "FidduPay Merchant API SDK";
collection.info.description = "Official Merchant API documentation for FidduPay Node.js SDK users. Includes Public, Merchant, and Sandbox endpoints.";

fs.writeFileSync(collectionPath, JSON.stringify(collection, null, 2));
console.log('Postman collection cleaned and updated.');
