// Fix all frontend API calls to use centralized service

// 1. RegisterPage - DONE
// 2. ContactPage - DONE  
// 3. StatusPage - DONE
// 4. PricingPage - DONE

// 5. SecurityDashboard
import { securityAPI } from '@/services/apiService';
// Replace fetch calls with: securityAPI.getEvents(), securityAPI.getAlerts(), etc.

// 6. WithdrawalInterface  
import { merchantAPI, withdrawalAPI } from '@/services/apiService';
// Replace fetch calls with: merchantAPI.getBalance(), withdrawalAPI.create(), etc.

// 7. WalletSetupWizard
import { walletAPI } from '@/services/apiService';
// Replace fetch calls with: walletAPI.getAll(), walletAPI.configure(), etc.

// 8. PaymentsPage, DashboardPage, WalletsPage, AuthContext
// Replace { apiService } from '@/services/api' with specific APIs from '@/services/apiService'

// All files need:
// - Remove all fetch() calls
// - Import specific APIs from '@/services/apiService'  
// - Use axios response format (.data instead of .json())
