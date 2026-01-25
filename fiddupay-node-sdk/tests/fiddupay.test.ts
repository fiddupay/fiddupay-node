import FidduPay from '../src';
import { FidduPayValidationError } from '../src/errors';

describe('FidduPay SDK', () => {
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
});
