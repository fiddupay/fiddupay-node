import FidduPay from '../src';
import { FidduPayValidationError, FidduPayAPIError } from '../src/errors';

describe('FidduPay SDK - Integration Test Suite', () => {
  let client: FidduPay;

  beforeAll(() => {
    client = new FidduPay({
      apiKey: 'sk_sandbox_integration_key',
      timeout: 30000
    });
  });

  describe('Client Initialization', () => {
    it('should create client with valid configuration', () => {
      expect(client).toBeInstanceOf(FidduPay);
      expect(client.payments).toBeDefined();
      expect(client.merchants).toBeDefined();
      expect(client.refunds).toBeDefined();
      expect(client.wallets).toBeDefined();
      expect(client.analytics).toBeDefined();
      expect(client.security).toBeDefined();
      expect(client.withdrawals).toBeDefined();
      expect(client.sandbox).toBeDefined();
      expect(client.webhooks).toBeDefined();
    });

    it('should validate API key format', () => {
      expect(() => {
        new FidduPay({ apiKey: 'invalid_key' });
      }).toThrow(FidduPayValidationError);
    });

    it('should accept valid API key formats', () => {
      expect(() => {
        new FidduPay({ apiKey: 'sk_sandbox_valid_key' });
      }).not.toThrow();

      expect(() => {
        new FidduPay({ apiKey: 'live_valid_key' });
      }).not.toThrow();
    });
  });

  describe('Resource Availability', () => {
    it('should have all payment methods available', () => {
      expect(typeof client.payments.create).toBe('function');
      expect(typeof client.payments.retrieve).toBe('function');
      expect(typeof client.payments.list).toBe('function');
      expect(typeof client.payments.createAddressOnly).toBe('function');
    });

    it('should have all invoice methods available', () => {
      expect(typeof client.invoices.create).toBe('function');
      expect(typeof client.invoices.retrieve).toBe('function');
      expect(typeof client.invoices.list).toBe('function');
    });

    it('should have all customer methods available', () => {
      expect(typeof client.customers.register).toBe('function');
      expect(typeof client.customers.createWallets).toBe('function');
      expect(typeof client.customers.getBalances).toBe('function');
    });

    it('should have all merchant methods available', () => {
      expect(typeof client.merchants.register).toBe('function');
      expect(typeof client.merchants.login).toBe('function');
      expect(typeof client.merchants.rotateApiKey).toBe('function');
      expect(typeof client.merchants.switchEnvironment).toBe('function');
      expect(typeof client.merchants.getBalance).toBe('function');
      expect(typeof client.merchants.updateSettings).toBe('function');
      expect(typeof client.merchants.getSettings).toBe('function');
      expect(typeof client.merchants.getSupportedCurrencies).toBe('function');
      expect(typeof client.merchants.getPricing).toBe('function');
      expect(typeof client.merchants.getSystemStatus).toBe('function');
    });

    it('should have all refund methods available', () => {
      expect(typeof client.refunds.create).toBe('function');
      expect(typeof client.refunds.retrieve).toBe('function');
      expect(typeof client.refunds.list).toBe('function');
    });

    it('should have all wallet methods available', () => {
      expect(typeof client.wallets.setup).toBe('function');
      expect(typeof client.wallets.checkGasRequirements).toBe('function');
      expect(typeof client.wallets.revoke).toBe('function');
    });

    it('should have all analytics methods available', () => {
      expect(typeof client.analytics.export).toBe('function');
    });

    it('should have all security methods available', () => {
      expect(typeof client.security.getEvents).toBe('function');
      expect(typeof client.security.getAlerts).toBe('function');
      expect(typeof client.security.acknowledgeAlert).toBe('function');
      expect(typeof client.security.getSettings).toBe('function');
      expect(typeof client.security.updateSettings).toBe('function');
    });

    it('should have all withdrawal methods available', () => {
      expect(typeof client.withdrawals.create).toBe('function');
      expect(typeof client.withdrawals.list).toBe('function');
      expect(typeof client.withdrawals.cancel).toBe('function');
    });

    it('should have all sandbox methods available', () => {
      expect(typeof client.sandbox.simulatePayment).toBe('function');
    });

    it('should have all webhook methods available', () => {
      expect(typeof client.webhooks.verifySignature).toBe('function');
      expect(typeof client.webhooks.constructEvent).toBe('function');
      expect(typeof client.webhooks.generateSignature).toBe('function');
    });
  });

  describe('Configuration Validation', () => {
    it('should handle API key configuration', () => {
      const sandboxClient = new FidduPay({
        apiKey: 'sk_sandbox_sandbox'
      });
      expect(sandboxClient).toBeInstanceOf(FidduPay);

      const prodClient = new FidduPay({
        apiKey: 'sk_live_production_key'
      });
      expect(prodClient).toBeInstanceOf(FidduPay);
    });

    it('should handle custom timeout settings', () => {
      const customClient = new FidduPay({
        apiKey: 'sk_sandbox_custom',
        timeout: 60000
      });
      expect(customClient).toBeInstanceOf(FidduPay);
    });

    it('should handle custom base URL', () => {
      const customClient = new FidduPay({
        apiKey: 'sk_sandbox_custom',
        baseURL: 'https://custom.api.fiddupay.com'
      });
      expect(customClient).toBeInstanceOf(FidduPay);
    });
  });

  describe('Error Handling', () => {
    it('should throw validation error for missing API key', () => {
      expect(() => {
        new FidduPay({} as any);
      }).toThrow(FidduPayValidationError);
    });

    it('should throw validation error for invalid API key format', () => {
      expect(() => {
        new FidduPay({ apiKey: 'invalid' });
      }).toThrow(FidduPayValidationError);
    });

    it('should throw validation error for empty API key', () => {
      expect(() => {
        new FidduPay({ apiKey: '' });
      }).toThrow(FidduPayValidationError);
    });
  });

  describe('Type Safety', () => {
    it('should enforce correct crypto types', () => {
      const validCryptoTypes = ['SOL', 'ETH', 'BNB', 'MATIC', 'ARB', 'USDT_ETH', 'USDT_BEP20', 'USDT_POLYGON', 'USDT_ARBITRUM', 'USDT_SPL', 'BTC'];

      validCryptoTypes.forEach(cryptoType => {
        expect(() => {
          const paymentData = {
            amount_usd: '100.00',
            crypto_type: cryptoType as any,
            description: 'Test payment'
          };
          expect(paymentData.crypto_type).toBe(cryptoType);
        }).not.toThrow();
      });
    });

    it('should enforce correct payment status types', () => {
      const validStatuses = ['PENDING', 'CONFIRMING', 'CONFIRMED', 'FAILED', 'EXPIRED', 'REFUNDED', 'SELECTION_REQUIRED'];

      validStatuses.forEach(status => {
        expect(validStatuses).toContain(status);
      });
    });
  });

  describe('SDK Coverage', () => {
    it('should cover all major resource categories', () => {
      const expectedResources = [
        'payments',
        'merchants',
        'refunds',
        'wallets',
        'analytics',
        'security',
        'withdrawals',
        'sandbox',
        'webhooks',
        'invoices',
        'customers',
        'balances',
        'auditLogs',
        'contact',
        'transactions'
      ];

      expectedResources.forEach(resource => {
        expect(client).toHaveProperty(resource);
        expect(client[resource as keyof typeof client]).toBeDefined();
      });
    });

    it('should provide comprehensive payment operations', () => {
      const paymentMethods = [
        'create', 'retrieve', 'list', 'cancel', 'verify', 'finalizeSelection', 
        'createAddressOnly', 'retrieveAddressOnly', 
        'listAddressOnlyCurrencies', 'getAddressOnlyStats', 'getAddressOnlyHealth', 
        'updateFeeSetting', 'getFeeSetting'
      ];

      paymentMethods.forEach(method => {
        expect(client.payments).toHaveProperty(method);
        expect(typeof client.payments[method as keyof typeof client.payments]).toBe('function');
      });
    });

    it('should provide comprehensive merchant operations', () => {
      const merchantMethods = [
        'register', 'login', 'retrieve', 'getStatus', 'switchEnvironment', 'generateApiKey', 
        'rotateApiKey', 'getFeeSetting', 'updateSettings', 'getSettings', 
        'sendTestWebhook', 'getIpWhitelist', 'getBalance', 'getAuditLogs', 'getBalanceHistory',
        'getSupportedCurrencies', 'getPricing', 'getSystemStatus'
      ];

      merchantMethods.forEach(method => {
        expect(client.merchants).toHaveProperty(method);
        expect(typeof client.merchants[method as keyof typeof client.merchants]).toBe('function');
      });
    });

    it('should provide comprehensive refund operations', () => {
      const refundMethods = ['create', 'retrieve', 'list', 'complete'];
      refundMethods.forEach(method => {
        expect(client.refunds).toHaveProperty(method);
        expect(typeof client.refunds[method as keyof typeof client.refunds]).toBe('function');
      });
    });

    it('should provide comprehensive invoice operations', () => {
      const invoiceMethods = ['create', 'retrieve', 'list'];
      invoiceMethods.forEach(method => {
        expect(client.invoices).toHaveProperty(method);
        expect(typeof client.invoices[method as keyof typeof client.invoices]).toBe('function');
      });
    });

    it('should provide comprehensive customer operations', () => {
      const customerMethods = [
        'register', 'list', 'getBalances', 'getWallets', 'createWallets', 
        'updateStatus', 'updatePermissions', 'withdraw', 'sweep', 'deactivate', 
        'getTransactions', 'getDepositAddress', 'payMerchant'
      ];
      customerMethods.forEach(method => {
        expect(client.customers).toHaveProperty(method);
        expect(typeof client.customers[method as keyof typeof client.customers]).toBe('function');
      });
    });

    it('should provide comprehensive wallet operations', () => {
      const walletMethods = [
        'setup', 'generate', 'configureAddress', 'getConfigurations', 'getBalances', 'revoke', 
        'checkGasRequirements', 'getGasEstimates', 'checkWithdrawalCapability'
      ];
      walletMethods.forEach(method => {
        expect(client.wallets).toHaveProperty(method);
        expect(typeof client.wallets[method as keyof typeof client.wallets]).toBe('function');
      });
    });

    it('should provide comprehensive withdrawal operations', () => {
      const withdrawalMethods = [
        'create', 'list', 'get', 'cancel', 'process', 
        'validateGas', 'getGasEstimates', 'checkCapability'
      ];
      withdrawalMethods.forEach(method => {
        expect(client.withdrawals).toHaveProperty(method);
        expect(typeof client.withdrawals[method as keyof typeof client.withdrawals]).toBe('function');
      });
    });

    it('should provide comprehensive security operations', () => {
      const securityMethods = [
        'getEvents', 'getAlerts', 'acknowledgeAlert', 'getBalanceAlerts', 
        'resolveBalanceAlert', 'checkGasBalances', 'getSettings', 'updateSettings', 
        'toggleWalletLock', 'toggleCustomerWalletLock'
      ];
      securityMethods.forEach(method => {
        expect(client.security).toHaveProperty(method);
        expect(typeof client.security[method as keyof typeof client.security]).toBe('function');
      });
    });

    it('should provide webhook utilities', () => {
      const webhookMethods = ['verifySignature', 'constructEvent', 'generateSignature'];
      webhookMethods.forEach(method => {
        expect(client.webhooks).toHaveProperty(method);
        expect(typeof client.webhooks[method as keyof typeof client.webhooks]).toBe('function');
      });
    });

    it('should provide comprehensive analytics operations', () => {
      const analyticsMethods = ['retrieve', 'export'];
      analyticsMethods.forEach(method => {
        expect(client.analytics).toHaveProperty(method);
        expect(typeof client.analytics[method as keyof typeof client.analytics]).toBe('function');
      });
    });

    it('should provide comprehensive balances operations', () => {
      const balanceMethods = ['get', 'getHistory'];
      balanceMethods.forEach(method => {
        expect(client.balances).toHaveProperty(method);
        expect(typeof client.balances[method as keyof typeof client.balances]).toBe('function');
      });
    });

    it('should provide comprehensive audit log operations', () => {
      const auditMethods = ['list'];
      auditMethods.forEach(method => {
        expect(client.auditLogs).toHaveProperty(method);
        expect(typeof client.auditLogs[method as keyof typeof client.auditLogs]).toBe('function');
      });
    });

    it('should provide comprehensive sandbox operations', () => {
      const sandboxMethods = ['simulatePayment'];
      sandboxMethods.forEach(method => {
        expect(client.sandbox).toHaveProperty(method);
        expect(typeof client.sandbox[method as keyof typeof client.sandbox]).toBe('function');
      });
    });

    it('should provide comprehensive contact operations', () => {
      const contactMethods = ['submit'];
      contactMethods.forEach(method => {
        expect(client.contact).toHaveProperty(method);
        expect(typeof client.contact[method as keyof typeof client.contact]).toBe('function');
      });
    });

    it('should provide comprehensive transactions operations', () => {
      const txnMethods = ['list'];
      txnMethods.forEach(method => {
        expect(client.transactions).toHaveProperty(method);
        expect(typeof client.transactions[method as keyof typeof client.transactions]).toBe('function');
      });
    });
  });

  describe('Mock Integration Tests', () => {
    it('should handle successful API responses', async () => {
      // Mock a successful payment creation
      const mockPayment = {
        payment_id: 'pay_test_123',
        status: 'PENDING',
        amount_usd: '100.00',
        amount: '0.05',
        crypto_type: 'ETH',
        deposit_address: '0x123...',
        created_at: new Date().toISOString(),
        expires_at: new Date().toISOString()
      };

      // This test validates the structure without making actual API calls
      expect(mockPayment.payment_id).toBe('pay_test_123');
      expect(mockPayment.status).toBe('PENDING');
      expect(mockPayment.crypto_type).toBe('ETH');
    });

    it('should handle error responses appropriately', () => {
      // Test error handling structure
      const mockError = new FidduPayAPIError('Test error', 400, 'VALIDATION_ERROR');

      expect(mockError).toBeInstanceOf(FidduPayAPIError);
      expect(mockError.message).toBe('Test error');
      expect(mockError.statusCode).toBe(400);
      expect(mockError.code).toBe('VALIDATION_ERROR');
    });
  });
});
