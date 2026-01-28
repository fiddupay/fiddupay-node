import { FidduPayClient } from '../src/index';

describe('SDK Updated Endpoints', () => {
  let client: FidduPayClient;
  let mockRequest: jest.Mock;

  beforeEach(() => {
    client = new FidduPayClient({
      apiKey: 'sk_test_key',
      baseURL: 'http://localhost:8080'
    });

    // Mock the request method
    mockRequest = jest.fn().mockResolvedValue({ data: { success: true } });
    (client as any).client.request = mockRequest;
  });

  afterEach(() => {
    jest.clearAllMocks();
  });

  describe('Merchant endpoints', () => {
    test('should use /api/v1/merchant/profile for merchant profile', async () => {
      await client.merchants.retrieve();
      expect(mockRequest).toHaveBeenCalledWith('GET', '/api/v1/merchant/profile');
    });

    test('should use /api/v1/merchant/register for registration', async () => {
      const registrationClient = new FidduPayClient({
        apiKey: 'registration_key',
        baseURL: 'http://localhost:8080'
      });
      (registrationClient as any).client.request = mockRequest;

      await registrationClient.merchants.register({
        email: 'test@example.com',
        business_name: 'Test Business',
        password: 'password123'
      });

      expect(mockRequest).toHaveBeenCalledWith('POST', '/api/v1/merchant/register', {
        email: 'test@example.com',
        business_name: 'Test Business',
        password: 'password123'
      });
    });
  });

  describe('Analytics endpoints', () => {
    test('should use /api/v1/merchant/analytics', async () => {
      await client.analytics.retrieve();
      expect(mockRequest).toHaveBeenCalledWith('GET', '/api/v1/merchant/analytics');
    });

    test('should use /api/v1/merchant/analytics with query params', async () => {
      await client.analytics.retrieve({ granularity: 'day' });
      expect(mockRequest).toHaveBeenCalledWith('GET', '/api/v1/merchant/analytics?granularity=day');
    });
  });

  describe('Payment endpoints', () => {
    test('should use /api/v1/merchant/payments for creating payments', async () => {
      await client.payments.create({
        amount_usd: '10.00',
        crypto_type: 'ETH',
        description: 'Test payment'
      });

      expect(mockRequest).toHaveBeenCalledWith('POST', '/api/v1/merchant/payments', {
        amount_usd: '10.00',
        crypto_type: 'ETH',
        description: 'Test payment'
      });
    });

    test('should use /api/v1/merchant/payments for listing payments', async () => {
      await client.payments.list();
      expect(mockRequest).toHaveBeenCalledWith('GET', '/api/v1/merchant/payments');
    });
  });

  describe('Balance endpoints', () => {
    test('should use /api/v1/merchant/balance', async () => {
      await client.balances.get();
      expect(mockRequest).toHaveBeenCalledWith('GET', '/api/v1/merchant/balance');
    });

    test('should use /api/v1/merchant/balance/history', async () => {
      await client.balances.getHistory();
      expect(mockRequest).toHaveBeenCalledWith('GET', '/api/v1/merchant/balance/history');
    });
  });

  describe('Wallet endpoints', () => {
    test('should use /api/v1/merchant/wallets', async () => {
      await client.wallets.getConfigurations();
      expect(mockRequest).toHaveBeenCalledWith('GET', '/api/v1/merchant/wallets');
    });

    test('should use /api/v1/merchant/wallets/generate', async () => {
      await client.wallets.generate({ crypto_type: 'ETH' });
      expect(mockRequest).toHaveBeenCalledWith('POST', '/api/v1/merchant/wallets/generate', {
        crypto_type: 'ETH'
      });
    });
  });

  describe('Invoice endpoints', () => {
    test('should use /api/v1/merchant/invoices for creating invoices', async () => {
      await client.invoices.create({
        amount_usd: '50.00',
        description: 'Test invoice',
        due_date: '2026-02-01T00:00:00Z'
      });

      expect(mockRequest).toHaveBeenCalledWith('POST', '/api/v1/merchant/invoices', {
        amount_usd: '50.00',
        description: 'Test invoice',
        due_date: '2026-02-01T00:00:00Z'
      });
    });

    test('should use /api/v1/merchant/invoices for listing invoices', async () => {
      await client.invoices.list();
      expect(mockRequest).toHaveBeenCalledWith('GET', '/api/v1/merchant/invoices');
    });
  });

  describe('Security endpoints', () => {
    test('should use /api/v1/merchant/security/events', async () => {
      await client.security.getEvents();
      expect(mockRequest).toHaveBeenCalledWith('GET', '/api/v1/merchant/security/events');
    });

    test('should use /api/v1/merchant/security/alerts', async () => {
      await client.security.getAlerts();
      expect(mockRequest).toHaveBeenCalledWith('GET', '/api/v1/merchant/security/alerts');
    });
  });

  describe('Withdrawal endpoints', () => {
    test('should use /api/v1/merchant/withdrawals', async () => {
      await client.withdrawals.create({
        crypto_type: 'ETH',
        amount: '1.0',
        destination_address: '0x123...'
      });

      expect(mockRequest).toHaveBeenCalledWith('POST', '/api/v1/merchant/withdrawals', {
        crypto_type: 'ETH',
        amount: '1.0',
        destination_address: '0x123...'
      });
    });
  });

  describe('Refund endpoints', () => {
    test('should use /api/v1/merchant/refunds', async () => {
      await client.refunds.create({
        payment_id: 'pay_123',
        amount_usd: '10.00',
        reason: 'Customer request'
      });

      expect(mockRequest).toHaveBeenCalledWith('POST', '/api/v1/merchant/refunds', {
        payment_id: 'pay_123',
        amount_usd: '10.00',
        reason: 'Customer request'
      });
    });
  });

  describe('Public endpoints (unchanged)', () => {
    test('should use /api/v1/contact for contact form', async () => {
      await client.contact.submit({
        name: 'Test User',
        email: 'test@example.com',
        subject: 'Test',
        message: 'Test message'
      });

      expect(mockRequest).toHaveBeenCalledWith('POST', '/api/v1/contact', {
        name: 'Test User',
        email: 'test@example.com',
        subject: 'Test',
        message: 'Test message'
      });
    });
  });
});
