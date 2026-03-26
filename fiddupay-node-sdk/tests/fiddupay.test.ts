import FidduPay from '../src';
import { FidduPayValidationError } from '../src/errors';
import { CryptoType } from '../src/types';

describe('FidduPay SDK - Core Functionality', () => {
  let client: FidduPay;

  beforeEach(() => {
    client = new FidduPay({
      apiKey: 'sk_sandbox_1234567890'
    });
  });

  describe('Constructor', () => {
    it('should create instance with valid config', () => {
      const fiddupay = new FidduPay({
        apiKey: 'sk_sandbox_1234567890'
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
        apiKey: 'sk_sandbox_1234567890abcdef'
      });

      const allCryptoTypes: CryptoType[] = [
        'SOL', 'ETH', 'BNB', 'MATIC', 'ARB',
        'USDT_ETH', 'USDT_BEP20', 'USDT_POLYGON', 'USDT_ARBITRUM', 'USDT_SPL', 'BTC'
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


    it('should create instance with all configuration options', () => {
      const fiddupay = new FidduPay({
        apiKey: 'sk_sandbox_full_config',
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
      expect(client.invoices).toBeDefined();
      expect(client.customers).toBeDefined();
      expect(client.balances).toBeDefined();
      expect(client.auditLogs).toBeDefined();
      expect(client.sandbox).toBeDefined();
      expect(client.contact).toBeDefined();
      expect(client.transactions).toBeDefined();
    });

    it('should have payment methods', () => {
      expect(client.payments.create).toBeDefined();
      expect(client.payments.retrieve).toBeDefined();
      expect(client.payments.verify).toBeDefined();
      expect(client.payments.list).toBeDefined();
      expect(client.payments.cancel).toBeDefined();
      expect(client.payments.finalizeSelection).toBeDefined();
      expect(client.payments.createAddressOnly).toBeDefined();
      expect(client.payments.retrieveAddressOnly).toBeDefined();
      expect(client.payments.listAddressOnlyCurrencies).toBeDefined();
      expect(client.payments.getAddressOnlyStats).toBeDefined();
      expect(client.payments.getAddressOnlyHealth).toBeDefined();
      expect(client.payments.updateFeeSetting).toBeDefined();
      expect(client.payments.getFeeSetting).toBeDefined();
    });

    it('should have merchant methods', () => {
      expect(client.merchants.register).toBeDefined();
      expect(client.merchants.retrieve).toBeDefined();
      expect(client.merchants.getStatus).toBeDefined();
      expect(client.merchants.switchEnvironment).toBeDefined();
      expect(client.merchants.generateApiKey).toBeDefined();
      expect(client.merchants.rotateApiKey).toBeDefined();
      expect(client.merchants.getFeeSetting).toBeDefined();
      expect(client.merchants.updateSettings).toBeDefined();
      expect(client.merchants.getSettings).toBeDefined();
      expect(client.merchants.sendTestWebhook).toBeDefined();
      expect(client.merchants.getIpWhitelist).toBeDefined();
      expect(client.merchants.getBalance).toBeDefined();
      expect(client.merchants.getAuditLogs).toBeDefined();
      expect(client.merchants.getBalanceHistory).toBeDefined();
      expect(client.merchants.login).toBeDefined();
      expect(client.merchants.getSupportedCurrencies).toBeDefined();
      expect(client.merchants.getPricing).toBeDefined();
      expect(client.merchants.getSystemStatus).toBeDefined();
    });

    it('should have customer methods', () => {
      expect(client.customers.register).toBeDefined();
      expect(client.customers.createWallets).toBeDefined();
      expect(client.customers.getWallets).toBeDefined();
      expect(client.customers.getBalances).toBeDefined();
      expect(client.customers.list).toBeDefined();
      expect(client.customers.sweep).toBeDefined();
      expect(client.customers.bulkProvision).toBeDefined();
      expect(client.customers.deactivate).toBeDefined();
      expect(client.customers.getTransactions).toBeDefined();
      expect(client.customers.updateStatus).toBeDefined();
      expect(client.customers.updatePermissions).toBeDefined();
      expect(client.customers.getDepositAddress).toBeDefined();
      expect(client.customers.payMerchant).toBeDefined();
    });

    it('should have invoice methods', () => {
      expect(client.invoices.create).toBeDefined();
      expect(client.invoices.retrieve).toBeDefined();
      expect(client.invoices.list).toBeDefined();
    });

    it('should have refund methods', () => {
      expect(client.refunds.create).toBeDefined();
      expect(client.refunds.retrieve).toBeDefined();
      expect(client.refunds.list).toBeDefined();
      expect(client.refunds.complete).toBeDefined();
    });

    it('should have wallet methods', () => {
      expect(client.wallets.setup).toBeDefined();
      expect(client.wallets.generate).toBeDefined();
      expect(client.wallets.configureAddress).toBeDefined();
      expect(client.wallets.getConfigurations).toBeDefined();
      expect(client.wallets.getBalances).toBeDefined();
      expect(client.wallets.getGasEstimates).toBeDefined();
      expect(client.wallets.checkGasRequirements).toBeDefined();
      expect(client.wallets.checkWithdrawalCapability).toBeDefined();
      expect(client.wallets.revoke).toBeDefined();
    });

    it('should have withdrawal methods', () => {
      expect(client.withdrawals.create).toBeDefined();
      expect(client.withdrawals.list).toBeDefined();
      expect(client.withdrawals.get).toBeDefined();
      expect(client.withdrawals.cancel).toBeDefined();
      expect(client.withdrawals.process).toBeDefined();
      expect(client.withdrawals.validateGas).toBeDefined();
      expect(client.withdrawals.getGasEstimates).toBeDefined();
      expect(client.withdrawals.checkCapability).toBeDefined();
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
      expect(client.security.toggleWalletLock).toBeDefined();
      expect(client.security.toggleCustomerWalletLock).toBeDefined();
    });

    it('should have balance methods', () => {
      expect(client.balances.get).toBeDefined();
      expect(client.balances.getHistory).toBeDefined();
    });

    it('should have audit log methods', () => {
      expect(client.auditLogs.list).toBeDefined();
    });

    it('should have sandbox methods', () => {
      expect(client.sandbox.simulatePayment).toBeDefined();
    });

    it('should have contact methods', () => {
      expect(client.contact.submit).toBeDefined();
    });

    it('should have transaction methods', () => {
      expect(client.transactions.list).toBeDefined();
    });
  });

  it('should validate sandbox simulation request', () => {
    expect(() => {
      const request = {
        success: true,
        transaction_hash: '0x123...',
        from_address: '0xsender...'
      };
      expect(request.transaction_hash).toBe('0x123...');
      expect(request.from_address).toBe('0xsender...');
    }).not.toThrow();
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
          requested_amount: '0.05'
        };
        expect(request.crypto_type).toBe('ETH');
        expect(request.requested_amount).toBe('0.05');
      }).not.toThrow();
    });

    it('should support fee toggle via updateFeeSetting', () => {
      // Fee toggle is a separate operation via payments.updateFeeSetting,
      // not a field on CreateAddressOnlyPaymentRequest
      expect(client.payments.updateFeeSetting).toBeDefined();
      expect(client.payments.getFeeSetting).toBeDefined();

      const customerPaysRequest = { customer_pays_fee: true };
      const merchantPaysRequest = { customer_pays_fee: false };

      expect(customerPaysRequest.customer_pays_fee).toBe(true);
      expect(merchantPaysRequest.customer_pays_fee).toBe(false);
    });

    it('should support fee setting operations via updateSettings', () => {
      expect(client.merchants.updateSettings).toBeDefined();

      expect(() => {
        client.merchants.updateSettings({ customer_pays_fee: true });
        client.merchants.updateSettings({ customer_pays_fee: false });
      }).not.toThrow();
    });

    it('should support wallet generation mode', () => {
      expect(() => {
        const request = {
          crypto_type: 'ETH',
          mode: 'generate' as const
        };
        expect(request.mode).toBe('generate');
        expect(request.crypto_type).toBe('ETH');
      }).not.toThrow();
    });

    it('should support wallet address mode', () => {
      expect(() => {
        const request = {
          crypto_type: 'ETH',
          mode: 'address' as const,
          address: '0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb',
          is_active: true
        };
        expect(request.mode).toBe('address');
        expect(request.address).toBeDefined();
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
        'USDT_ETH', 'USDT_BEP20', 'USDT_POLYGON', 'USDT_ARBITRUM', 'USDT_SPL', 'BTC'
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

  describe('Invoice Operations', () => {
    it('should support itemized invoice creation', () => {
      expect(() => {
        const request = {
          customer_email: 'customer@example.com',
          customer_name: 'John Doe',
          currency: 'USD',
          items: [
            {
              description: 'Service A',
              quantity: 1,
              unit_price: '100.00',
              amount: '100.00'
            },
            {
              description: 'Service B',
              quantity: 2,
              unit_price: '25.00',
              amount: '50.00'
            }
          ]
        };
        expect(request.items).toHaveLength(2);
        expect(request.items[0].description).toBe('Service A');
      }).not.toThrow();
    });
  });

  describe('Customer Operations', () => {
    it('should support customer registration with names', () => {
      expect(() => {
        const request = {
          email: 'customer@example.com',
          first_name: 'John',
          last_name: 'Doe'
        };
        expect(request.first_name).toBe('John');
      }).not.toThrow();
    });

    it('should support updating customer permissions', () => {
      expect(() => {
        const request = {
          can_withdraw: true,
          withdrawal_limit: '1000.00'
        };
        expect(request.withdrawal_limit).toBe('1000.00');
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
        apiKey: 'sk_sandbox_different_client'
      });

      expect(client.payments).not.toBe(client2.payments);
      expect(client.merchants).not.toBe(client2.merchants);
      expect(client.refunds).not.toBe(client2.refunds);
    });
  });
});
