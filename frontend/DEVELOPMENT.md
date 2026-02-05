# FidduPay Frontend Development Guide

Quick reference for developers working with the FidduPay React frontend.

## Quick Start

```bash
cd frontend
npm install
npm run dev
```

## API Integration Examples

### Authentication
```typescript
// Login
const { user, api_key } = await apiService.auth.login({
  email: 'merchant@example.com',
  password: 'password123'
});

// Get profile
const profile = await apiService.merchant.getProfile();
```

### Payments
```typescript
// Create payment (Multi-currency checkout)
const payment = await apiService.payment.create({
  amount_usd: '100.00',
  description: 'Order #123'
});

// List payments
const payments = await apiService.payment.getHistory({
  limit: 10,
  status: 'PENDING'
});
```

### Wallets
```typescript
// Get wallets
const wallets = await apiService.wallet.getAll();

// Setup new wallet (Auto-generate)
const wallet = await apiService.wallet.setup({
  crypto_type: 'ETH',
  mode: 'generate'
});

// Check gas requirements
const gasCheck = await apiService.withdrawal.validateGas('ETH', 0.1);
```

### Security
```typescript
// Get security events
const events = await apiService.security.getEvents({ limit: 50 });

// Get security alerts
const alerts = await apiService.security.getAlerts();

// Update merchant settings (Unified)
await apiService.merchant.updateSettings({
  webhook_url: 'https://example.com/webhook',
  settlement_mode: 'managed'
});
```

## Component Usage

### Security Dashboard
```tsx
import { SecurityDashboard } from '@/components/SecurityDashboard';

<SecurityDashboard />
```

### Wallet Setup Wizard
```tsx
import { WalletSetupWizard } from '@/components/WalletSetupWizard';

<WalletSetupWizard onComplete={handleWalletSetup} />
```

### Withdrawal Interface
```tsx
import { WithdrawalInterface } from '@/components/WithdrawalInterface';

<WithdrawalInterface />
```

## State Management

```typescript
// Auth store
const { user, login, logout } = useAuthStore();

// Payment store
const { payments, createPayment, fetchPayments } = usePaymentStore();
```

## Environment Variables

```bash
VITE_API_URL=http://localhost:8080
VITE_ENVIRONMENT=development
```

## Build Commands

```bash
npm run dev      # Development server
npm run build    # Production build
npm run preview  # Preview build
```
