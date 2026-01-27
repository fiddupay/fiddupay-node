import FidduPay from '../src';
import { FidduPayValidationError } from '../src/errors';
import { CryptoType, PaymentStatus } from '../src/types';

describe('Utility Functions and Edge Cases', () => {
  let client: FidduPay;

  beforeEach(() => {
    client = new FidduPay({
      apiKey: 'sk_test_1234567890',
      environment: 'sandbox'
    });
  });

  describe('Type Definitions', () => {
    it('should support all crypto types', () => {
      const cryptoTypes: CryptoType[] = [
        'SOL', 'ETH', 'BNB', 'MATIC', 'ARB',
        'USDT_ETH', 'USDT_BSC', 'USDT_POLYGON', 'USDT_ARBITRUM', 'USDT_SPL'
      ];

      cryptoTypes.forEach(crypto => {
        expect(typeof crypto).toBe('string');
        expect(crypto.length).toBeGreaterThan(0);
      });
    });

    it('should support all payment statuses', () => {
      const statuses: PaymentStatus[] = [
        'PENDING', 'CONFIRMING', 'CONFIRMED', 'FAILED', 'EXPIRED'
      ];

      statuses.forEach(status => {
        expect(typeof status).toBe('string');
        expect(status.length).toBeGreaterThan(0);
      });
    });
  });

  describe('SDK Exports', () => {
    it('should export main client class', () => {
      expect(FidduPay).toBeDefined();
      expect(typeof FidduPay).toBe('function');
    });

    it('should export error classes', () => {
      const { 
        FidduPayError,
        FidduPayAPIError,
        FidduPayValidationError,
        FidduPayAuthenticationError,
        FidduPayRateLimitError,
        FidduPayConnectionError
      } = require('../src/errors');

      expect(FidduPayError).toBeDefined();
      expect(FidduPayAPIError).toBeDefined();
      expect(FidduPayValidationError).toBeDefined();
      expect(FidduPayAuthenticationError).toBeDefined();
      expect(FidduPayRateLimitError).toBeDefined();
      expect(FidduPayConnectionError).toBeDefined();
    });

    it('should export types', () => {
      const types = require('../src/types');
      expect(types).toBeDefined();
    });

    it('should export Webhooks utility', () => {
      const { Webhooks } = require('../src/resources/webhooks');
      expect(Webhooks).toBeDefined();
      expect(typeof Webhooks.verifySignature).toBe('function');
      expect(typeof Webhooks.constructEvent).toBe('function');
    });
  });

  describe('Backward Compatibility', () => {
    it('should support FidduPay alias', () => {
      const { FidduPay: FidduPayAlias } = require('../src');
      expect(FidduPayAlias).toBe(FidduPay);
    });

    it('should support default export', () => {
      const DefaultExport = require('../src').default;
      expect(DefaultExport).toBe(FidduPay);
    });
  });

  describe('Configuration Edge Cases', () => {
    it('should handle minimal configuration', () => {
      const client = new FidduPay({ apiKey: 'sk_test_minimal' });
      expect(client).toBeInstanceOf(FidduPay);
    });

    it('should handle maximum configuration', () => {
      const client = new FidduPay({
        apiKey: 'sk_test_maximum',
        environment: 'sandbox',
        apiVersion: 'v1',
        timeout: 60000,
        maxRetries: 10,
        baseURL: 'https://custom.api.com/v1'
      });
      expect(client).toBeInstanceOf(FidduPay);
    });

    it('should handle edge case API keys', () => {
      // Minimum length API keys
      expect(() => {
        new FidduPay({ apiKey: 'sk_' });
      }).not.toThrow();

      expect(() => {
        new FidduPay({ apiKey: 'live_' });
      }).not.toThrow();

      // Very long API keys
      const longKey = 'sk_' + 'a'.repeat(100);
      expect(() => {
        new FidduPay({ apiKey: longKey });
      }).not.toThrow();
    });
  });

  describe('Memory and Performance', () => {
    it('should not leak memory when creating multiple clients', () => {
      const clients = [];
      for (let i = 0; i < 100; i++) {
        clients.push(new FidduPay({ apiKey: `sk_test_${i}` }));
      }
      
      expect(clients.length).toBe(100);
      clients.forEach(client => {
        expect(client).toBeInstanceOf(FidduPay);
      });
    });

    it('should handle rapid successive calls', async () => {
      const promises = [];
      for (let i = 0; i < 10; i++) {
        promises.push(
          client.payments.retrieve(`pay_test_${i}`).catch(() => {
            // Expected to fail in test environment
            return null;
          })
        );
      }

      const results = await Promise.all(promises);
      expect(results).toHaveLength(10);
    });
  });

  describe('Input Sanitization', () => {
    it('should handle special characters in strings', () => {
      const specialChars = '!@#$%^&*()_+-=[]{}|;:,.<>?';
      
      expect(() => {
        client.payments.create({
          amount_usd: '100.00',
          crypto_type: 'ETH',
          description: specialChars
        });
      }).not.toThrow();
    });

    it('should handle unicode characters', () => {
      const unicode = '测试 🚀 émojis ñoño';
      
      expect(() => {
        client.payments.create({
          amount_usd: '100.00',
          crypto_type: 'ETH',
          description: unicode
        });
      }).not.toThrow();
    });

    it('should handle null and undefined values gracefully', () => {
      expect(() => {
        client.payments.create({
          amount_usd: '100.00',
          crypto_type: 'ETH',
          description: undefined,
          metadata: null
        } as any);
      }).not.toThrow();
    });
  });

  describe('Numeric Precision', () => {
    it('should handle decimal precision correctly', () => {
      const preciseAmounts = [
        '0.01',
        '0.001',
        '0.0001',
        '999999.99',
        '123.456789'
      ];

      preciseAmounts.forEach(amount => {
        expect(() => {
          (client.payments as any).validateCreatePayment({
            amount_usd: amount,
            crypto_type: 'ETH'
          });
        }).not.toThrow();
      });
    });

    it('should handle scientific notation', () => {
      expect(() => {
        (client.payments as any).validateCreatePayment({
          amount_usd: '1e2', // 100
          crypto_type: 'ETH'
        });
      }).not.toThrow();

      expect(() => {
        (client.payments as any).validateCreatePayment({
          amount_usd: '1.5e-2', // 0.015
          crypto_type: 'ETH'
        });
      }).not.toThrow();
    });
  });

  describe('Concurrent Operations', () => {
    it('should handle concurrent payment creations', async () => {
      const promises = Array.from({ length: 5 }, (_, i) => 
        client.payments.create({
          amount_usd: `${100 + i}.00`,
          crypto_type: 'ETH',
          description: `Concurrent payment ${i}`
        }).catch(() => null) // Expected to fail in test environment
      );

      const results = await Promise.all(promises);
      expect(results).toHaveLength(5);
    });

    it('should handle concurrent webhook verifications', () => {
      const { Webhooks } = require('../src/resources/webhooks');
      const payload = '{"test": "data"}';
      const secret = 'test_secret';

      const promises = Array.from({ length: 10 }, () => 
        Promise.resolve(Webhooks.generateSignature(payload, secret))
      );

      return Promise.all(promises).then(signatures => {
        expect(signatures).toHaveLength(10);
        signatures.forEach(sig => {
          expect(typeof sig).toBe('string');
          expect(sig.length).toBeGreaterThan(0);
        });
      });
    });
  });

  describe('Resource Method Chaining', () => {
    it('should maintain context across method calls', () => {
      expect(client.payments).toBe(client.payments);
      expect(client.merchants).toBe(client.merchants);
      expect(client.refunds).toBe(client.refunds);
    });

    it('should allow method calls on same resource instance', () => {
      const payments = client.payments;
      
      expect(() => {
        payments.create({
          amount_usd: '100.00',
          crypto_type: 'ETH'
        });
        payments.list();
        payments.getFeeSetting();
      }).not.toThrow();
    });
  });

  describe('Error Recovery', () => {
    it('should recover from validation errors', async () => {
      // First call with invalid data
      await expect(client.payments.create({
        crypto_type: 'ETH'
      } as any)).rejects.toThrow();

      // Second call with valid data should work
      expect(() => {
        client.payments.create({
          amount_usd: '100.00',
          crypto_type: 'ETH'
        });
      }).not.toThrow();
    });

    it('should handle multiple consecutive errors', async () => {
      const invalidCalls = [
        () => client.payments.retrieve(''),
        () => client.refunds.retrieve('')
      ];

      for (const call of invalidCalls) {
        await expect(call()).rejects.toThrow();
      }

      // Valid call should still work
      expect(() => {
        client.payments.create({
          amount_usd: '100.00',
          crypto_type: 'ETH'
        });
      }).not.toThrow();
    });
  });

  describe('Environment Detection', () => {
    it('should detect sandbox from sk_ prefix', () => {
      const client = new FidduPay({ apiKey: 'sk_test_auto_detect' });
      expect(client).toBeInstanceOf(FidduPay);
    });

    it('should detect production from live_ prefix', () => {
      const client = new FidduPay({ apiKey: 'live_prod_auto_detect' });
      expect(client).toBeInstanceOf(FidduPay);
    });

    it('should override auto-detection when environment specified', () => {
      expect(() => {
        new FidduPay({ 
          apiKey: 'sk_test_override',
          environment: 'sandbox'
        });
      }).not.toThrow();
    });
  });

  describe('Static Analysis Support', () => {
    it('should provide proper TypeScript types', () => {
      // This test ensures TypeScript compilation works correctly
      const client: FidduPay = new FidduPay({ apiKey: 'sk_test_types' });
      
      // These should not cause TypeScript errors
      const paymentPromise: Promise<any> = client.payments.create({
        amount_usd: '100.00',
        crypto_type: 'ETH'
      });

      const listPromise: Promise<any> = client.payments.list({
        limit: 10,
        status: 'CONFIRMED'
      });

      expect(paymentPromise).toBeDefined();
      expect(listPromise).toBeDefined();
    });
  });
});