# Frontend Pages API Usage Documentation

## ALL FRONTEND PAGES (43 total)

### PUBLIC PAGES (No Auth Required)
1. **LandingPage.tsx** - No API calls
2. **HomePage.tsx** - No API calls  
3. **AboutPage.tsx** - No API calls
4. **FeaturesPage.tsx** - No API calls
5. **PricingPage.tsx** -  Uses fetch('/api/v1/pricing')
6. **ContactPage.tsx** -  Uses fetch('/api/v1/contact')
7. **BlogPage.tsx** - No API calls
8. **CareersPage.tsx** - No API calls
9. **DocsPage.tsx** - No API calls
10. **StatusPage.tsx** -  Uses fetch('/api/status')
11. **SecurityPage.tsx** - No API calls
12. **CompliancePage.tsx** - No API calls
13. **TermsPage.tsx** - No API calls
14. **PrivacyPage.tsx** - No API calls
15. **CookiesPage.tsx** - No API calls

### AUTH PAGES
16. **RegisterPage.tsx** -  Uses authAPI.register()
17. **LoginPage.tsx** - No direct API calls (uses AuthContext)
18. **ForgotPasswordPage.tsx** - No API calls (simulated)

### MERCHANT DASHBOARD PAGES (Auth Required)
19. **DashboardPage.tsx** -  Uses apiService from '@/services/api'
20. **PaymentsPage.tsx** -  Uses apiService from '@/services/api'
21. **WalletsPage.tsx** -  Uses apiService from '@/services/api'

### COMPONENTS WITH API CALLS
22. **SecurityDashboard.tsx** -  Uses fetch() for security endpoints
23. **WithdrawalInterface.tsx** -  Uses fetch() for withdrawal endpoints
24. **WalletSetupWizard.tsx** -  Uses fetch() for wallet endpoints

### CONTEXTS WITH API CALLS
25. **AuthContext.tsx** -  Uses authAPI and merchantAPI

### COMPONENTS WITHOUT API CALLS (18 total)
- Layout components, UI components, etc.

## DUPLICATE API SERVICES FOUND
1. `/services/api.ts` - OLD service (9135 bytes)
2. `/services/apiService.ts` - NEW service (2692 bytes)

## PAGES NEEDING FIXES (8 total)
1. PricingPage.tsx - Replace fetch with publicAPI
2. ContactPage.tsx - Replace fetch with publicAPI  
3. StatusPage.tsx - Replace fetch with publicAPI
4. DashboardPage.tsx - Replace apiService with new APIs
5. PaymentsPage.tsx - Replace apiService with new APIs
6. WalletsPage.tsx - Replace apiService with new APIs
7. SecurityDashboard.tsx - Replace fetch with securityAPI
8. WithdrawalInterface.tsx - Replace fetch with merchantAPI/withdrawalAPI
9. WalletSetupWizard.tsx - Replace fetch with walletAPI

## FETCH USAGE FOUND IN
- SecurityDashboard.tsx (6 fetch calls)
- WithdrawalInterface.tsx (4 fetch calls)  
- WalletSetupWizard.tsx (4 fetch calls)
- StatusPage.tsx (1 fetch call)
- PricingPage.tsx (1 fetch call)
- ContactPage.tsx (1 fetch call)

TOTAL: 17 fetch calls to replace
