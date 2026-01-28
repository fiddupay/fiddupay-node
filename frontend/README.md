# FidduPay Frontend Documentation v2.2

React-based frontend for the FidduPay cryptocurrency payment gateway with comprehensive API integration and modern UI components.

## Overview

The FidduPay frontend is a React TypeScript application that provides a complete merchant dashboard for managing cryptocurrency payments, wallets, security, and analytics.

## Features

- **🔐 Authentication**: Secure login/registration with JWT tokens
- **💳 Payment Management**: Create, view, and manage cryptocurrency payments
- **🏦 Wallet Management**: 3-mode wallet system (address-only, generated, imported)
- **🔒 Security Dashboard**: Real-time monitoring, alerts, and IP whitelisting
- **💰 Balance Management**: View balances and transaction history
- **📊 Analytics**: Comprehensive reporting and data visualization
- **🔄 Withdrawal Processing**: Create and manage withdrawals
- **🧪 Sandbox Testing**: Complete testing environment
- **📋 Audit Logs**: Compliance tracking and activity monitoring

## Tech Stack

- **React 18** with TypeScript
- **Vite** for build tooling
- **Axios** for API communication
- **React Router** for navigation
- **Zustand** for state management
- **React Hook Form** with Zod validation
- **Lucide React** for icons

## Project Structure

```
frontend/
├── src/
│   ├── components/          # Reusable UI components
│   │   ├── ui/             # Base UI components
│   │   ├── layout/         # Layout components
│   │   ├── SecurityDashboard.tsx
│   │   ├── WalletSetupWizard.tsx
│   │   └── WithdrawalInterface.tsx
│   ├── pages/              # Page components
│   │   ├── auth/           # Authentication pages
│   │   ├── DashboardPage.tsx
│   │   ├── PaymentsPage.tsx
│   │   ├── WalletsPage.tsx
│   │   └── StatusPage.tsx
│   ├── services/           # API services
│   │   └── api.ts          # Main API client
│   ├── stores/             # State management
│   │   ├── authStore.ts
│   │   └── paymentStore.ts
│   ├── types/              # TypeScript definitions
│   │   └── index.ts
│   ├── contexts/           # React contexts
│   │   ├── ToastContext.tsx
│   │   └── LoadingContext.tsx
│   ├── App.tsx
│   └── main.tsx
├── public/                 # Static assets
├── package.json
└── vite.config.ts
```

## API Integration

### API Service (`src/services/api.ts`)

The main API client handles all backend communication with proper authentication and error handling.

```typescript
import { apiService } from '@/services/api';

// Authentication
const { user, api_key } = await apiService.login(credentials);
const profile = await apiService.getProfile();

// Payments - USD-based
const payment = await apiService.createPayment({
  amount_usd: "100.00",
  crypto_type: "SOL",
  description: "Order payment"
});

// Payments - Crypto-based
const cryptoPayment = await apiService.createPayment({
  amount: "2.5",
  crypto_type: "SOL", 
  description: "Order payment"
});

const payments = await apiService.getPayments({ limit: 10 });

// Wallets
const wallets = await apiService.getWallets();
const wallet = await apiService.generateWallet('SOL');

// Security
const events = await apiService.getSecurityEvents();
const alerts = await apiService.getSecurityAlerts();

// Withdrawals
const withdrawal = await apiService.createWithdrawal(data);
const withdrawals = await apiService.getWithdrawals();
```

### Supported API Endpoints

#### Authentication & Merchant Management
- `POST /api/v1/merchant/login` - User login
- `POST /api/v1/merchant/register` - User registration
- `GET /api/v1/merchant/profile` - Get merchant profile
- `POST /api/v1/merchant/environment/switch` - Switch environment
- `POST /api/v1/merchant/api-keys/generate` - Generate API key

#### Payment Management
- `POST /api/v1/payments` - Create payment (with `amount_usd` or `amount`)
- `GET /api/v1/payments` - List payments
- `GET /api/v1/payments/:id` - Get payment details
- `POST /api/v1/payments/:id/verify` - Verify payment

#### Wallet Management
- `GET /api/v1/wallets` - Get wallet configurations
- `POST /api/v1/wallets/generate` - Generate new wallet
- `POST /api/v1/wallets/import` - Import existing wallet
- `POST /api/v1/wallets/export-key` - Export private key
- `GET /api/v1/wallets/gas-check` - Check gas requirements

#### Security & Monitoring
- `GET /api/v1/security/events` - Get security events
- `GET /api/v1/security/alerts` - Get security alerts
- `POST /api/v1/security/alerts/:id/acknowledge` - Acknowledge alert
- `GET /api/v1/security/settings` - Get security settings
- `PUT /api/v1/security/settings` - Update security settings

#### Balance & Withdrawals
- `GET /api/v1/merchant/balance` - Get current balance
- `GET /api/v1/merchant/balance/history` - Get balance history
- `POST /api/v1/withdrawals` - Create withdrawal
- `GET /api/v1/withdrawals` - List withdrawals
- `POST /api/v1/withdrawals/:id/cancel` - Cancel withdrawal

#### Analytics & Reporting
- `GET /api/v1/analytics` - Get analytics data
- `GET /api/v1/analytics/export` - Export analytics
- `GET /api/v1/audit-logs` - Get audit logs

