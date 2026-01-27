import FidduPay from '../src';
import { FidduPayValidationError } from '../src/errors';

describe('API Format Validation', () => {
  let client: FidduPay;

  beforeEach(() => {
    client = new FidduPay({
      apiKey: 'sk_test_1234567890',
      environment: 'sandbox'
    });
  });

  describe('Payment Creation - Amount Format Validation', () => {
    it('should accept amount_usd only', () => {
      expect(() => {
        client.payments.create({
          amount_usd: '100.00',
          crypto_type: 'ETH',
          description: 'Test payment with amount_usd'
        });
      }).not.toThrow();
    });

    it('should accept amount only', () => {
      expect(() => {
        client.payments.create({
          amount: '0.05',
          crypto_type: 'ETH',
          description: 'Test payment with amount'
        });
      }).not.toThrow();
    });

    it('should reject both amount and amount_usd', async () => {
      await expect(client.payments.create({
        amount: '0.05',
        amount_usd: '100.00',
        crypto_type: 'ETH',
        description: 'Test payment with both amounts'
      })).rejects.toThrow('Provide either amount or amount_usd, not both');
    });

    it('should reject neither amount nor amount_usd', async () => {
      await expect(client.payments.create({
        crypto_type: 'ETH',
        description: 'Test payment with no amount'
      } as any)).rejects.toThrow('Either amount or amount_usd must be provided');
    });

    it('should validate amount_usd format', async () => {
      await expect(client.payments.create({
        amount_usd: 'invalid',
        crypto_type: 'ETH'
      })).rejects.toThrow('Amount must be a positive number');
    });

    it('should validate amount format', async () => {
      await expect(client.payments.create({
        amount: 'invalid',
        crypto_type: 'ETH'
      })).rejects.toThrow('Amount must be a positive number');
    });

    it('should validate minimum amount_usd', async () => {
      await expect(client.payments.create({
        amount_usd: '0.005',
        crypto_type: 'ETH'
      })).rejects.toThrow('Minimum amount is $0.01');
    });

    it('should validate minimum amount', async () => {
      await expect(client.payments.create({
        amount: '0.005',
        crypto_type: 'ETH'
      })).rejects.toThrow('Minimum amount is $0.01');
    });

    it('should validate maximum amount_usd', async () => {
      await expect(client.payments.create({
        amount_usd: '2000000',
        crypto_type: 'ETH'
      })).rejects.toThrow('Maximum amount is $1,000,000');
    });

    it('should validate maximum amount', async () => {
      await expect(client.payments.create({
        amount: '2000000',
        crypto_type: 'ETH'
      })).rejects.toThrow('Maximum amount is $1,000,000');
    });

    it('should accept valid decimal amounts', () => {
      expect(() => {
        client.payments.create({
          amount_usd: '99.99',
          crypto_type: 'ETH'
        });
      }).not.toThrow();

      expect(() => {
        client.payments.create({
          amount: '0.123456',
          crypto_type: 'ETH'
        });
      }).not.toThrow();
    });

    it('should accept string numbers', () => {
      expect(() => {
        client.payments.create({
          amount_usd: '100',
          crypto_type: 'ETH'
        });
      }).not.toThrow();

      expect(() => {
        client.payments.create({
          amount: '1.0',
          crypto_type: 'ETH'
        });
      }).not.toThrow();
    });
  });

  describe('Crypto Type Validation', () => {
    const validCryptoTypes = ['SOL', 'ETH', 'BNB', 'MATIC', 'ARB', 'USDT_ETH', 'USDT_BSC', 'USDT_POLYGON', 'USDT_ARBITRUM', 'USDT_SPL'];
    
    it('should accept all valid crypto types', () => {
      validCryptoTypes.forEach(cryptoType => {
        expect(() => {
          client.payments.create({
            amount_usd: '100.00',
            crypto_type: cryptoType as any,
            description: `Test ${cryptoType} payment`
          });
        }).not.toThrow();
      });
    });

    it('should reject invalid crypto types', async () => {
      await expect(client.payments.create({
        amount_usd: '100.00',
        crypto_type: 'INVALID' as any
      })).rejects.toThrow('Invalid crypto type');
    });

    it('should reject empty crypto type', async () => {
      await expect(client.payments.create({
        amount_usd: '100.00',
        crypto_type: '' as any
      })).rejects.toThrow('Crypto type is required');
    });

    it('should reject undefined crypto type', async () => {
      await expect(client.payments.create({
        amount_usd: '100.00'
      } as any)).rejects.toThrow('Crypto type is required');
    });
  });

  describe('Optional Fields Validation', () => {
    it('should accept valid expiration_minutes', () => {
      expect(() => {
        client.payments.create({
          amount_usd: '100.00',
          crypto_type: 'ETH',
          expiration_minutes: 60
        });
      }).not.toThrow();
    });

    it('should reject expiration_minutes below minimum', async () => {
      await expect(client.payments.create({
        amount_usd: '100.00',
        crypto_type: 'ETH',
        expiration_minutes: 4
      })).rejects.toThrow('Expiration must be between 5 and 1440 minutes');
    });

    it('should reject expiration_minutes above maximum', async () => {
      await expect(client.payments.create({
        amount_usd: '100.00',
        crypto_type: 'ETH',
        expiration_minutes: 1441
      })).rejects.toThrow('Expiration must be between 5 and 1440 minutes');
    });

    it('should accept valid description', () => {
      expect(() => {
        client.payments.create({
          amount_usd: '100.00',
          crypto_type: 'ETH',
          description: 'Valid description'
        });
      }).not.toThrow();
    });

    it('should reject description that is too long', async () => {
      const longDescription = 'a'.repeat(501);
      await expect(client.payments.create({
        amount_usd: '100.00',
        crypto_type: 'ETH',
        description: longDescription
      })).rejects.toThrow('Description must be 500 characters or less');
    });

    it('should accept metadata object', () => {
      expect(() => {
        client.payments.create({
          amount_usd: '100.00',
          crypto_type: 'ETH',
          metadata: {
            order_id: '12345',
            customer_id: 'cust_67890'
          }
        });
      }).not.toThrow();
    });

    it('should accept webhook_url', () => {
      expect(() => {
        client.payments.create({
          amount_usd: '100.00',
          crypto_type: 'ETH',
          webhook_url: 'https://example.com/webhook'
        });
      }).not.toThrow();
    });
  });

  describe('Edge Cases', () => {
    it('should handle zero amounts', async () => {
      await expect(client.payments.create({
        amount_usd: '0',
        crypto_type: 'ETH'
      })).rejects.toThrow('Amount must be a positive number');

      await expect(client.payments.create({
        amount: '0.00',
        crypto_type: 'ETH'
      })).rejects.toThrow('Amount must be a positive number');
    });

    it('should handle negative amounts', async () => {
      await expect(client.payments.create({
        amount_usd: '-100',
        crypto_type: 'ETH'
      })).rejects.toThrow('Amount must be a positive number');

      await expect(client.payments.create({
        amount: '-0.5',
        crypto_type: 'ETH'
      })).rejects.toThrow('Amount must be a positive number');
    });

    it('should handle very small valid amounts', () => {
      expect(() => {
        client.payments.create({
          amount_usd: '0.01',
          crypto_type: 'ETH'
        });
      }).not.toThrow();

      expect(() => {
        client.payments.create({
          amount: '0.01',
          crypto_type: 'ETH'
        });
      }).not.toThrow();
    });

    it('should handle maximum valid amounts', () => {
      expect(() => {
        client.payments.create({
          amount_usd: '1000000',
          crypto_type: 'ETH'
        });
      }).not.toThrow();

      expect(() => {
        client.payments.create({
          amount: '1000000.00',
          crypto_type: 'ETH'
        });
      }).not.toThrow();
    });
  });
});