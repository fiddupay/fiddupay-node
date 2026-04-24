import { Merchants } from '../src/resources/merchants';
import { Payments } from '../src/resources/payments';
import { HttpClient } from '../src/client';
import { MerchantRegistrationRequest, UnifiedSettingsRequest } from '../src/types';
import { AddressOnly } from '../src/resources/address_only';
import { Invoices } from '../src/resources/invoices';

describe('Compliance and Synchronization (v2.6.17)', () => {
  let mockHttpClient: any;
  let merchants: Merchants;
  let payments: Payments;

  beforeEach(() => {
    mockHttpClient = {
      request: jest.fn(),
      get: jest.fn(),
      post: jest.fn(),
      patch: jest.fn(),
      put: jest.fn(),
    };
    merchants = new Merchants(mockHttpClient);
    payments = new Payments(mockHttpClient);
    (merchants as any).addressOnly = new AddressOnly(mockHttpClient);
    (merchants as any).invoices = new Invoices(mockHttpClient);
  });

  test('MerchantRegistrationRequest supports new KYC fields', async () => {
    const registrationData: MerchantRegistrationRequest = {
      email: 'test@merchant.com',
      business_name: 'Test Inc',
      password: 'secure123',
      first_name: 'John',
      last_name: 'Doe',
      gender: 'Male',
      phone_number: '+1234567890',
      country: 'US',
      applicant_role: 'Director',
      terms_accepted: true,
      business_country: 'US',
      business_license_number: 'LIC123456',
      website_url: 'https://test.com'
    };

    mockHttpClient.request.mockResolvedValue({ user: { id: 1 }, dashboard_token: 'token' });
    
    await merchants.register(registrationData);
    
    expect(mockHttpClient.request).toHaveBeenCalledWith(
      'POST',
      '/api/v1/merchants/register',
      registrationData
    );
  });

  test('UnifiedSettingsRequest supports sandbox flags and withdrawal fee', async () => {
    const settingsData: UnifiedSettingsRequest = {
      withdrawal_fee_percentage: 0.5,
      solana_sandbox_enabled: true,
      bnb_sandbox_enabled: false,
      eth_sandbox_enabled: true
    };

    mockHttpClient.request.mockResolvedValue({ status: 'success', message: 'Updated' });
    
    await merchants.updateSettings(settingsData);
    
    expect(mockHttpClient.request).toHaveBeenCalledWith(
      'PATCH',
      '/api/v1/merchants/settings',
      settingsData
    );
  });

  test('Payment validation allows new crypto types', async () => {
    const paymentData = {
      amount_usd: '100.00',
      crypto_type: 'USDC_POLYGON' as any,
      description: 'Test USDC payment'
    };

    mockHttpClient.request.mockResolvedValue({ payment_id: 'pay_123' });
    
    await payments.create(paymentData as any);
    
    expect(mockHttpClient.request).toHaveBeenCalledWith(
      'POST',
      '/api/v1/merchants/payments',
      paymentData
    );
  });

  test('Payment validation fails for unsupported crypto types', async () => {
    const paymentData = {
      amount_usd: '100.00',
      crypto_type: 'INVALID_COIN' as any
    };

    await expect(payments.create(paymentData as any)).rejects.toThrow('Invalid crypto type');
  });

  test('AddressOnly resource parity', async () => {
    const addressOnly = new AddressOnly(mockHttpClient);
    const data = { crypto_type: 'SOL' as any, merchant_address: 'addr123', requested_amount: '1.0' };
    
    mockHttpClient.request.mockResolvedValue({ payment_id: 'ao_123' });
    await addressOnly.create(data);
    
    expect(mockHttpClient.request).toHaveBeenCalledWith('POST', '/api/v1/merchants/address-only/create', data);
  });

  test('Merchant security methods parity', async () => {
    mockHttpClient.request.mockResolvedValue({ success: true });
    
    await merchants.toggleWalletLock(true);
    expect(mockHttpClient.request).toHaveBeenCalledWith('POST', '/api/v1/merchants/security/wallets/lock', { locked: true });
    
    await merchants.setTransactionPin('1234');
    expect(mockHttpClient.request).toHaveBeenCalledWith('POST', '/api/v1/merchants/security/transaction-pin', { pin: '1234' });
  });
});
