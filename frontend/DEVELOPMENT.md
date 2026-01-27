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
const { user, api_key } = await apiService.login({
  email: 'merchant@example.com',
  password: 'password123'
});

// Get profile
const profile = await apiService.getProfile();
```

### Payments
```typescript
// Create payment
const payment = await apiService.createPayment({
  amount_usd: '100.00',
  crypto_type: 'SOL',
  description: 'Order #123'
});

// List payments
const payments = await apiService.getPayments({
  limit: 10,
  status: 'PENDING'
});
```

### Wallets
```typescript
// Get wallets
const wallets = await apiService.getWallets();

// Generate wallet
const wallet = await apiService.generateWallet('ETH');

// Check gas requirements
const gasCheck = await apiService.checkGasRequirements();
```

### Security
```typescript
// Get security events
const events = await apiService.getSecurityEvents({ limit: 50 });

// Get security alerts
const alerts = await apiService.getSecurityAlerts();

// Update security settings
await apiService.updateSecuritySettings({
  enable_notifications: true,
  alert_thresholds: { low_balance: '1.0' }
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
