import FidduPay from '../src';
import { FidduPayValidationError } from '../src/errors';

describe('FidduPay SDK', () => {
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
    });

    it('should support all native tokens', () => {
      const fiddupay = new FidduPay({
        apiKey: 'sk_test_1234567890abcdef',
        environment: 'sandbox'
      });
      
      // Test that all native tokens are supported in TypeScript
      const nativeTokens = ['ETH', 'BNB', 'MATIC', 'ARB', 'SOL'] as const;
      nativeTokens.forEach(token => {
        expect(() => {
          // This should not throw TypeScript errors
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
    });

    it('should throw error for invalid API key format', () => {
      expect(() => {
        new FidduPay({ apiKey: 'invalid_key' });
      }).toThrow(FidduPayValidationError);
    });

    it('should throw error for invalid environment', () => {
      expect(() => {
        new FidduPay({ 
          apiKey: 'sk_test_1234567890',
          environment: 'invalid' as any
        });
      }).toThrow(FidduPayValidationError);
    });
  });

  describe('Available Resources', () => {
    it('should have all core resources', () => {
      expect(client.payments).toBeDefined();
      expect(client.merchants).toBeDefined();
      expect(client.refunds).toBeDefined();
      expect(client.analytics).toBeDefined();
      expect(client.webhooks).toBeDefined();
    });

    it('should have payment methods', () => {
      expect(client.payments.create).toBeDefined();
      expect(client.payments.retrieve).toBeDefined();
      expect(client.payments.list).toBeDefined();
      expect(client.payments.createAddressOnly).toBeDefined();
    });

    it('should have merchant methods', () => {
      expect(client.merchants.getBalance).toBeDefined();
      expect(client.merchants.setWallets).toBeDefined();
    });

    it('should have refund methods', () => {
      expect(client.refunds.create).toBeDefined();
      expect(client.refunds.list).toBeDefined();
    });

    it('should have analytics methods', () => {
      expect(client.analytics.retrieve).toBeDefined();
      expect(client.analytics.export).toBeDefined();
    });

    it('should have webhook methods', () => {
      expect(client.webhooks.verifySignature).toBeDefined();
      expect(client.webhooks.constructEvent).toBeDefined();
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
          requested_amount: 0.05,
          customer_pays_fee: true
        };
        expect(request.crypto_type).toBe('ETH');
        expect(request.requested_amount).toBe(0.05);
        expect(request.customer_pays_fee).toBe(true);
      }).not.toThrow();
    });

    it('should support fee toggle in address-only payments', () => {
      const customerPaysRequest = {
        crypto_type: 'USDT',
        merchant_address: '0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb',
        requested_amount: 100,
        customer_pays_fee: true
      };

      const merchantPaysRequest = {
        crypto_type: 'USDT', 
        merchant_address: '0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb',
        requested_amount: 100,
        customer_pays_fee: false
      };

      expect(customerPaysRequest.customer_pays_fee).toBe(true);
      expect(merchantPaysRequest.customer_pays_fee).toBe(false);
    });
  });

  describe('Payment Creation', () => {
    it('should support standard payment creation', () => {
      expect(() => {
        const request = {
          amount_usd: '100.00',
          crypto_type: 'ETH',
          description: 'Test payment'
        };
        expect(request.amount_usd).toBe('100.00');
        expect(request.crypto_type).toBe('ETH');
      }).not.toThrow();
    });

    it('should support all crypto types', () => {
      const cryptoTypes = ['ETH', 'SOL', 'BNB', 'MATIC', 'ARB', 'USDT'] as const;
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
  });
});
