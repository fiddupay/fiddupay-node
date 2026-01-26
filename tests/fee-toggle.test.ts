import FidduPay from '../src';
import { FidduPayValidationError } from '../src/errors';

describe('Fee Toggle Functionality', () => {
  let client: FidduPay;

  beforeEach(() => {
    client = new FidduPay({
      apiKey: 'sk_test_123456789',
      environment: 'sandbox'
    });
  });

  describe('Fee Setting Management', () => {
    test('should have updateFeeSetting method', () => {
      expect(typeof client.payments.updateFeeSetting).toBe('function');
    });

    test('should have getFeeSetting method', () => {
      expect(typeof client.payments.getFeeSetting).toBe('function');
    });

    test('should validate customer_pays_fee parameter', async () => {
      await expect(client.payments.updateFeeSetting({} as any))
        .rejects.toThrow('customer_pays_fee must be a boolean');
    });

    test('should validate customer_pays_fee as boolean', async () => {
      await expect(client.payments.updateFeeSetting({ customer_pays_fee: 'true' as any }))
        .rejects.toThrow('customer_pays_fee must be a boolean');
    });
  });

  describe('Address-Only Payments', () => {
    test('should have createAddressOnly method', () => {
      expect(typeof client.payments.createAddressOnly).toBe('function');
    });

    test('should have retrieveAddressOnly method', () => {
      expect(typeof client.payments.retrieveAddressOnly).toBe('function');
    });

    test('should validate required fields for address-only payment', async () => {
      await expect(client.payments.createAddressOnly({} as any))
        .rejects.toThrow('Requested amount is required');
    });

    test('should validate crypto_type for address-only payment', async () => {
      await expect(client.payments.createAddressOnly({
        requested_amount: '100.00'
      } as any)).rejects.toThrow('Crypto type is required');
    });

    test('should validate merchant_address for address-only payment', async () => {
      await expect(client.payments.createAddressOnly({
        requested_amount: '100.00',
        crypto_type: 'ETH'
      } as any)).rejects.toThrow('Merchant address is required');
    });

    test('should validate amount format', async () => {
      await expect(client.payments.createAddressOnly({
        requested_amount: 'invalid',
        crypto_type: 'ETH',
        merchant_address: '0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb'
      })).rejects.toThrow('Requested amount must be a positive number');
    });

    test('should validate minimum amount', async () => {
      await expect(client.payments.createAddressOnly({
        requested_amount: '0.005',
        crypto_type: 'ETH',
        merchant_address: '0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb'
      })).rejects.toThrow('Minimum amount is $0.01');
    });

    test('should validate maximum amount', async () => {
      await expect(client.payments.createAddressOnly({
        requested_amount: '2000000',
        crypto_type: 'ETH',
        merchant_address: '0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb'
      })).rejects.toThrow('Maximum amount is $1,000,000');
    });

    test('should validate crypto type', async () => {
      await expect(client.payments.createAddressOnly({
        requested_amount: '100.00',
        crypto_type: 'INVALID' as any,
        merchant_address: '0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb'
      })).rejects.toThrow('Invalid crypto type');
    });

    test('should validate address format', async () => {
      await expect(client.payments.createAddressOnly({
        requested_amount: '100.00',
        crypto_type: 'ETH',
        merchant_address: 'short'
      })).rejects.toThrow('Invalid merchant address format');
    });
  });
});
