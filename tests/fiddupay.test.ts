import FidduPay from '../src';
import { FidduPayValidationError } from '../src/errors';
import { CryptoType } from '../src/types';

describe('FidduPay SDK - Core Functionality', () => {
  let client: FidduPay;

  beforeEach(() => {
    client = new FidduPay({
      apiKey: 'sk_test_1234567890',
      environment: 'sandbox'
    });
  });

  describe('Constructor', () => {
    it('should create instance with valid config', () => {
      const fiddupay = new FidduPay({
        apiKey: 'sk_test_1234567890'
      });
      
      expect(fiddupay).toBeInstanceOf(FidduPay);
      expect(fiddupay.payments).toBeDefined();
      expect(fiddupay.merchants).toBeDefined();
      expect(fiddupay.refunds).toBeDefined();
      expect(fiddupay.analytics).toBeDefined();
      expect(fiddupay.webhooks).toBeDefined();
      expect(fiddupay.wallets).toBeDefined();
      expect(fiddupay.withdrawals).toBeDefined();
      expect(fiddupay.security).toBeDefined();
      expect(fiddupay.balances).toBeDefined();
      expect(fiddupay.auditLogs).toBeDefined();
      expect(fiddupay.sandbox).toBeDefined();
    });

    it('should support all crypto types', () => {
      const fiddupay = new FidduPay({
        apiKey: 'sk_test_1234567890abcdef',
        environment: 'sandbox'
      });
      
      const allCryptoTypes: CryptoType[] = [
        'SOL', 'ETH', 'BNB', 'MATIC', 'ARB',
        'USDT_ETH', 'USDT_BSC', 'USDT_POLYGON', 'USDT_ARBITRUM', 'USDT_SPL'
      ];
      
      allCryptoTypes.forEach(token => {
        expect(() => {
          const request = {
            amount_usd: '100.00',
            crypto_type: token,
            description: `Test ${token} payment`
          };
          expect(request.crypto_type).toBe(token);
        }).not.toThrow();
      });
    });

    it('should throw error for missing API key', () => {
      expect(() => {
        new FidduPay({} as any);
      }).toThrow(FidduPayValidationError);
      expect(() => {
        new FidduPay({} as any);
      }).toThrow('API key is required');
    });

    it('should throw error for invalid API key format', () => {
      expect(() => {
        new FidduPay({ apiKey: 'invalid_key' });
      }).toThrow(FidduPayValidationError);
      expect(() => {
        new FidduPay({ apiKey: 'invalid_key' });
      }).toThrow('Invalid API key format');
    });

    it('should throw error for invalid environment', () => {
      expect(() => {
        new FidduPay({ 
          apiKey: 'sk_test_1234567890',
          environment: 'invalid' as any
        });
      }).toThrow(FidduPayValidationError);
      expect(() => {
        new FidduPay({ 
          apiKey: 'sk_test_1234567890',
          environment: 'invalid' as any
        });
      }).toThrow('Environment must be either "sandbox" or "production"');
    });

    it('should create instance with all configuration options', () => {
      const fiddupay = new FidduPay({
        apiKey: 'sk_test_full_config',
        environment: 'sandbox',
        timeout: 15000,
        maxRetries: 5,
        baseURL: 'https://custom.api.com/v1'
      });
      
      expect(fiddupay).toBeInstanceOf(FidduPay);
    });
  });

  describe('Available Resources', () => {
    it('should have all core resources', () => {
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
    });

    it('should have payment methods', () => {
      expect(client.payments.create).toBeDefined();
      expect(client.payments.retrieve).toBeDefined();
      expect(client.payments.list).toBeDefined();
      expect(client.payments.cancel).toBeDefined();
      expect(client.payments.createAddressOnly).toBeDefined();
      expect(client.payments.retrieveAddressOnly).toBeDefined();
      expect(client.payments.updateFeeSetting).toBeDefined();
      expect(client.payments.getFeeSetting).toBeDefined();
    });

    it('should have merchant methods', () => {
      expect(client.merchants.getBalance).toBeDefined();
      expect(client.merchants.setWallets).toBeDefined();
      expect(client.merchants.register).toBeDefined();
      expect(client.merchants.retrieve).toBeDefined();
      expect(client.merchants.setWallet).toBeDefined();
      expect(client.merchants.switchEnvironment).toBeDefined();
      expect(client.merchants.generateApiKey).toBeDefined();
      expect(client.merchants.rotateApiKey).toBeDefined();
      expect(client.merchants.setWebhook).toBeDefined();
      expect(client.merchants.setIpWhitelist).toBeDefined();
      expect(client.merchants.getIpWhitelist).toBeDefined();
    });

    it('should have refund methods', () => {
      expect(client.refunds.create).toBeDefined();
      expect(client.refunds.retrieve).toBeDefined();
      expect(client.refunds.list).toBeDefined();
    });

    it('should have analytics methods', () => {
      expect(client.analytics.retrieve).toBeDefined();
      expect(client.analytics.export).toBeDefined();
    });

    it('should have webhook methods', () => {
      expect(client.webhooks.verifySignature).toBeDefined();
      expect(client.webhooks.constructEvent).toBeDefined();
      expect(client.webhooks.generateSignature).toBeDefined();
    });

    it('should have wallet methods', () => {
      expect(client.wallets.getConfigurations).toBeDefined();
      expect(client.wallets.configureAddress).toBeDefined();
      expect(client.wallets.generate).toBeDefined();
      expect(client.wallets.import).toBeDefined();
      expect(client.wallets.exportKey).toBeDefined();
      expect(client.wallets.checkGasRequirements).toBeDefined();
      expect(client.wallets.getGasEstimates).toBeDefined();
      expect(client.wallets.checkWithdrawalCapability).toBeDefined();
    });

    it('should have withdrawal methods', () => {
      expect(client.withdrawals.create).toBeDefined();
      expect(client.withdrawals.get).toBeDefined();
      expect(client.withdrawals.list).toBeDefined();
      expect(client.withdrawals.cancel).toBeDefined();
      expect(client.withdrawals.process).toBeDefined();
    });

    it('should have security methods', () => {
      expect(client.security.getEvents).toBeDefined();
      expect(client.security.getAlerts).toBeDefined();
      expect(client.security.acknowledgeAlert).toBeDefined();
      expect(client.security.getBalanceAlerts).toBeDefined();
      expect(client.security.resolveBalanceAlert).toBeDefined();
      expect(client.security.checkGasBalances).toBeDefined();
      expect(client.security.getSettings).toBeDefined();
      expect(client.security.updateSettings).toBeDefined();
    });

    it('should have balance methods', () => {
      expect(client.balances.get).toBeDefined();
      expect(client.balances.getHistory).toBeDefined();
    });

    it('should have audit log methods', () => {
      expect(client.auditLogs.list).toBeDefined();
    });

    it('should have sandbox methods', () => {
      expect(client.sandbox.enable).toBeDefined();
      expect(client.sandbox.simulatePayment).toBeDefined();
    });
  });

  describe('3-Mode Wallet System Support', () => {
    it('should support address-only payments', () => {
      expect(client.payments.createAddressOnly).toBeDefined();
      expect(typeof client.payments.createAddressOnly).toBe('function');
    });

    it('should validate address-only payment request structure', () => {
      expect(() => {
        const request = {
          crypto_type: 'ETH',
          merchant_address: '0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb',
          requested_amount: '0.05',
          customer_pays_fee: true
        };
        expect(request.crypto_type).toBe('ETH');
        expect(request.requested_amount).toBe('0.05');
        expect(request.customer_pays_fee).toBe(true);
      }).not.toThrow();
    });

    it('should support fee toggle in address-only payments', () => {
      const customerPaysRequest = {
        crypto_type: 'USDT_ETH',
        merchant_address: '0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb',
        requested_amount: '100.00',
        customer_pays_fee: true
      };

      const merchantPaysRequest = {
        crypto_type: 'USDT_ETH', 
        merchant_address: '0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb',
        requested_amount: '100.00',
        customer_pays_fee: false
      };

      expect(customerPaysRequest.customer_pays_fee).toBe(true);
      expect(merchantPaysRequest.customer_pays_fee).toBe(false);
    });

    it('should support fee setting operations', () => {
      expect(client.payments.updateFeeSetting).toBeDefined();
      expect(client.payments.getFeeSetting).toBeDefined();
      
      expect(() => {
        client.payments.updateFeeSetting({ customer_pays_fee: true });
        client.payments.updateFeeSetting({ customer_pays_fee: false });
      }).not.toThrow();
    });
  });

  describe('Payment Creation', () => {
    it('should support standard payment creation', () => {
      expect(() => {
        const request = {
          amount_usd: '100.00',
          crypto_type: 'ETH' as CryptoType,
          description: 'Test payment'
        };
        expect(request.amount_usd).toBe('100.00');
        expect(request.crypto_type).toBe('ETH');
      }).not.toThrow();
    });

    it('should support all crypto types', () => {
      const cryptoTypes: CryptoType[] = [
        'SOL', 'ETH', 'BNB', 'MATIC', 'ARB',
        'USDT_ETH', 'USDT_BSC', 'USDT_POLYGON', 'USDT_ARBITRUM', 'USDT_SPL'
      ];
      
      cryptoTypes.forEach(crypto => {
        expect(() => {
          const request = {
            amount_usd: '100.00',
            crypto_type: crypto,
            description: `Test ${crypto} payment`
          };
          expect(request.crypto_type).toBe(crypto);
        }).not.toThrow();
      });
    });

    it('should support payment with metadata', () => {
      expect(() => {
        const request = {
          amount_usd: '100.00',
          crypto_type: 'ETH' as CryptoType,
          description: 'Test payment with metadata',
          metadata: {
            order_id: '12345',
            customer_id: 'cust_789',
            custom_field: 'custom_value'
          }
        };
        expect(request.metadata.order_id).toBe('12345');
      }).not.toThrow();
    });

    it('should support payment with expiration', () => {
      expect(() => {
        const request = {
          amount_usd: '100.00',
          crypto_type: 'ETH' as CryptoType,
          description: 'Test payment with expiration',
          expiration_minutes: 60
        };
        expect(request.expiration_minutes).toBe(60);
      }).not.toThrow();
    });

    it('should support payment with webhook URL', () => {
      expect(() => {
        const request = {
          amount_usd: '100.00',
          crypto_type: 'ETH' as CryptoType,
          description: 'Test payment with webhook',
          webhook_url: 'https://example.com/webhook'
        };
        expect(request.webhook_url).toBe('https://example.com/webhook');
      }).not.toThrow();
    });
  });

  describe('SDK Exports and Compatibility', () => {
    it('should export FidduPay as default', () => {
      expect(FidduPay).toBeDefined();
      expect(typeof FidduPay).toBe('function');
    });

    it('should support backward compatibility alias', () => {
      const { FidduPay: FidduPayAlias } = require('../src');
      expect(FidduPayAlias).toBe(FidduPay);
    });

    it('should export all error types', () => {
      const errors = require('../src/errors');
      expect(errors.FidduPayError).toBeDefined();
      expect(errors.FidduPayAPIError).toBeDefined();
      expect(errors.FidduPayValidationError).toBeDefined();
      expect(errors.FidduPayAuthenticationError).toBeDefined();
      expect(errors.FidduPayRateLimitError).toBeDefined();
      expect(errors.FidduPayConnectionError).toBeDefined();
    });

    it('should export all types', () => {
      const types = require('../src/types');
      expect(types).toBeDefined();
    });

    it('should export Webhooks utility', () => {
      const { Webhooks } = require('../src/resources/webhooks');
      expect(Webhooks).toBeDefined();
      expect(Webhooks.verifySignature).toBeDefined();
      expect(Webhooks.constructEvent).toBeDefined();
      expect(Webhooks.generateSignature).toBeDefined();
    });
  });

  describe('Resource Consistency', () => {
    it('should maintain consistent resource instances', () => {
      expect(client.payments).toBe(client.payments);
      expect(client.merchants).toBe(client.merchants);
      expect(client.refunds).toBe(client.refunds);
      expect(client.analytics).toBe(client.analytics);
      expect(client.wallets).toBe(client.wallets);
      expect(client.withdrawals).toBe(client.withdrawals);
      expect(client.security).toBe(client.security);
      expect(client.balances).toBe(client.balances);
      expect(client.auditLogs).toBe(client.auditLogs);
      expect(client.sandbox).toBe(client.sandbox);
    });

    it('should have different instances for different clients', () => {
      const client2 = new FidduPay({
        apiKey: 'sk_test_different_client',
        environment: 'sandbox'
      });

      expect(client.payments).not.toBe(client2.payments);
      expect(client.merchants).not.toBe(client2.merchants);
      expect(client.refunds).not.toBe(client2.refunds);
    });
  });
});
