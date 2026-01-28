import { FidduPay } from '../src/index';

describe('SDK Endpoint Updates', () => {
  let client: FidduPay;

  beforeEach(() => {
    client = new FidduPay({
      apiKey: 'test_key',
      baseURL: 'http://localhost:8080'
    });
  });

  test('should use updated merchant endpoints', async () => {
    // Mock the request method to capture the URL
    const mockRequest = jest.fn().mockResolvedValue({ data: { success: true } });
    (client as any).client.request = mockRequest;

    // Test merchant profile endpoint
    await client.merchants.getProfile();
    expect(mockRequest).toHaveBeenCalledWith('GET', '/api/v1/merchant/profile');

    // Test analytics endpoint
    await client.analytics.get();
    expect(mockRequest).toHaveBeenCalledWith('GET', '/api/v1/merchant/analytics');

    // Test payments endpoint
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

    // Test balance endpoint
    await client.balances.get();
    expect(mockRequest).toHaveBeenCalledWith('GET', '/api/v1/merchant/balance');

    // Test wallets endpoint
    await client.wallets.getAll();
    expect(mockRequest).toHaveBeenCalledWith('GET', '/api/v1/merchant/wallets');
  });

  test('should use correct invoice endpoints', async () => {
    const mockRequest = jest.fn().mockResolvedValue({ data: { success: true } });
    (client as any).client.request = mockRequest;

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

    await client.invoices.list();
    expect(mockRequest).toHaveBeenCalledWith('GET', '/api/v1/merchant/invoices');
  });

  test('should use correct security endpoints', async () => {
    const mockRequest = jest.fn().mockResolvedValue({ data: { success: true } });
    (client as any).client.request = mockRequest;

    await client.security.getEvents();
    expect(mockRequest).toHaveBeenCalledWith('GET', '/api/v1/merchant/security/events');

    await client.security.getAlerts();
    expect(mockRequest).toHaveBeenCalledWith('GET', '/api/v1/merchant/security/alerts');
  });
});
