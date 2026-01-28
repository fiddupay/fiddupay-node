#!/bin/bash

# Fix all axios response data access issues

echo "Fixing all axios response data access..."

# Fix stores - authStore
sed -i 's/response\.api_key/response.data.api_key/g' /home/vibes/crypto-payment-gateway/frontend/src/stores/authStore.ts
sed -i 's/response\.user/response.data.user/g' /home/vibes/crypto-payment-gateway/frontend/src/stores/authStore.ts
sed -i 's/set({ user: user })/set({ user: user.data })/g' /home/vibes/crypto-payment-gateway/frontend/src/stores/authStore.ts

# Fix stores - paymentStore  
sed -i 's/response\.pagination/response.data.pagination/g' /home/vibes/crypto-payment-gateway/frontend/src/stores/paymentStore.ts
sed -i 's/set({ payments: response })/set({ payments: response.data })/g' /home/vibes/crypto-payment-gateway/frontend/src/stores/paymentStore.ts

# Fix PaymentsPage
sed -i 's/payment\.payment_id/payment.data.payment_id/g' /home/vibes/crypto-payment-gateway/frontend/src/pages/PaymentsPage.tsx
sed -i 's/\[payment\]/[payment.data]/g' /home/vibes/crypto-payment-gateway/frontend/src/pages/PaymentsPage.tsx

# Fix WalletSetupWizard
sed -i 's/data\.wallet/data.data.wallet/g' /home/vibes/crypto-payment-gateway/frontend/src/components/WalletSetupWizard.tsx

# Fix WithdrawalInterface
sed -i 's/createData\.id/createData.data.id/g' /home/vibes/crypto-payment-gateway/frontend/src/components/WithdrawalInterface.tsx
sed -i 's/processData\.withdrawal/processData.data.withdrawal/g' /home/vibes/crypto-payment-gateway/frontend/src/components/WithdrawalInterface.tsx

echo "Fixed all axios response data access issues"
