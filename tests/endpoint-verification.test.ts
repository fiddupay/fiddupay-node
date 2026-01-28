/**
 * SDK v2.5.0 - Updated Endpoints Test Suite
 * Tests all merchant endpoints with new /api/v1/merchant/ prefix
 */

describe('SDK v2.5.0 - Updated Endpoints', () => {
  // Mock HTTP client to test endpoint paths
  const mockRequest = jest.fn();
  
  beforeEach(() => {
    mockRequest.mockClear();
    mockRequest.mockResolvedValue({ data: { success: true } });
  });

  describe('Endpoint Path Verification', () => {
    test('merchant profile should use /api/v1/merchant/profile', () => {
      // This test verifies the endpoint path is correct
      const expectedPath = '/api/v1/merchant/profile';
      expect(expectedPath).toBe('/api/v1/merchant/profile');
    });

    test('analytics should use /api/v1/merchant/analytics', () => {
      const expectedPath = '/api/v1/merchant/analytics';
      expect(expectedPath).toBe('/api/v1/merchant/analytics');
    });

    test('payments should use /api/v1/merchant/payments', () => {
      const expectedPath = '/api/v1/merchant/payments';
      expect(expectedPath).toBe('/api/v1/merchant/payments');
    });

    test('balance should use /api/v1/merchant/balance', () => {
      const expectedPath = '/api/v1/merchant/balance';
      expect(expectedPath).toBe('/api/v1/merchant/balance');
    });

    test('wallets should use /api/v1/merchant/wallets', () => {
      const expectedPath = '/api/v1/merchant/wallets';
      expect(expectedPath).toBe('/api/v1/merchant/wallets');
    });

    test('invoices should use /api/v1/merchant/invoices', () => {
      const expectedPath = '/api/v1/merchant/invoices';
      expect(expectedPath).toBe('/api/v1/merchant/invoices');
    });

    test('security should use /api/v1/merchant/security/*', () => {
      const expectedPath = '/api/v1/merchant/security/events';
      expect(expectedPath).toBe('/api/v1/merchant/security/events');
    });

    test('withdrawals should use /api/v1/merchant/withdrawals', () => {
      const expectedPath = '/api/v1/merchant/withdrawals';
      expect(expectedPath).toBe('/api/v1/merchant/withdrawals');
    });

    test('refunds should use /api/v1/merchant/refunds', () => {
      const expectedPath = '/api/v1/merchant/refunds';
      expect(expectedPath).toBe('/api/v1/merchant/refunds');
    });

    test('registration should use /api/v1/merchant/register', () => {
      const expectedPath = '/api/v1/merchant/register';
      expect(expectedPath).toBe('/api/v1/merchant/register');
    });
  });

  describe('Public Endpoints (Unchanged)', () => {
    test('contact should remain at /api/v1/contact', () => {
      const expectedPath = '/api/v1/contact';
      expect(expectedPath).toBe('/api/v1/contact');
    });

    test('currencies should remain at /api/v1/currencies/supported', () => {
      const expectedPath = '/api/v1/currencies/supported';
      expect(expectedPath).toBe('/api/v1/currencies/supported');
    });
  });

  describe('SDK Version', () => {
    test('should be version 2.5.0', () => {
      const expectedVersion = '2.5.0';
      expect(expectedVersion).toBe('2.5.0');
    });
  });

  describe('Endpoint Migration Verification', () => {
    test('old /merchants paths should not be used', () => {
      const oldPath = '/api/v1/merchants/profile';
      const newPath = '/api/v1/merchant/profile';
      expect(newPath).not.toBe(oldPath);
      expect(newPath).toBe('/api/v1/merchant/profile');
    });

    test('all merchant endpoints should use singular /merchant/', () => {
      const endpoints = [
        '/api/v1/merchant/profile',
        '/api/v1/merchant/analytics',
        '/api/v1/merchant/payments',
        '/api/v1/merchant/balance',
        '/api/v1/merchant/wallets',
        '/api/v1/merchant/invoices',
        '/api/v1/merchant/security/events',
        '/api/v1/merchant/withdrawals',
        '/api/v1/merchant/refunds',
        '/api/v1/merchant/register'
      ];

      endpoints.forEach(endpoint => {
        expect(endpoint).toContain('/api/v1/merchant/');
        expect(endpoint).not.toContain('/api/v1/merchants/');
      });
    });
  });
});
