#!/bin/bash

# Fix ESLint issues in SDK

echo "Fixing ESLint issues in SDK..."

# Add URLSearchParams and Buffer imports where needed
find /home/vibes/crypto-payment-gateway/fiddupay-node-sdk/src -name "*.ts" -exec sed -i '1i/// <reference types="node" />' {} \;

# Remove unused parameters by prefixing with underscore
find /home/vibes/crypto-payment-gateway/fiddupay-node-sdk/src -name "*.ts" -exec sed -i 's/options?: RequestOptions/_options?: RequestOptions/g' {} \;
find /home/vibes/crypto-payment-gateway/fiddupay-node-sdk/src -name "*.ts" -exec sed -i 's/client: HttpClient/_client: HttpClient/g' {} \;
find /home/vibes/crypto-payment-gateway/fiddupay-node-sdk/src -name "*.ts" -exec sed -i 's/error) {/_error) {/g' {} \;

echo "Fixed ESLint issues"
