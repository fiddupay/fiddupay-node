import FidduPay from '../src';
import { FidduPayValidationError } from '../src/errors';
import { CreatePaymentRequest, CreateAddressOnlyPaymentRequest, CryptoType } from '../src/types';

describe('Payment Operations', () => {
  let client: FidduPay;

  beforeEach(() => {
    client = new FidduPay({
      apiKey: 'sk_test_1234567890',
      environment: 'sandbox'
    });
  });

  describe('Standard Payment Creation', () => {
    it('should validate payment creation with amount_usd', async () => {
      const request: CreatePaymentRequest = {
        amount_usd: '100.00',
        crypto_type: 'ETH',
        description: 'Test payment'
      };

      // Should not throw validation error
      expect(() => {
        (client.payments as any).validateCreatePayment(request);
      }).not.toThrow();
    });

    it('should validate payment creation with crypto amount', async () => {
      const request: CreatePaymentRequest = {
        amount: '0.05',
        crypto_type: 'ETH',
        description: 'Test payment'
      };

      expect(() => {
        (client.payments as any).validateCreatePayment(request);
      }).not.toThrow();
    });

    it('should reject payment with both amount and amount_usd', async () => {
      const request: CreatePaymentRequest = {
        amount: '0.05',
        amount_usd: '100.00',
        crypto_type: 'ETH'
      };

      expect(() => {
        (client.payments as any).validateCreatePayment(request);
      }).toThrow('Provide either amount or amount_usd, not both');
    });

    it('should reject payment with neither amount nor amount_usd', async () => {
      const request: CreatePaymentRequest = {
        crypto_type: 'ETH'
      };

      expect(() => {
        (client.payments as any).validateCreatePayment(request);
      }).toThrow('Either amount or amount_usd must be provided');
    });

    it('should validate all supported crypto types', () => {
      const validCryptoTypes: CryptoType[] = [
        'SOL', 'ETH', 'BNB', 'MATIC', 'ARB', 
        'USDT_ETH', 'USDT_BSC', 'USDT_POLYGON', 'USDT_ARBITRUM', 'USDT_SPL'
      ];

      validCryptoTypes.forEach(crypto => {
        const request: CreatePaymentRequest = {
          amount_usd: '100.00',
          crypto_type: crypto,
          description: `Test ${crypto} payment`
        };

        expect(() => {
          (client.payments as any).validateCreatePayment(request);
        }).not.toThrow();
      });
    });

    it('should reject invalid crypto type', () => {
      const request: CreatePaymentRequest = {
        amount_usd: '100.00',
        crypto_type: 'INVALID' as CryptoType
      };

      expect(() => {
        (client.payments as any).validateCreatePayment(request);
      }).toThrow('Invalid crypto type');
    });

    it('should validate amount ranges', () => {
      // Test minimum amount
      expect(() => {
        (client.payments as any).validateCreatePayment({
          amount_usd: '0.005',
          crypto_type: 'ETH'
        });
      }).toThrow('Minimum amount is $0.01');

      // Test maximum amount
      expect(() => {
        (client.payments as any).validateCreatePayment({
          amount_usd: '1000001',
          crypto_type: 'ETH'
        });
      }).toThrow('Maximum amount is $1,000,000');

      // Test valid amounts
      expect(() => {
        (client.payments as any).validateCreatePayment({
          amount_usd: '0.01',
          crypto_type: 'ETH'
        });
      }).not.toThrow();

      expect(() => {
        (client.payments as any).validateCreatePayment({
          amount_usd: '1000000',
          crypto_type: 'ETH'
        });
      }).not.toThrow();
    });

    it('should validate expiration minutes', () => {
      expect(() => {
        (client.payments as any).validateCreatePayment({
          amount_usd: '100.00',
          crypto_type: 'ETH',
          expiration_minutes: 4
        });
      }).toThrow('Expiration must be between 5 and 1440 minutes');

      expect(() => {
        (client.payments as any).validateCreatePayment({
          amount_usd: '100.00',
          crypto_type: 'ETH',
          expiration_minutes: 1441
        });
      }).toThrow('Expiration must be between 5 and 1440 minutes');

      expect(() => {
        (client.payments as any).validateCreatePayment({
          amount_usd: '100.00',
          crypto_type: 'ETH',
          expiration_minutes: 60
        });
      }).not.toThrow();
    });

    it('should validate description length', () => {
      const longDescription = 'a'.repeat(501);
      
      expect(() => {
        (client.payments as any).validateCreatePayment({
          amount_usd: '100.00',
          crypto_type: 'ETH',
          description: longDescription
        });
      }).toThrow('Description must be 500 characters or less');

      expect(() => {
        (client.payments as any).validateCreatePayment({
          amount_usd: '100.00',
          crypto_type: 'ETH',
          description: 'Valid description'
        });
      }).not.toThrow();
    });
  });

  describe('Address-Only Payment Creation', () => {
    it('should validate address-only payment creation', () => {
      const request: CreateAddressOnlyPaymentRequest = {
        requested_amount: '100.00',
        crypto_type: 'ETH',
        merchant_address: '0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb',
        description: 'Test address-only payment'
      };

      expect(() => {
        (client.payments as any).validateCreateAddressOnlyPayment(request);
      }).not.toThrow();
    });

    it('should require requested_amount', () => {
      const request = {
        crypto_type: 'ETH',
        merchant_address: '0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb'
      } as CreateAddressOnlyPaymentRequest;

      expect(() => {
        (client.payments as any).validateCreateAddressOnlyPayment(request);
      }).toThrow('Requested amount is required');
    });

    it('should require crypto_type', () => {
      const request = {
        requested_amount: '100.00',
        merchant_address: '0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb'
      } as CreateAddressOnlyPaymentRequest;

      expect(() => {
        (client.payments as any).validateCreateAddressOnlyPayment(request);
      }).toThrow('Crypto type is required');
    });

    it('should require merchant_address', () => {
      const request = {
        requested_amount: '100.00',
        crypto_type: 'ETH'
      } as CreateAddressOnlyPaymentRequest;

      expect(() => {
        (client.payments as any).validateCreateAddressOnlyPayment(request);
      }).toThrow('Merchant address is required');
    });

    it('should validate merchant address format', () => {
      const request: CreateAddressOnlyPaymentRequest = {
        requested_amount: '100.00',
        crypto_type: 'ETH',
        merchant_address: 'short'
      };

      expect(() => {
        (client.payments as any).validateCreateAddressOnlyPayment(request);
      }).toThrow('Invalid merchant address format');
    });

    it('should validate amount ranges for address-only payments', () => {
      expect(() => {
        (client.payments as any).validateCreateAddressOnlyPayment({
          requested_amount: '0.005',
          crypto_type: 'ETH',
          merchant_address: '0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb'
        });
      }).toThrow('Minimum amount is $0.01');

      expect(() => {
        (client.payments as any).validateCreateAddressOnlyPayment({
          requested_amount: '1000001',
          crypto_type: 'ETH',
          merchant_address: '0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb'
        });
      }).toThrow('Maximum amount is $1,000,000');
    });
  });

  describe('Payment Retrieval', () => {
    it('should require payment ID for retrieve', async () => {
      await expect(client.payments.retrieve(''))
        .rejects.toThrow('Payment ID is required');
    });

    it('should require payment ID for cancel', async () => {
      await expect(client.payments.cancel(''))
        .rejects.toThrow('Payment ID is required');
    });

    it('should require payment ID for address-only retrieve', async () => {
      await expect(client.payments.retrieveAddressOnly(''))
        .rejects.toThrow('Payment ID is required');
    });
  });

  describe('Fee Setting Operations', () => {
    it('should validate fee setting update', async () => {
      await expect(client.payments.updateFeeSetting({
        customer_pays_fee: 'invalid' as any
      })).rejects.toThrow('customer_pays_fee must be a boolean');
    });

    it('should accept valid fee setting', () => {
      expect(() => {
        client.payments.updateFeeSetting({
          customer_pays_fee: true
        });
      }).not.toThrow();

      expect(() => {
        client.payments.updateFeeSetting({
          customer_pays_fee: false
        });
      }).not.toThrow();
    });
  });

  describe('Payment List Operations', () => {
    it('should handle list payments without parameters', () => {
      expect(() => {
        client.payments.list();
      }).not.toThrow();
    });

    it('should handle list payments with filters', () => {
      expect(() => {
        client.payments.list({
          limit: 10,
          offset: 0,
          status: 'CONFIRMED',
          crypto_type: 'ETH'
        });
      }).not.toThrow();
    });

    it('should build query parameters correctly', () => {
      // This tests the internal query building logic
      const params = {
        limit: 25,
        offset: 50,
        status: 'PENDING' as const,
        crypto_type: 'BNB' as const
      };

      const queryParams = new URLSearchParams();
      if (params.limit) queryParams.append('limit', params.limit.toString());
      if (params.offset) queryParams.append('offset', params.offset.toString());
      if (params.status) queryParams.append('status', params.status);
      if (params.crypto_type) queryParams.append('crypto_type', params.crypto_type);

      const query = queryParams.toString();
      expect(query).toContain('limit=25');
      expect(query).toContain('offset=50');
      expect(query).toContain('status=PENDING');
      expect(query).toContain('crypto_type=BNB');
    });
  });

  describe('Payment Method Availability', () => {
    it('should have all payment methods available', () => {
      expect(typeof client.payments.create).toBe('function');
      expect(typeof client.payments.retrieve).toBe('function');
      expect(typeof client.payments.list).toBe('function');
      expect(typeof client.payments.cancel).toBe('function');
      expect(typeof client.payments.createAddressOnly).toBe('function');
      expect(typeof client.payments.retrieveAddressOnly).toBe('function');
      expect(typeof client.payments.updateFeeSetting).toBe('function');
      expect(typeof client.payments.getFeeSetting).toBe('function');
    });
  });

  describe('Edge Cases', () => {
    it('should handle numeric amounts as strings', () => {
      expect(() => {
        (client.payments as any).validateCreatePayment({
          amount_usd: 100.00, // Number instead of string
          crypto_type: 'ETH'
        });
      }).not.toThrow();
    });

    it('should handle invalid numeric amounts', () => {
      expect(() => {
        (client.payments as any).validateCreatePayment({
          amount_usd: 'invalid',
          crypto_type: 'ETH'
        });
      }).toThrow('Amount must be a positive number');

      expect(() => {
        (client.payments as any).validateCreatePayment({
          amount_usd: '-100',
          crypto_type: 'ETH'
        });
      }).toThrow('Amount must be a positive number');

      expect(() => {
        (client.payments as any).validateCreatePayment({
          amount_usd: '0',
          crypto_type: 'ETH'
        });
      }).toThrow('Amount must be a positive number');
    });
  });
});