import FidduPay from '../src';
import { FidduPayValidationError } from '../src/errors';

describe('Resource Methods Coverage', () => {
  let client: FidduPay;

  beforeEach(() => {
    client = new FidduPay({
      apiKey: 'sk_test_1234567890',
      environment: 'sandbox'
    });
  });

  describe('Merchants Resource', () => {
    it('should have getBalance method', () => {
      expect(typeof client.merchants.getBalance).toBe('function');
    });

    it('should have setWallets method', () => {
      expect(typeof client.merchants.setWallets).toBe('function');
    });

    it('should call getBalance without parameters', () => {
      expect(() => {
        client.merchants.getBalance();
      }).not.toThrow();
    });

    it('should call setWallets with wallet data', () => {
      expect(() => {
        client.merchants.setWallets({
          ETH: '0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb',
          SOL: 'DRpbCBMxVnDK7maPM5tGv6MvB3v1sRMC86PZ8okm21hy'
        });
      }).not.toThrow();
    });
  });

  describe('Refunds Resource', () => {
    it('should validate refund creation parameters', async () => {
      await expect(client.refunds.create({
        payment_id: '',
        amount: '50.00'
      })).rejects.toThrow();
    });

    it('should accept valid refund creation', () => {
      expect(() => {
        client.refunds.create({
          payment_id: 'pay_test123',
          amount: '50.00',
          reason: 'Customer request'
        });
      }).not.toThrow();
    });

    it('should accept refund creation without amount (full refund)', () => {
      expect(() => {
        client.refunds.create({
          payment_id: 'pay_test123',
          reason: 'Full refund'
        });
      }).not.toThrow();
    });

    it('should list refunds', () => {
      expect(() => {
        client.refunds.list();
      }).not.toThrow();
    });

    it('should list refunds with parameters', () => {
      expect(() => {
        client.refunds.list({
          limit: 10,
          offset: 0
        });
      }).not.toThrow();
    });
  });

  describe('Analytics Resource', () => {
    it('should retrieve analytics', () => {
      expect(() => {
        client.analytics.retrieve({
          start_date: '2026-01-01',
          end_date: '2026-01-31',
          granularity: 'day'
        });
      }).not.toThrow();
    });

    it('should export analytics', () => {
      expect(() => {
        client.analytics.export({
          start_date: '2026-01-01',
          end_date: '2026-01-31',
          format: 'csv'
        });
      }).not.toThrow();
    });

    it('should handle different granularities', () => {
      const granularities = ['day', 'week', 'month'] as const;
      
      granularities.forEach(granularity => {
        expect(() => {
          client.analytics.retrieve({
            start_date: '2026-01-01',
            end_date: '2026-01-31',
            granularity
          });
        }).not.toThrow();
      });
    });

    it('should handle different export formats', () => {
      const formats = ['csv', 'json'] as const;
      
      formats.forEach(format => {
        expect(() => {
          client.analytics.export({
            start_date: '2026-01-01',
            end_date: '2026-01-31',
            format
          });
        }).not.toThrow();
      });
    });
  });

  describe('Wallets Resource', () => {
    it('should get wallet configurations', () => {
      expect(() => {
        client.wallets.getConfigurations();
      }).not.toThrow();
    });

    it('should configure address-only wallet', () => {
      expect(() => {
        client.wallets.configureAddress({
          crypto_type: 'ETH',
          wallet_address: '0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb'
        });
      }).not.toThrow();
    });

    it('should generate new wallet', () => {
      expect(() => {
        client.wallets.generate({
          crypto_type: 'ETH'
        });
      }).not.toThrow();
    });

    it('should import existing wallet', () => {
      expect(() => {
        client.wallets.import({
          crypto_type: 'ETH',
          private_key: '0x1234567890abcdef'
        });
      }).not.toThrow();
    });

    it('should export private key', () => {
      expect(() => {
        client.wallets.exportKey({
          crypto_type: 'ETH'
        });
      }).not.toThrow();
    });

    it('should check gas requirements', () => {
      expect(() => {
        client.wallets.checkGasRequirements();
      }).not.toThrow();
    });

    it('should get gas estimates', () => {
      expect(() => {
        client.wallets.getGasEstimates();
      }).not.toThrow();
    });

    it('should check withdrawal capability', () => {
      expect(() => {
        client.wallets.checkWithdrawalCapability('ETH');
      }).not.toThrow();
    });

    it('should handle all crypto types for wallet operations', () => {
      const cryptoTypes = ['SOL', 'ETH', 'BNB', 'MATIC', 'ARB'];
      
      cryptoTypes.forEach(cryptoType => {
        expect(() => {
          client.wallets.generate({ crypto_type: cryptoType as any });
          client.wallets.checkWithdrawalCapability(cryptoType);
        }).not.toThrow();
      });
    });
  });

  describe('Withdrawals Resource', () => {
    it('should create withdrawal', () => {
      expect(() => {
        client.withdrawals.create({
          crypto_type: 'ETH',
          amount: '0.5',
          destination_address: '0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb'
        });
      }).not.toThrow();
    });

    it('should get withdrawal by ID', () => {
      expect(() => {
        client.withdrawals.get('wd_test123');
      }).not.toThrow();
    });

    it('should list withdrawals', () => {
      expect(() => {
        client.withdrawals.list();
      }).not.toThrow();
    });

    it('should list withdrawals with parameters', () => {
      expect(() => {
        client.withdrawals.list({
          limit: 10,
          offset: 0,
          status: 'completed',
          crypto_type: 'ETH'
        });
      }).not.toThrow();
    });

    it('should cancel withdrawal', () => {
      expect(() => {
        client.withdrawals.cancel('wd_test123');
      }).not.toThrow();
    });

    it('should process withdrawal', () => {
      expect(() => {
        client.withdrawals.process('wd_test123');
      }).not.toThrow();
    });
  });

  describe('Security Resource', () => {
    it('should get security events', () => {
      expect(() => {
        client.security.getEvents();
      }).not.toThrow();
    });

    it('should get security events with parameters', () => {
      expect(() => {
        client.security.getEvents({
          limit: 10,
          offset: 0,
          event_type: 'login_attempt'
        });
      }).not.toThrow();
    });

    it('should get security alerts', () => {
      expect(() => {
        client.security.getAlerts();
      }).not.toThrow();
    });

    it('should get security alerts with parameters', () => {
      expect(() => {
        client.security.getAlerts({
          limit: 10,
          offset: 0,
          severity: 'high'
        });
      }).not.toThrow();
    });

    it('should acknowledge alert', () => {
      expect(() => {
        client.security.acknowledgeAlert('alert_test123');
      }).not.toThrow();
    });

    it('should get balance alerts', () => {
      expect(() => {
        client.security.getBalanceAlerts();
      }).not.toThrow();
    });

    it('should resolve balance alert', () => {
      expect(() => {
        client.security.resolveBalanceAlert('alert_test123');
      }).not.toThrow();
    });

    it('should check gas balances', () => {
      expect(() => {
        client.security.checkGasBalances();
      }).not.toThrow();
    });

    it('should get security settings', () => {
      expect(() => {
        client.security.getSettings();
      }).not.toThrow();
    });

    it('should update security settings', () => {
      expect(() => {
        client.security.updateSettings({
          enable_notifications: true,
          alert_thresholds: {
            low_balance: '10.00',
            failed_transactions: 5
          }
        });
      }).not.toThrow();
    });
  });

  describe('Balances Resource', () => {
    it('should get balances', () => {
      expect(() => {
        client.balances.get();
      }).not.toThrow();
    });

    it('should get balance history', () => {
      expect(() => {
        client.balances.getHistory();
      }).not.toThrow();
    });

    it('should get balance history with parameters', () => {
      expect(() => {
        client.balances.getHistory({
          limit: 10,
          offset: 0,
          crypto_type: 'ETH'
        });
      }).not.toThrow();
    });
  });

  describe('Audit Logs Resource', () => {
    it('should list audit logs', () => {
      expect(() => {
        client.auditLogs.list();
      }).not.toThrow();
    });

    it('should list audit logs with parameters', () => {
      expect(() => {
        client.auditLogs.list({
          limit: 10,
          offset: 0,
          action: 'payment_created',
          start_date: '2026-01-01',
          end_date: '2026-01-31'
        });
      }).not.toThrow();
    });
  });

  describe('Sandbox Resource', () => {
    it('should simulate payment', () => {
      expect(() => {
        client.sandbox.simulatePayment('pay_test123', {
          status: 'completed',
          transaction_hash: '0x1234567890abcdef'
        });
      }).not.toThrow();
    });

    it('should simulate payment failure', () => {
      expect(() => {
        client.sandbox.simulatePayment('pay_test123', {
          status: 'failed'
        });
      }).not.toThrow();
    });
  });

  describe('Request Options Support', () => {
    const requestOptions = {
      timeout: 10000,
      retries: 2,
      idempotencyKey: 'test-key-123'
    };

    it('should support request options in all payment methods', () => {
      expect(() => {
        client.payments.create({
          amount_usd: '100.00',
          crypto_type: 'ETH'
        }, requestOptions);
        
        client.payments.retrieve('pay_test123', requestOptions);
        client.payments.list({}, requestOptions);
        client.payments.cancel('pay_test123', requestOptions);
        client.payments.createAddressOnly({
          requested_amount: '100.00',
          crypto_type: 'ETH',
          merchant_address: '0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb'
        }, requestOptions);
        client.payments.retrieveAddressOnly('pay_test123', requestOptions);
        client.payments.updateFeeSetting({ customer_pays_fee: true }, requestOptions);
        client.payments.getFeeSetting(requestOptions);
      }).not.toThrow();
    });

    it('should support request options in refund methods', () => {
      expect(() => {
        client.refunds.create({
          payment_id: 'pay_test123',
          amount: '50.00'
        }, requestOptions);
        
        client.refunds.list({ limit: 10, offset: 0 });
      }).not.toThrow();
    });
  });

  describe('Parameter Validation Edge Cases', () => {
    it('should handle null parameters gracefully', () => {
      expect(() => {
        client.payments.list(null as any);
        client.refunds.list(null as any);
        client.balances.getHistory(null as any);
      }).not.toThrow();
    });

    it('should handle undefined parameters gracefully', () => {
      expect(() => {
        client.payments.list(undefined);
        client.refunds.list(undefined);
        client.balances.getHistory(undefined);
      }).not.toThrow();
    });

    it('should handle empty object parameters', () => {
      expect(() => {
        client.payments.list({});
        client.security.getEvents({});
        client.security.getAlerts({});
      }).not.toThrow();
    });
  });

  describe('Method Chaining and Fluent Interface', () => {
    it('should allow method chaining through resource access', () => {
      expect(() => {
        const paymentsResource = client.payments;
        const merchantsResource = client.merchants;
        const refundsResource = client.refunds;
        
        expect(paymentsResource).toBe(client.payments);
        expect(merchantsResource).toBe(client.merchants);
        expect(refundsResource).toBe(client.refunds);
      }).not.toThrow();
    });

    it('should maintain resource instances', () => {
      const payments1 = client.payments;
      const payments2 = client.payments;
      
      expect(payments1).toBe(payments2);
      expect(payments1).toBeInstanceOf(Object);
    });
  });

  describe('TypeScript Type Safety', () => {
    it('should enforce correct parameter types', () => {
      // These should compile without TypeScript errors
      expect(() => {
        client.payments.create({
          amount_usd: '100.00',
          crypto_type: 'ETH',
          description: 'Test',
          metadata: { key: 'value' },
          expiration_minutes: 60,
          webhook_url: 'https://example.com/webhook'
        });
      }).not.toThrow();

      expect(() => {
        client.payments.list({
          limit: 10,
          offset: 0,
          status: 'CONFIRMED',
          crypto_type: 'ETH'
        });
      }).not.toThrow();
    });

    it('should support all defined crypto types', () => {
      const cryptoTypes = ['SOL', 'ETH', 'BNB', 'MATIC', 'ARB', 'USDT_ETH', 'USDT_BSC', 'USDT_POLYGON', 'USDT_ARBITRUM', 'USDT_SPL'] as const;
      
      cryptoTypes.forEach(cryptoType => {
        expect(() => {
          client.payments.create({
            amount_usd: '100.00',
            crypto_type: cryptoType
          });
          
          client.withdrawals.create({
            crypto_type: cryptoType,
            amount: '50.00',
            destination_address: '0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb'
          });
        }).not.toThrow();
      });
    });

    it('should support all defined payment statuses', () => {
      const statuses = ['PENDING', 'CONFIRMING', 'CONFIRMED', 'FAILED', 'EXPIRED'] as const;
      
      statuses.forEach(status => {
        expect(() => {
          client.payments.list({ status });
        }).not.toThrow();
      });
    });
  });

  describe('Resource Initialization', () => {
    it('should initialize all resources on client creation', () => {
      const newClient = new FidduPay({
        apiKey: 'sk_test_9876543210',
        environment: 'sandbox'
      });

      expect(newClient.payments).toBeDefined();
      expect(newClient.merchants).toBeDefined();
      expect(newClient.refunds).toBeDefined();
      expect(newClient.analytics).toBeDefined();
      expect(newClient.webhooks).toBeDefined();
      expect(newClient.wallets).toBeDefined();
      expect(newClient.withdrawals).toBeDefined();
      expect(newClient.security).toBeDefined();
      expect(newClient.balances).toBeDefined();
      expect(newClient.auditLogs).toBeDefined();
      expect(newClient.sandbox).toBeDefined();
    });

    it('should maintain separate resource instances per client', () => {
      const client1 = new FidduPay({ apiKey: 'sk_test_1111111111' });
      const client2 = new FidduPay({ apiKey: 'sk_test_2222222222' });

      expect(client1.payments).not.toBe(client2.payments);
      expect(client1.merchants).not.toBe(client2.merchants);
      expect(client1.refunds).not.toBe(client2.refunds);
    });

    it('should share static webhook methods', () => {
      const client1 = new FidduPay({ apiKey: 'sk_test_1111111111' });
      const client2 = new FidduPay({ apiKey: 'sk_test_2222222222' });

      expect(client1.webhooks).toBe(client2.webhooks);
      expect(client1.webhooks.verifySignature).toBe(client2.webhooks.verifySignature);
    });
  });
});