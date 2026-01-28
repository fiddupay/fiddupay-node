import { FidduPayClient } from '../src';

describe('SDK Integration Tests - Updated Endpoints', () => {
  let client: FidduPayClient;

  beforeAll(() => {
    client = new FidduPayClient({
      apiKey: 'sk_merchant_2_secure_key',
      baseURL: 'http://127.0.0.1:8080',
      timeout: 30000
    });
  });

  describe('Merchant Endpoints', () => {
    test('should retrieve merchant profile', async () => {
      const profile = await client.merchants.retrieve();
      expect(profile).toBeDefined();
      expect(profile.business_name).toBe('Test Business');
      expect(profile.email).toBeDefined();
    });

    test('should get merchant analytics', async () => {
      const analytics = await client.analytics.retrieve();
      expect(analytics).toBeDefined();
      expect(analytics.total_volume_usd).toBeDefined();
      expect(analytics.successful_payments).toBeDefined();
    });

    test('should get merchant balance', async () => {
      const balance = await client.balances.get();
      expect(balance).toBeDefined();
      expect(typeof balance).toBe('object');
    });

    test('should list payments', async () => {
      const payments = await client.payments.list();
      expect(payments).toBeDefined();
      expect(payments.data).toBeDefined();
      expect(Array.isArray(payments.data)).toBe(true);
    });

    test('should get wallet configurations', async () => {
      const wallets = await client.wallets.getConfigurations();
      expect(wallets).toBeDefined();
    });

    test('should list invoices', async () => {
      const invoices = await client.invoices.list();
      expect(invoices).toBeDefined();
    });

    test('should get security events', async () => {
      const events = await client.security.getEvents();
      expect(events).toBeDefined();
    });

    test('should list withdrawals', async () => {
      const withdrawals = await client.withdrawals.list();
      expect(withdrawals).toBeDefined();
    });
  });

  describe('Payment Operations', () => {
    test('should create a payment', async () => {
      const payment = await client.payments.create({
        amount_usd: '10.00',
        crypto_type: 'ETH',
        description: 'Test payment'
      });
      expect(payment).toBeDefined();
      expect(payment.id).toBeDefined();
      expect(payment.amount_usd).toBe('10.00');
    });
  });

  describe('Invoice Operations', () => {
    test('should create an invoice', async () => {
      const invoice = await client.invoices.create({
        amount_usd: '50.00',
        description: 'Test invoice',
        due_date: '2026-02-01T00:00:00Z'
      });
      expect(invoice).toBeDefined();
      expect(invoice.id).toBeDefined();
      expect(invoice.amount_usd).toBe('50.00');
    });
  });

  describe('Analytics with Parameters', () => {
    test('should get analytics with granularity', async () => {
      const analytics = await client.analytics.retrieve({ granularity: 'day' });
      expect(analytics).toBeDefined();
      expect(analytics.total_volume_usd).toBeDefined();
    });

    test('should get analytics with date range', async () => {
      const analytics = await client.analytics.retrieve({
        start_date: '2026-01-01',
        end_date: '2026-01-31',
        granularity: 'week'
      });
      expect(analytics).toBeDefined();
    });
  });

  describe('Public Endpoints (Unchanged)', () => {
    test('should submit contact form', async () => {
      const result = await client.contact.submit({
        name: 'Test User',
        email: 'test@example.com',
        subject: 'SDK Test',
        message: 'Testing SDK contact endpoint'
      });
      expect(result).toBeDefined();
    });
  });
}, 60000);
