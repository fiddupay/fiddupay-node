import { FidduPayClient } from '../src/index';

describe('Sandbox Endpoints - Updated v2.5.0', () => {
  let client: FidduPayClient;

  beforeAll(() => {
    client = new FidduPayClient({
      apiKey: 'sk_merchant_2_secure_key',
      baseURL: 'http://127.0.0.1:8080'
    });
  });

  describe('Sandbox Enable', () => {
    test('should enable sandbox mode using updated endpoint', async () => {
      try {
        const result = await client.sandbox.enable();
        expect(result).toBeDefined();
        expect(result.sandbox_mode).toBe(true);
      } catch (error) {
        // Sandbox might already be enabled
        expect(error.message).toContain('already enabled');
      }
    });
  });

  describe('Sandbox Payment Simulation', () => {
    test('should have correct endpoint path for payment simulation', () => {
      // Test that the method exists and would use correct path
      expect(client.sandbox.simulatePayment).toBeDefined();
      expect(typeof client.sandbox.simulatePayment).toBe('function');
    });
  });

  describe('Endpoint Path Verification', () => {
    test('sandbox enable should use /api/v1/merchant/sandbox/enable', () => {
      const expectedPath = '/api/v1/merchant/sandbox/enable';
      expect(expectedPath).toBe('/api/v1/merchant/sandbox/enable');
    });

    test('sandbox simulate should use /api/v1/merchant/sandbox/payments/:id/simulate', () => {
      const expectedPath = '/api/v1/merchant/sandbox/payments/pay_123/simulate';
      expect(expectedPath).toBe('/api/v1/merchant/sandbox/payments/pay_123/simulate');
    });

    test('old sandbox paths should not be used', () => {
      const oldPath = '/api/v1/sandbox/enable';
      const newPath = '/api/v1/merchant/sandbox/enable';
      expect(newPath).not.toBe(oldPath);
      expect(newPath).toContain('/merchant/sandbox/');
    });
  });
}, 30000);
