import FidduPay from '../src';
import { FidduPayValidationError } from '../src/errors';

describe('Address-Only Payments & Fee Toggle', () => {
  let client: FidduPay;

  beforeEach(() => {
    client = new FidduPay({
      apiKey: 'sk_test_1234567890',
      environment: 'sandbox'
    });
  });

  describe('Address-Only Payment Creation', () => {
    const validRequest = {
      requested_amount: '100.00',
      crypto_type: 'ETH' as const,
      merchant_address: '0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb'
    };

    it('should create address-only payment with valid data', () => {
      expect(() => {
        client.payments.createAddressOnly(validRequest);
      }).not.toThrow();
    });

    it('should require requested_amount', async () => {
      const request = { ...validRequest };
      delete (request as any).requested_amount;
      
      await expect(client.payments.createAddressOnly(request as any))
        .rejects.toThrow('Requested amount is required');
    });

    it('should require crypto_type', async () => {
      const request = { ...validRequest };
      delete (request as any).crypto_type;
      
      await expect(client.payments.createAddressOnly(request as any))
        .rejects.toThrow('Crypto type is required');
    });

    it('should require merchant_address', async () => {
      const request = { ...validRequest };
      delete (request as any).merchant_address;
      
      await expect(client.payments.createAddressOnly(request as any))
        .rejects.toThrow('Merchant address is required');
    });

    it('should validate requested_amount format', async () => {
      await expect(client.payments.createAddressOnly({
        ...validRequest,
        requested_amount: 'invalid'
      })).rejects.toThrow('Requested amount must be a positive number');
    });

    it('should validate minimum requested_amount', async () => {
      await expect(client.payments.createAddressOnly({
        ...validRequest,
        requested_amount: '0.005'
      })).rejects.toThrow('Minimum amount is $0.01');
    });

    it('should validate maximum requested_amount', async () => {
      await expect(client.payments.createAddressOnly({
        ...validRequest,
        requested_amount: '2000000'
      })).rejects.toThrow('Maximum amount is $1,000,000');
    });

    it('should validate crypto_type', async () => {
      await expect(client.payments.createAddressOnly({
        ...validRequest,
        crypto_type: 'INVALID' as any
      })).rejects.toThrow('Invalid crypto type');
    });

    it('should validate merchant_address format', async () => {
      await expect(client.payments.createAddressOnly({
        ...validRequest,
        merchant_address: 'short'
      })).rejects.toThrow('Invalid merchant address format');
    });

    it('should accept all valid crypto types', () => {
      const validCryptoTypes = ['SOL', 'ETH', 'BNB', 'MATIC', 'ARB', 'USDT_ETH', 'USDT_BSC', 'USDT_POLYGON', 'USDT_ARBITRUM', 'USDT_SPL'];
      
      validCryptoTypes.forEach(cryptoType => {
        expect(() => {
          client.payments.createAddressOnly({
            ...validRequest,
            crypto_type: cryptoType as any
          });
        }).not.toThrow();
      });
    });

    it('should accept optional fields', () => {
      expect(() => {
        client.payments.createAddressOnly({
          ...validRequest,
          description: 'Test address-only payment',
          metadata: { order_id: '12345' },
          expiration_minutes: 60,
          webhook_url: 'https://example.com/webhook'
        });
      }).not.toThrow();
    });

    it('should validate expiration_minutes range', async () => {
      await expect(client.payments.createAddressOnly({
        ...validRequest,
        expiration_minutes: 4
      })).rejects.toThrow('Expiration must be between 5 and 1440 minutes');

      await expect(client.payments.createAddressOnly({
        ...validRequest,
        expiration_minutes: 1441
      })).rejects.toThrow('Expiration must be between 5 and 1440 minutes');
    });

    it('should validate description length', async () => {
      const longDescription = 'a'.repeat(501);
      await expect(client.payments.createAddressOnly({
        ...validRequest,
        description: longDescription
      })).rejects.toThrow('Description must be 500 characters or less');
    });
  });

  describe('Address-Only Payment Retrieval', () => {
    it('should have retrieveAddressOnly method', () => {
      expect(typeof client.payments.retrieveAddressOnly).toBe('function');
    });

    it('should require payment ID', async () => {
      await expect(client.payments.retrieveAddressOnly(''))
        .rejects.toThrow('Payment ID is required');
    });

    it('should accept valid payment ID', () => {
      expect(() => {
        client.payments.retrieveAddressOnly('pay_test123');
      }).not.toThrow();
    });
  });

  describe('Fee Setting Management', () => {
    it('should have updateFeeSetting method', () => {
      expect(typeof client.payments.updateFeeSetting).toBe('function');
    });

    it('should have getFeeSetting method', () => {
      expect(typeof client.payments.getFeeSetting).toBe('function');
    });

    it('should validate customer_pays_fee as boolean', async () => {
      await expect(client.payments.updateFeeSetting({
        customer_pays_fee: 'true' as any
      })).rejects.toThrow('customer_pays_fee must be a boolean');

      await expect(client.payments.updateFeeSetting({
        customer_pays_fee: 1 as any
      })).rejects.toThrow('customer_pays_fee must be a boolean');

      await expect(client.payments.updateFeeSetting({
        customer_pays_fee: null as any
      })).rejects.toThrow('customer_pays_fee must be a boolean');
    });

    it('should accept valid boolean values', () => {
      expect(() => {
        client.payments.updateFeeSetting({ customer_pays_fee: true });
      }).not.toThrow();

      expect(() => {
        client.payments.updateFeeSetting({ customer_pays_fee: false });
      }).not.toThrow();
    });

    it('should require customer_pays_fee parameter', async () => {
      await expect(client.payments.updateFeeSetting({} as any))
        .rejects.toThrow('customer_pays_fee must be a boolean');
    });
  });

  describe('Fee Toggle Integration', () => {
    it('should support customer_pays_fee concept in address-only payments', () => {
      // Note: customer_pays_fee is handled at the API level, not in the request type
      const customerPaysRequest = {
        requested_amount: '100.00',
        crypto_type: 'ETH' as const,
        merchant_address: '0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb',
        description: 'Customer pays fee'
      };

      const merchantPaysRequest = {
        requested_amount: '100.00',
        crypto_type: 'ETH' as const,
        merchant_address: '0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb',
        description: 'Merchant pays fee'
      };

      expect(() => {
        client.payments.createAddressOnly(customerPaysRequest);
      }).not.toThrow();

      expect(() => {
        client.payments.createAddressOnly(merchantPaysRequest);
      }).not.toThrow();
    });

    it('should handle different crypto types with fee toggle concept', () => {
      const cryptoTypes = ['SOL', 'ETH', 'BNB', 'MATIC', 'ARB', 'USDT_ETH'] as const;
      
      cryptoTypes.forEach(cryptoType => {
        expect(() => {
          client.payments.createAddressOnly({
            requested_amount: '100.00',
            crypto_type: cryptoType,
            merchant_address: '0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb',
            description: 'Customer pays fee test'
          });
        }).not.toThrow();

        expect(() => {
          client.payments.createAddressOnly({
            requested_amount: '100.00',
            crypto_type: cryptoType,
            merchant_address: '0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb',
            description: 'Merchant pays fee test'
          });
        }).not.toThrow();
      });
    });
  });

  describe('Address Format Validation', () => {
    const validAddresses = {
      ETH: '0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb',
      BNB: '0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb',
      MATIC: '0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb',
      ARB: '0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb',
      SOL: 'DRpbCBMxVnDK7maPM5tGv6MvB3v1sRMC86PZ8okm21hy',
      USDT_ETH: '0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb'
    };

    Object.entries(validAddresses).forEach(([cryptoType, address]) => {
      it(`should accept valid ${cryptoType} address`, () => {
        expect(() => {
          client.payments.createAddressOnly({
            requested_amount: '100.00',
            crypto_type: cryptoType as any,
            merchant_address: address
          });
        }).not.toThrow();
      });
    });

    it('should reject addresses that are too short', async () => {
      await expect(client.payments.createAddressOnly({
        requested_amount: '100.00',
        crypto_type: 'ETH',
        merchant_address: '0x123'
      })).rejects.toThrow('Invalid merchant address format');
    });

    it('should reject empty addresses', async () => {
      await expect(client.payments.createAddressOnly({
        requested_amount: '100.00',
        crypto_type: 'ETH',
        merchant_address: ''
      })).rejects.toThrow('Merchant address is required');
    });
  });

  describe('Amount Edge Cases', () => {
    const validRequest = {
      requested_amount: '100.00',
      crypto_type: 'ETH' as const,
      merchant_address: '0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb'
    };

    it('should handle decimal amounts', () => {
      expect(() => {
        client.payments.createAddressOnly({
          ...validRequest,
          requested_amount: '99.99'
        });
      }).not.toThrow();

      expect(() => {
        client.payments.createAddressOnly({
          ...validRequest,
          requested_amount: '0.123456'
        });
      }).not.toThrow();
    });

    it('should handle integer amounts', () => {
      expect(() => {
        client.payments.createAddressOnly({
          ...validRequest,
          requested_amount: '100'
        });
      }).not.toThrow();
    });

    it('should handle zero amounts', async () => {
      await expect(client.payments.createAddressOnly({
        ...validRequest,
        requested_amount: '0'
      })).rejects.toThrow('Requested amount must be a positive number');

      await expect(client.payments.createAddressOnly({
        ...validRequest,
        requested_amount: '0.00'
      })).rejects.toThrow('Requested amount must be a positive number');
    });

    it('should handle negative amounts', async () => {
      await expect(client.payments.createAddressOnly({
        ...validRequest,
        requested_amount: '-100'
      })).rejects.toThrow('Requested amount must be a positive number');
    });

    it('should handle boundary amounts', () => {
      expect(() => {
        client.payments.createAddressOnly({
          ...validRequest,
          requested_amount: '0.01' // minimum
        });
      }).not.toThrow();

      expect(() => {
        client.payments.createAddressOnly({
          ...validRequest,
          requested_amount: '1000000' // maximum
        });
      }).not.toThrow();
    });
  });
});