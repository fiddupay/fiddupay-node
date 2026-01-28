import { FidduPayClient } from '../src/index';

describe('SDK Live Endpoint Tests', () => {
  let client: FidduPayClient;

  beforeAll(() => {
    client = new FidduPayClient({
      apiKey: 'sk_merchant_2_secure_key',
      baseURL: 'http://127.0.0.1:8080'
    });
  });

  test('should retrieve merchant profile using updated endpoint', async () => {
    const profile = await client.merchants.retrieve();
    expect(profile).toBeDefined();
    expect(profile.business_name).toBe('Test Business');
  });

  test('should get analytics using updated endpoint', async () => {
    const analytics = await client.analytics.retrieve();
    expect(analytics).toBeDefined();
    expect(analytics.total_volume_usd).toBeDefined();
  });

  test('should get balance using updated endpoint', async () => {
    const balance = await client.balances.get();
    expect(balance).toBeDefined();
    expect(typeof balance).toBe('object');
  });

  test('should list payments using updated endpoint', async () => {
    const payments = await client.payments.list();
    expect(payments).toBeDefined();
    expect(payments.data).toBeDefined();
    expect(Array.isArray(payments.data)).toBe(true);
  });

  test('should get wallet configurations using updated endpoint', async () => {
    const wallets = await client.wallets.getConfigurations();
    expect(wallets).toBeDefined();
    expect(Array.isArray(wallets)).toBe(true);
  });

  test('should list invoices using updated endpoint', async () => {
    const invoices = await client.invoices.list();
    expect(invoices).toBeDefined();
  });

  test('should get security events using updated endpoint', async () => {
    const events = await client.security.getEvents();
    expect(events).toBeDefined();
  });

  test('should get withdrawals using updated endpoint', async () => {
    const withdrawals = await client.withdrawals.list();
    expect(withdrawals).toBeDefined();
  });
}, 30000);
