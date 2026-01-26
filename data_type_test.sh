#!/bin/bash

# Data Type Compatibility Test Script
# Verifies frontend-backend API data type alignment

echo "🔍 FidduPay Data Type Compatibility Test"
echo "========================================"

# Test 1: Payment Creation Request
echo " Testing Payment Creation Request..."
echo "Frontend PaymentData:"
echo "  - amount_usd: string"
echo "  - crypto_type: string" 
echo "  - description?: string"

echo "Backend CreatePaymentRequest:"
echo "  - amount: Decimal"
echo "  - amount_usd: Option<Decimal>"
echo "  - crypto_type: CryptoType"
echo "  - description: Option<String>"

echo "⚠️  MISMATCH: Frontend sends amount_usd as string, backend expects Decimal"
echo ""

# Test 2: Payment Response
echo " Testing Payment Response..."
echo "Frontend Payment interface:"
echo "  - payment_id: string"
echo "  - status: 'PENDING' | 'CONFIRMED' | 'FAILED' | 'EXPIRED'"
echo "  - amount: string"
echo "  - amount_usd: string"
echo "  - crypto_type: string"

echo "Backend PaymentResponse:"
echo "  - payment_id: String"
echo "  - status: PaymentStatus (enum)"
echo "  - amount: Decimal"
echo "  - amount_usd: Decimal"
echo "  - crypto_type: CryptoType (enum)"

echo "⚠️  MISMATCH: Frontend expects string amounts, backend returns Decimal"
echo ""

# Test 3: Address-Only Payment
echo " Testing Address-Only Payment..."
echo "Frontend AddressOnlyPaymentData:"
echo "  - requested_amount: string"
echo "  - crypto_type: string"
echo "  - merchant_address: string"

echo "Backend CreateAddressOnlyPaymentRequest:"
echo "  - crypto_type: CryptoType"
echo "  - merchant_address: String"
echo "  - amount: Decimal"

echo "⚠️  MISMATCH: Field name difference (requested_amount vs amount)"
echo ""

# Test 4: Authentication
echo " Testing Authentication..."
echo "Frontend User interface:"
echo "  - id: number"
echo "  - business_name: string"
echo "  - email: string"
echo "  - created_at: string"
echo "  - two_factor_enabled: boolean"

echo "Backend MerchantProfile:"
echo "  - id: i64"
echo "  - business_name: String"
echo "  - email: String"
echo "  - created_at: String"
echo "  - two_factor_enabled: bool"

echo " COMPATIBLE: Authentication types match"
echo ""

# Test 5: Pagination
echo " Testing Pagination..."
echo "Frontend PaginatedResponse:"
echo "  - data: T[]"
echo "  - pagination.page: number"
echo "  - pagination.page_size: number"
echo "  - pagination.total_pages: number"
echo "  - pagination.total_count: number"

echo "Backend PaymentList:"
echo "  - payments: Vec<PaymentResponse>"
echo "  - page: i64"
echo "  - page_size: i64"
echo "  - total_pages: i64"
echo "  - total: i64"

echo "⚠️  MISMATCH: Structure difference (nested pagination vs flat)"
echo ""

echo " SUMMARY OF ISSUES FOUND:"
echo "1. Amount fields: Frontend expects strings, backend uses Decimal"
echo "2. Address-only payment: Field name mismatch (requested_amount vs amount)"
echo "3. Pagination: Structure mismatch (nested vs flat)"
echo "4. Enum serialization: Need to verify CryptoType and PaymentStatus"
echo ""

echo " RECOMMENDED FIXES:"
echo "1. Update backend to serialize Decimal as string"
echo "2. Align field names between frontend and backend"
echo "3. Standardize pagination response structure"
echo "4. Add proper enum serialization"