#### Sandbox & Testing
- `POST /api/v1/sandbox/enable` - Enable sandbox mode
- `POST /api/v1/sandbox/payments/:id/simulate` - Simulate payment

## Key Components

### SecurityDashboard.tsx
Comprehensive security monitoring interface with:
- Real-time security events
- Security alerts management
- IP whitelist configuration
- Security settings

### WalletSetupWizard.tsx
Step-by-step wallet configuration supporting:
- Address-only wallets
- Generated wallets
- Imported wallets
- Gas requirement checks

### WithdrawalInterface.tsx
Complete withdrawal management with:
- Withdrawal creation
- Status tracking
- Cancellation options
- Balance validation

## State Management

### Auth Store (`src/stores/authStore.ts`)
```typescript
interface AuthState {
  user: User | null;
  token: string | null;
  isAuthenticated: boolean;
  login: (credentials: LoginCredentials) => Promise<void>;
  logout: () => void;
  updateProfile: (data: Partial<User>) => void;
}
```

### Payment Store (`src/stores/paymentStore.ts`)
```typescript
interface PaymentState {
  payments: Payment[];
  currentPayment: Payment | null;
  loading: boolean;
  fetchPayments: () => Promise<void>;
  createPayment: (data: PaymentData) => Promise<Payment>;
  updatePayment: (id: string, data: Partial<Payment>) => void;
}
```

## Type Definitions

### Core Types (`src/types/index.ts`)
```typescript
export interface User {
  merchant_id: string;
  email: string;
  business_name: string;
  status: 'pending_verification' | 'verified' | 'suspended';
  created_at: string;
}

export interface Payment {
  payment_id: string;
  amount_usd: string;
  crypto_amount: string;
  crypto_type: CryptoType;
  status: PaymentStatus;
  deposit_address: string;
  transaction_hash?: string;
  created_at: string;
  expires_at: string;
}

export interface Wallet {
  crypto_type: CryptoType;
  wallet_address: string;
  has_private_key: boolean;
  can_withdraw: boolean;
  created_at: string;
}

export interface SecurityEvent {
  event_id: string;
  event_type: string;
  severity: 'low' | 'medium' | 'high' | 'critical';
  description: string;
  created_at: string;
}
```

## Environment Configuration

### Development (.env.local)
```bash
VITE_API_URL=http://localhost:8080
VITE_ENVIRONMENT=development
```

### Production (.env.production)
```bash
VITE_API_URL=https://api.fiddupay.com
VITE_ENVIRONMENT=production
```

## Development Setup

### Prerequisites
- Node.js 18+
- npm or yarn

### Installation
```bash
cd frontend
npm install
```

### Development Server
```bash
npm run dev
```

### Build for Production
```bash
npm run build
```

### Preview Production Build
```bash
npm run preview
```

## Authentication Flow

1. **Login/Register**: User provides credentials
2. **Token Storage**: JWT token stored in localStorage
3. **API Requests**: Token automatically added to Authorization header
4. **Token Refresh**: Automatic logout on 401 responses
5. **Route Protection**: Private routes require authentication

## Error Handling

### API Error Interceptor
```typescript
api.interceptors.response.use(
  (response) => response,
  (error) => {
    if (error.response?.status === 401) {
      localStorage.removeItem('fiddupay_token');
      window.location.href = '/login';
    }
    return Promise.reject(error);
  }
);
```

### Component Error Boundaries
```typescript
try {
  const result = await apiService.createPayment(data);
  // Handle success
} catch (error) {
  if (error instanceof ValidationError) {
    // Handle validation errors
  } else if (error instanceof APIError) {
    // Handle API errors
  }
}
```

## Security Features

### Authentication
- JWT token-based authentication
- Automatic token refresh
- Secure token storage
- Route-level protection

### API Security
- Request/response interceptors
- CSRF protection
- Input validation
- Error sanitization

### Data Protection
- Sensitive data masking
- Secure form handling
- XSS prevention
- Content Security Policy

## Performance Optimizations

### Code Splitting
- Route-based code splitting
- Component lazy loading
- Dynamic imports

### API Optimization
- Request caching
- Debounced API calls
- Pagination support
- Efficient data fetching

### Bundle Optimization
- Tree shaking
- Asset optimization
- Compression
- CDN integration

## Testing

### Unit Tests
```bash
npm run test
```

### E2E Tests
```bash
npm run test:e2e
```

### API Integration Tests
```bash
npm run test:api
```

## Deployment

### Vercel (Recommended)
```bash
npm run vercel-build
```

### Docker
```dockerfile
FROM node:18-alpine
WORKDIR /app
COPY package*.json ./
RUN npm ci --only=production
COPY . .
RUN npm run build
EXPOSE 3000
CMD ["npm", "run", "preview"]
```

## Browser Support

- Chrome 90+
- Firefox 88+
- Safari 14+
- Edge 90+

## Contributing

1. Follow TypeScript best practices
2. Use proper component structure
3. Implement error boundaries
4. Add proper type definitions
5. Write comprehensive tests
6. Follow accessibility guidelines

## License

Copyright © 2026 TechyTro Software. All rights reserved.
