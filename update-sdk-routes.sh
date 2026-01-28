#!/bin/bash

# SDK Route Update Script
# Updates all SDK endpoints to use new /api/v1/merchant/ prefix

echo "🔄 Updating FidduPay SDK routes to v2.5.0..."

# Update analytics.ts
sed -i 's|/api/v1/analytics|/api/v1/merchant/analytics|g' fiddupay-node-sdk/src/resources/analytics.ts

# Update payments.ts  
sed -i 's|/api/v1/payments|/api/v1/merchant/payments|g' fiddupay-node-sdk/src/resources/payments.ts

# Update refunds.ts
sed -i 's|/api/v1/refunds|/api/v1/merchant/refunds|g' fiddupay-node-sdk/src/resources/refunds.ts

# Update merchants.ts (profile endpoints)
sed -i 's|/api/v1/merchant/|/api/v1/merchant/|g' fiddupay-node-sdk/src/resources/merchants.ts

# Update wallets.ts
sed -i 's|/api/v1/wallets|/api/v1/merchant/wallets|g' fiddupay-node-sdk/src/resources/wallets.ts

# Update security.ts
sed -i 's|/api/v1/security|/api/v1/merchant/security|g' fiddupay-node-sdk/src/resources/security.ts

# Update withdrawals.ts
sed -i 's|/api/v1/withdrawals|/api/v1/merchant/withdrawals|g' fiddupay-node-sdk/src/resources/withdrawals.ts

# Update balances.ts
sed -i 's|/api/v1/balance|/api/v1/merchant/balance|g' fiddupay-node-sdk/src/resources/balances.ts

# Update invoices.ts
sed -i 's|/api/v1/invoices|/api/v1/merchant/invoices|g' fiddupay-node-sdk/src/resources/invoices.ts

# Update package.json version
sed -i 's|"version": "2.4.5"|"version": "2.5.0"|g' fiddupay-node-sdk/package.json

echo "✅ SDK routes updated to v2.5.0"
echo "📦 Version bumped to 2.5.0"
echo ""
echo "Next steps:"
echo "1. cd fiddupay-node-sdk"
echo "2. npm run build"
echo "3. npm run test"
echo "4. npm publish"
echo "5. git tag v2.5.0"
echo "6. git push origin v2.5.0"
