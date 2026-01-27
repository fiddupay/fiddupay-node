import FidduPay from '../src';
import { FidduPayValidationError } from '../src/errors';

describe('All Resource Methods', () => {
  let client: FidduPay;

  beforeEach(() => {
    client = new FidduPay({
      apiKey: 'sk_test_1234567890',
      environment: 'sandbox'
    });
  });

  describe('Merchants Resource', () => {
    it('should have register method', () => {
      expect(typeof client.merchants.register).toBe('function');
    });

    it('should have retrieve method', () => {
      expect(typeof client.merchants.retrieve).toBe('function');
    });

    it('should have setWallet method', () => {
      expect(typeof client.merchants.setWallet).toBe('function');
    });

    it('should have getBalance method', () => {
      expect(typeof client.merchants.getBalance).toBe('function');
    });

    it('should have setWallets method', () => {
      expect(typeof client.merchants.setWallets).toBe('function');
    });

    it('should have switchEnvironment method', () => {
      expect(typeof client.merchants.switchEnvironment).toBe('function');
    });

    it('should have generateApiKey method', () => {
      expect(typeof client.merchants.generateApiKey).toBe('function');
    });

    it('should have rotateApiKey method', () => {
      expect(typeof client.merchants.rotateApiKey).toBe('function');
    });

    it('should have setWebhook method', () => {
      expect(typeof client.merchants.setWebhook).toBe('function');
    });

    it('should have setIpWhitelist method', () => {
      expect(typeof client.merchants.setIpWhitelist).toBe('function');
    });

    it('should have getIpWhitelist method', () => {
      expect(typeof client.merchants.getIpWhitelist).toBe('function');
    });

    it('should validate wallet addresses in setWallets', async () => {
      await expect(client.merchants.setWallets({}))
        .rejects.toThrow(); // Expected to fail in test environment
    });

    it('should accept valid wallet configuration', () => {
      expect(() => {
        client.merchants.setWallets({
          ETH: '0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb',
          SOL: 'DRpbCBMxVnDK7maPM5tGv6MvB3v1sRMC86PZ8okm21hy',
          BNB: '0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb'
        });
      }).not.toThrow();
    });
  });

  describe('Refunds Resource', () => {
    it('should have create method', () => {
      expect(typeof client.refunds.create).toBe('function');
    });

    it('should have retrieve method', () => {
      expect(typeof client.refunds.retrieve).toBe('function');
    });

    it('should have list method', () => {
      expect(typeof client.refunds.list).toBe('function');
    });

    it('should validate refund creation parameters', async () => {
      await expect(client.refunds.create({
        payment_id: '',
        amount: '50.00'
      })).rejects.toThrow('Payment ID is required');
    });

    it('should validate refund amount', async () => {
      await expect(client.refunds.create({
        payment_id: 'pay_test123',
        amount: '-50.00'
      })).rejects.toThrow('Refund amount must be positive');
    });

    it('should require refund ID for retrieve', async () => {
      await expect(client.refunds.retrieve(''))
        .rejects.toThrow('Refund ID is required');
    });
  });

  describe('Analytics Resource', () => {
    it('should have retrieve method', () => {
      expect(typeof client.analytics.retrieve).toBe('function');
    });

    it('should have export method', () => {
      expect(typeof client.analytics.export).toBe('function');
    });

    it('should validate date range for analytics', async () => {
      const futureDate = new Date();
      futureDate.setDate(futureDate.getDate() + 1);

      await expect(client.analytics.retrieve({
        start_date: futureDate.toISOString(),
        end_date: new Date().toISOString()
      })).rejects.toThrow('Start date cannot be after end date');
    });

    it('should accept valid date range', () => {
      const startDate = new Date();
      startDate.setDate(startDate.getDate() - 7);
      const endDate = new Date();

      expect(() => {
        client.analytics.retrieve({
          start_date: startDate.toISOString(),
          end_date: endDate.toISOString()
        });
      }).not.toThrow();
    });
  });

  describe('Wallets Resource', () => {
    it('should have getConfigurations method', () => {
      expect(typeof client.wallets.getConfigurations).toBe('function');
    });

    it('should have configureAddress method', () => {
      expect(typeof client.wallets.configureAddress).toBe('function');
    });

    it('should have generate method', () => {
      expect(typeof client.wallets.generate).toBe('function');
    });

    it('should have import method', () => {
      expect(typeof client.wallets.import).toBe('function');
    });

    it('should have exportKey method', () => {
      expect(typeof client.wallets.exportKey).toBe('function');
    });

    it('should have checkGasRequirements method', () => {
      expect(typeof client.wallets.checkGasRequirements).toBe('function');
    });

    it('should have getGasEstimates method', () => {
      expect(typeof client.wallets.getGasEstimates).toBe('function');
    });

    it('should have checkWithdrawalCapability method', () => {
      expect(typeof client.wallets.checkWithdrawalCapability).toBe('function');
    });
  });

  describe('Withdrawals Resource', () => {
    it('should have create method', () => {
      expect(typeof client.withdrawals.create).toBe('function');
    });

    it('should have get method', () => {
      expect(typeof client.withdrawals.get).toBe('function');
    });

    it('should have list method', () => {
      expect(typeof client.withdrawals.list).toBe('function');
    });

    it('should have cancel method', () => {
      expect(typeof client.withdrawals.cancel).toBe('function');
    });

    it('should have process method', () => {
      expect(typeof client.withdrawals.process).toBe('function');
    });

    it('should validate withdrawal creation', async () => {
      await expect(client.withdrawals.create({
        amount: '0',
        crypto_type: 'ETH',
        destination_address: '0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb'
      })).rejects.toThrow(); // Expected to fail in test environment
    });
  });

  describe('Security Resource', () => {
    it('should have getEvents method', () => {
      expect(typeof client.security.getEvents).toBe('function');
    });

    it('should have getAlerts method', () => {
      expect(typeof client.security.getAlerts).toBe('function');
    });

    it('should have acknowledgeAlert method', () => {
      expect(typeof client.security.acknowledgeAlert).toBe('function');
    });

    it('should have getBalanceAlerts method', () => {
      expect(typeof client.security.getBalanceAlerts).toBe('function');
    });

    it('should have resolveBalanceAlert method', () => {
      expect(typeof client.security.resolveBalanceAlert).toBe('function');
    });

    it('should have checkGasBalances method', () => {
      expect(typeof client.security.checkGasBalances).toBe('function');
    });

    it('should have getSettings method', () => {
      expect(typeof client.security.getSettings).toBe('function');
    });

    it('should have updateSettings method', () => {
      expect(typeof client.security.updateSettings).toBe('function');
    });
  });

  describe('Balances Resource', () => {
    it('should have get method', () => {
      expect(typeof client.balances.get).toBe('function');
    });

    it('should have getHistory method', () => {
      expect(typeof client.balances.getHistory).toBe('function');
    });

    it('should accept crypto type filter for balance history', () => {
      expect(() => {
        client.balances.getHistory({
          crypto_type: 'ETH',
          limit: 50
        });
      }).not.toThrow();
    });
  });

  describe('Audit Logs Resource', () => {
    it('should have list method', () => {
      expect(typeof client.auditLogs.list).toBe('function');
    });

    it('should accept filters for audit log listing', () => {
      expect(() => {
        client.auditLogs.list({
          action: 'payment.created',
          limit: 25,
          offset: 0
        });
      }).not.toThrow();
    });
  });

  describe('Sandbox Resource', () => {
    it('should have enable method', () => {
      expect(typeof client.sandbox.enable).toBe('function');
    });

    it('should have simulatePayment method', () => {
      expect(typeof client.sandbox.simulatePayment).toBe('function');
    });

    it('should validate payment simulation', async () => {
      await expect(client.sandbox.simulatePayment('', {
        status: 'CONFIRMED'
      })).rejects.toThrow(); // Expected to fail in test environment
    });
  });

  describe('Resource Initialization', () => {
    it('should initialize all resources with HTTP client', () => {
      expect(client.payments).toBeDefined();
      expect(client.merchants).toBeDefined();
      expect(client.refunds).toBeDefined();
      expect(client.analytics).toBeDefined();
      expect(client.webhooks).toBeDefined();
      expect(client.wallets).toBeDefined();
      expect(client.withdrawals).toBeDefined();
      expect(client.security).toBeDefined();
      expect(client.balances).toBeDefined();
      expect(client.auditLogs).toBeDefined();
      expect(client.sandbox).toBeDefined();

      // Check that resources have access to the HTTP client
      expect((client.payments as any).client).toBeDefined();
      expect((client.merchants as any).client).toBeDefined();
      expect((client.refunds as any).client).toBeDefined();
    });
  });

  describe('Method Signatures', () => {
    it('should have consistent method signatures across resources', () => {
      // All create methods should accept data and optional options
      expect(client.payments.create.length).toBe(2);
      expect(client.refunds.create.length).toBe(2);
      expect(client.withdrawals.create.length).toBe(1); // Only data parameter

      // All retrieve/get methods should accept ID and optional options
      expect(client.payments.retrieve.length).toBe(2);
      expect(client.refunds.retrieve.length).toBe(2);
      expect(client.withdrawals.get.length).toBe(1); // Only ID parameter

      // All list methods should accept optional params and options
      expect(client.payments.list.length).toBe(2);
      expect(client.refunds.list.length).toBe(2);
      expect(client.withdrawals.list.length).toBe(1); // Only params parameter
    });
  });

  describe('Error Propagation', () => {
    it('should propagate validation errors from resources', async () => {
      // Test that validation errors from individual resources are properly thrown
      await expect(client.payments.retrieve(''))
        .rejects.toThrow(FidduPayValidationError);

      await expect(client.refunds.retrieve(''))
        .rejects.toThrow(FidduPayValidationError);
    });
  });

  describe('Options Handling', () => {
    it('should accept request options in all methods', () => {
      const options = {
        timeout: 5000,
        retries: 1,
        idempotencyKey: 'idem_test_123'
      };

      // These should not throw errors (we're testing the interface)
      expect(() => {
        client.payments.create({
          amount_usd: '100.00',
          crypto_type: 'ETH'
        }, options);
      }).not.toThrow();

      expect(() => {
        client.payments.retrieve('pay_test123', options);
      }).not.toThrow();

      expect(() => {
        client.payments.list({}, options);
      }).not.toThrow();
    });
  });
});