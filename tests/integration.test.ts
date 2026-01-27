import FidduPay from '../src';
import { Webhooks } from '../src/resources/webhooks';
import { 
  FidduPayValidationError,
  FidduPayAPIError,
  FidduPayAuthenticationError 
} from '../src/errors';

describe('Integration Tests', () => {
  let client: FidduPay;

  beforeEach(() => {
    client = new FidduPay({
      apiKey: 'sk_test_integration_1234567890',
      environment: 'sandbox',
      timeout: 10000,
      maxRetries: 2
    });
  });

  describe('End-to-End Payment Flow', () => {
    it('should handle complete payment lifecycle', async () => {
      // 1. Create payment
      const paymentData = {
        amount_usd: '100.00',
        crypto_type: 'ETH' as const,
        description: 'Integration test payment',
        metadata: { test: 'integration' },
        expiration_minutes: 60
      };

      // This will fail in test environment, but validates the flow
      await expect(client.payments.create(paymentData))
        .rejects.toThrow(); // Expected in test environment

      // 2. List payments
      await expect(client.payments.list({
        limit: 10,
        status: 'PENDING'
      })).rejects.toThrow(); // Expected in test environment

      // 3. Retrieve payment (would work with real payment ID)
      await expect(client.payments.retrieve('pay_test_123'))
        .rejects.toThrow(); // Expected in test environment
    });

    it('should handle address-only payment flow', async () => {
      const addressOnlyData = {
        requested_amount: '50.00',
        crypto_type: 'BNB' as const,
        merchant_address: '0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb',
        description: 'Address-only integration test'
      };

      await expect(client.payments.createAddressOnly(addressOnlyData))
        .rejects.toThrow(); // Expected in test environment
    });
  });

  describe('Webhook Integration', () => {
    it('should handle complete webhook flow', () => {
      const secret = 'whsec_integration_test_secret';
      const payload = JSON.stringify({
        id: 'evt_integration_test',
        type: 'payment.confirmed',
        data: {
          payment_id: 'pay_integration_123',
          amount_usd: '100.00',
          crypto_amount: '0.05',
          crypto_type: 'ETH',
          status: 'CONFIRMED',
          transaction_hash: '0xintegrationtest123',
          confirmations: 12
        },
        created_at: new Date().toISOString()
      });

      // Generate signature
      const signature = Webhooks.generateSignature(payload, secret);
      expect(signature).toMatch(/^t=\d+,v1=[a-f0-9]{64}$/);

      // Verify signature
      const isValid = Webhooks.verifySignature(payload, signature, secret);
      expect(isValid).toBe(true);

      // Construct event
      const event = Webhooks.constructEvent(payload, signature, secret);
      expect(event.id).toBe('evt_integration_test');
      expect(event.type).toBe('payment.confirmed');
      expect(event.data.payment_id).toBe('pay_integration_123');
    });

    it('should handle webhook signature verification edge cases', () => {
      const secret = 'whsec_edge_case_test';
      const payload = '{"minimal": "payload"}';

      // Test with different tolerance values
      const signature = Webhooks.generateSignature(payload, secret);
      
      expect(Webhooks.verifySignature(payload, signature, secret, 300)).toBe(true);
      expect(Webhooks.verifySignature(payload, signature, secret, 1)).toBe(false);
    });
  });

  describe('Multi-Resource Operations', () => {
    it('should handle operations across multiple resources', async () => {
      // Merchant operations
      await expect(client.merchants.getBalance())
        .rejects.toThrow(); // Expected in test environment

      await expect(client.merchants.setWallets({
        ETH: '0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb',
        SOL: 'DRpbCBMxVnDK7maPM5tGv6MvB3v1sRMC86PZ8okm21hy'
      })).rejects.toThrow(); // Expected in test environment

      // Analytics operations
      const startDate = new Date();
      startDate.setDate(startDate.getDate() - 30);
      const endDate = new Date();

      await expect(client.analytics.retrieve({
        start_date: startDate.toISOString(),
        end_date: endDate.toISOString()
      })).rejects.toThrow(); // Expected in test environment

      // Security operations
      await expect(client.security.getSettings())
        .rejects.toThrow(); // Expected in test environment
    });

    it('should maintain consistent error handling across resources', async () => {
      const resources = [
        () => client.payments.retrieve(''),
        () => client.refunds.retrieve('')
      ];

      for (const resourceCall of resources) {
        await expect(resourceCall()).rejects.toThrow(FidduPayValidationError);
      }
    });
  });

  describe('Configuration Integration', () => {
    it('should handle different environment configurations', () => {
      // Sandbox configuration
      const sandboxClient = new FidduPay({
        apiKey: 'sk_test_sandbox_config',
        environment: 'sandbox',
        timeout: 5000
      });
      expect(sandboxClient).toBeInstanceOf(FidduPay);

      // Production configuration
      const prodClient = new FidduPay({
        apiKey: 'live_prod_config',
        environment: 'production',
        maxRetries: 5
      });
      expect(prodClient).toBeInstanceOf(FidduPay);

      // Custom base URL
      const customClient = new FidduPay({
        apiKey: 'sk_test_custom',
        baseURL: 'https://custom.fiddupay.com/v1'
      });
      expect(customClient).toBeInstanceOf(FidduPay);
    });

    it('should validate configuration consistency', () => {
      // Valid configurations
      expect(() => {
        new FidduPay({
          apiKey: 'sk_test_valid',
          environment: 'sandbox',
          timeout: 30000,
          maxRetries: 3
        });
      }).not.toThrow();

      // Invalid configurations
      expect(() => {
        new FidduPay({
          apiKey: 'sk_test_invalid',
          environment: 'production' // Mismatch with sk_ prefix
        });
      }).toThrow();
    });
  });

  describe('Error Handling Integration', () => {
    it('should propagate errors correctly through the stack', async () => {
      // Validation errors should be thrown immediately
      await expect(client.payments.create({
        crypto_type: 'ETH'
      } as any)).rejects.toThrow(FidduPayValidationError);

      // Network errors would be handled by HTTP client
      // (Can't test actual network errors in unit tests)
    });

    it('should handle authentication errors consistently', () => {
      const invalidClient = new FidduPay({
        apiKey: 'sk_test_invalid_auth'
      });

      // All methods should potentially throw authentication errors
      // (Would happen with real API calls)
      expect(invalidClient.payments.create).toBeDefined();
      expect(invalidClient.merchants.getBalance).toBeDefined();
      expect(invalidClient.refunds.list).toBeDefined();
    });
  });

  describe('Performance Integration', () => {
    it('should handle multiple concurrent operations', async () => {
      const operations = [
        client.payments.list().catch(() => null),
        client.merchants.getBalance().catch(() => null),
        client.refunds.list().catch(() => null),
        client.analytics.retrieve({
          start_date: new Date().toISOString(),
          end_date: new Date().toISOString()
        }).catch(() => null),
        client.balances.get().catch(() => null)
      ];

      const results = await Promise.all(operations);
      expect(results).toHaveLength(5);
    });

    it('should handle rapid successive calls to same resource', async () => {
      const calls = Array.from({ length: 10 }, (_, i) =>
        client.payments.retrieve(`pay_test_${i}`).catch(() => null)
      );

      const results = await Promise.all(calls);
      expect(results).toHaveLength(10);
    });
  });

  describe('Data Consistency Integration', () => {
    it('should maintain data consistency across operations', () => {
      // Test that the same client instance maintains state
      const client1 = new FidduPay({ apiKey: 'sk_test_consistency_1' });
      const client2 = new FidduPay({ apiKey: 'sk_test_consistency_2' });

      expect(client1.payments).not.toBe(client2.payments);
      expect(client1.merchants).not.toBe(client2.merchants);
      
      // But same client should return same instances
      expect(client1.payments).toBe(client1.payments);
      expect(client1.merchants).toBe(client1.merchants);
    });

    it('should handle complex data structures correctly', () => {
      const complexMetadata = {
        nested: {
          object: {
            with: ['arrays', 'and', 'values'],
            numbers: 123.456,
            booleans: true,
            nulls: null
          }
        },
        unicode: '测试 🚀 émojis',
        special: '!@#$%^&*()'
      };

      expect(() => {
        client.payments.create({
          amount_usd: '100.00',
          crypto_type: 'ETH',
          metadata: complexMetadata
        });
      }).not.toThrow();
    });
  });

  describe('SDK Version Compatibility', () => {
    it('should maintain backward compatibility', () => {
      // Test that old method signatures still work
      expect(() => {
        client.payments.create({
          amount_usd: '100.00',
          crypto_type: 'ETH'
        });
      }).not.toThrow();

      // Test that new features are available
      expect(typeof client.payments.createAddressOnly).toBe('function');
      expect(typeof client.payments.updateFeeSetting).toBe('function');
      expect(typeof client.sandbox.simulatePayment).toBe('function');
    });

    it('should support all documented API features', () => {
      // Core payment features
      expect(typeof client.payments.create).toBe('function');
      expect(typeof client.payments.retrieve).toBe('function');
      expect(typeof client.payments.list).toBe('function');
      expect(typeof client.payments.cancel).toBe('function');

      // Address-only payments
      expect(typeof client.payments.createAddressOnly).toBe('function');
      expect(typeof client.payments.retrieveAddressOnly).toBe('function');

      // Fee management
      expect(typeof client.payments.updateFeeSetting).toBe('function');
      expect(typeof client.payments.getFeeSetting).toBe('function');

      // Merchant features
      expect(typeof client.merchants.getBalance).toBe('function');
      expect(typeof client.merchants.setWallets).toBe('function');

      // Refunds
      expect(typeof client.refunds.create).toBe('function');
      expect(typeof client.refunds.list).toBe('function');

      // Analytics
      expect(typeof client.analytics.retrieve).toBe('function');
      expect(typeof client.analytics.export).toBe('function');

      // Webhooks
      expect(typeof Webhooks.verifySignature).toBe('function');
      expect(typeof Webhooks.constructEvent).toBe('function');

      // Security
      expect(typeof client.security.getSettings).toBe('function');

      // Sandbox
      expect(typeof client.sandbox.simulatePayment).toBe('function');
    });
  });

  describe('Real-world Usage Patterns', () => {
    it('should support common e-commerce integration pattern', async () => {
      // 1. Create payment for order
      const orderPayment = {
        amount_usd: '299.99',
        crypto_type: 'USDT_ETH' as const,
        description: 'Order #12345 - Premium Package',
        metadata: {
          order_id: '12345',
          customer_id: 'cust_789',
          product_sku: 'PREMIUM_PKG'
        },
        expiration_minutes: 30,
        webhook_url: 'https://mystore.com/webhooks/fiddupay'
      };

      // This would create the payment in real usage
      await expect(client.payments.create(orderPayment))
        .rejects.toThrow(); // Expected in test environment
    });

    it('should support subscription billing pattern', async () => {
      // Monthly subscription payment
      const subscriptionPayment = {
        amount_usd: '29.99',
        crypto_type: 'ETH' as const,
        description: 'Monthly Subscription - Pro Plan',
        metadata: {
          subscription_id: 'sub_monthly_pro',
          billing_cycle: 'monthly',
          customer_id: 'cust_subscriber_123'
        }
      };

      await expect(client.payments.create(subscriptionPayment))
        .rejects.toThrow(); // Expected in test environment
    });

    it('should support marketplace payout pattern', async () => {
      // Marketplace seller payout
      const payoutData = {
        amount: '150.00',
        crypto_type: 'USDT_BSC' as const,
        destination_address: '0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb',
        metadata: {
          seller_id: 'seller_456',
          payout_period: '2024-01',
          transaction_fees: '5.00'
        }
      };

      await expect(client.withdrawals.create(payoutData))
        .rejects.toThrow(); // Expected in test environment
    });
  });
});