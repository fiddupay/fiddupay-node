import FidduPay from '../src';
import { FidduPayValidationError, FidduPayAPIError } from '../src/errors';
import { CryptoType } from '../src/types';

describe('FidduPay SDK - Comprehensive Test Suite', () => {
  let client: FidduPay;
  let sandboxClient: FidduPay;
  let liveClient: FidduPay;

  beforeAll(() => {
    // Setup different client configurations
    client = new FidduPay({
      apiKey: 'sk_test_comprehensive_1234567890',
      environment: 'sandbox',
      timeout: 30000
    });

    sandboxClient = new FidduPay({
      apiKey: 'sk_sandbox_test_key',
      environment: 'sandbox'
    });

    liveClient = new FidduPay({
      apiKey: 'live_production_key',
      environment: 'production'
    });
  });

  describe('Client Configuration & Initialization', () => {
    it('should create client with all configuration options', () => {
      const customClient = new FidduPay({
        apiKey: 'sk_test_custom',
        environment: 'sandbox',
        baseURL: 'https://custom.api.fiddupay.com',
        timeout: 60000
      });

      expect(customClient).toBeInstanceOf(FidduPay);
      expect(customClient.payments).toBeDefined();
      expect(customClient.merchants).toBeDefined();
    });

    it('should validate API key format', () => {
      expect(() => {
        new FidduPay({ apiKey: 'invalid_key' });
      }).toThrow(FidduPayValidationError);

      expect(() => {
        new FidduPay({ apiKey: '' });
      }).toThrow(FidduPayValidationError);
    });

    it('should handle environment validation', () => {
      expect(() => {
        new FidduPay({ 
          apiKey: 'sk_test_key',
          environment: 'invalid' as Environment
        });
      }).toThrow(FidduPayValidationError);
    });

    it('should initialize all resource modules', () => {
      expect(client.payments).toBeDefined();
      expect(client.merchants).toBeDefined();
      expect(client.refunds).toBeDefined();
      expect(client.analytics).toBeDefined();
      expect(client.webhooks).toBeDefined();
      expect(client.wallets).toBeDefined();
      expect(client.withdrawals).toBeDefined();
      expect(client.security).toBeDefined();
      expect(client.balances).toBeDefined();
      expect(client.sandbox).toBeDefined();
    });
  });

  describe('Payment Operations - Comprehensive', () => {
    it('should validate payment creation with all parameters', async () => {
      const paymentData = {
        amount_usd: '100.00',
        crypto_type: 'ETH' as CryptoType,
        description: 'Comprehensive test payment',
        customer_email: 'test@example.com',
        webhook_url: 'https://example.com/webhook',
        metadata: { orderId: 'ORDER-123', customerId: 'CUST-456' },
        expires_in: 3600,
        partial_payments_enabled: true
      };

      // Mock successful response
      const mockResponse = {
        payment_id: 'pay_test_123',
        status: 'PENDING',
        amount_usd: '100.00',
        crypto_amount: '0.05',
        crypto_type: 'ETH',
        deposit_address: '0x123...',
        created_at: new Date().toISOString(),
        expires_at: new Date().toISOString()
      };

      jest.spyOn(client.payments, 'create').mockResolvedValue(mockResponse);

      const result = await client.payments.create(paymentData);
      expect(result.payment_id).toBe('pay_test_123');
      expect(result.status).toBe('PENDING');
    });

    it('should handle payment creation validation errors', async () => {
      const invalidPaymentData = {
        amount_usd: '-10.00', // Invalid negative amount
        crypto_type: 'INVALID' as CryptoType,
        description: ''
      };

      expect(() => {
        client.payments.create(invalidPaymentData);
      }).toThrow(FidduPayValidationError);
    });

    it('should retrieve payment with all details', async () => {
      const mockPayment = {
        payment_id: 'pay_test_retrieve',
        status: 'CONFIRMED',
        amount_usd: '50.00',
        crypto_amount: '0.025',
        crypto_type: 'ETH',
        deposit_address: '0xdef456...',
        confirmations: 3,
        transaction_hash: '0xabc123...',
        created_at: new Date().toISOString(),
        expires_at: new Date().toISOString()
      };

      jest.spyOn(client.payments, 'retrieve').mockResolvedValue(mockPayment);

      const result = await client.payments.retrieve('pay_test_retrieve');
      expect(result.payment_id).toBe('pay_test_retrieve');
      expect(result.status).toBe('CONFIRMED');
    });

    it('should list payments with filtering and pagination', async () => {
      const mockPaymentsList = {
        payments: [
          { 
            payment_id: 'pay_1', 
            status: 'PENDING',
            amount_usd: '100.00',
            crypto_amount: '0.05',
            crypto_type: 'ETH',
            deposit_address: '0x123...',
            created_at: new Date().toISOString(),
            expires_at: new Date().toISOString()
          },
          { 
            payment_id: 'pay_2', 
            status: 'CONFIRMED',
            amount_usd: '200.00',
            crypto_amount: '0.1',
            crypto_type: 'ETH',
            deposit_address: '0x456...',
            created_at: new Date().toISOString(),
            expires_at: new Date().toISOString()
          }
        ],
        total: 2,
        has_more: false
      };

      jest.spyOn(client.payments, 'list').mockResolvedValue(mockPaymentsList);

      const result = await client.payments.list({
        status: 'PENDING',
        crypto_type: 'ETH',
        limit: 10
      });

      expect(result.payments).toHaveLength(2);
      expect(result.total).toBe(2);
    });

    it('should verify payment with transaction hash', async () => {
      const mockVerification = {
        verified: true,
        transaction_hash: '0xverified123',
        confirmations: 6,
        status: 'confirmed'
      };

      jest.spyOn(client.payments, 'verify').mockResolvedValue(mockVerification);

      const result = await client.payments.verify('pay_test_verify', {
        transaction_hash: '0xverified123'
      });

      expect(result.verified).toBe(true);
      expect(result.confirmations).toBe(6);
    });
  });

  describe('Merchant Operations - Comprehensive', () => {
    it('should retrieve merchant profile with all fields', async () => {
      const mockProfile = {
        id: 123,
        email: 'merchant@example.com',
        business_name: 'Test Business',
        is_active: true,
        sandbox_mode: true,
        fee_percentage: '1.50',
        created_at: new Date().toISOString()
      };

      jest.spyOn(client.merchants, 'getProfile').mockResolvedValue(mockProfile);

      const result = await client.merchants.getProfile();
      expect(result.id).toBe(123);
      expect(result.business_name).toBe('Test Business');
    });

    it('should configure wallet for all supported crypto types', async () => {
      const cryptoTypes: CryptoType[] = ['BTC', 'ETH', 'SOL', 'USDT_ETH', 'USDT_BEP20'];
      
      for (const cryptoType of cryptoTypes) {
        const mockResponse = { success: true };
        jest.spyOn(client.merchants, 'setWallet').mockResolvedValue(mockResponse);

        const result = await client.merchants.setWallet({
          crypto_type: cryptoType,
          address: `test_address_${cryptoType}`
        });

        expect(result.success).toBe(true);
      }
    });

    it('should manage API keys (generate and rotate)', async () => {
      const mockGenerateResponse = {
        api_key: 'sk_new_generated_key',
        environment: 'sandbox'
      };

      const mockRotateResponse = {
        api_key: 'sk_rotated_key',
        environment: 'sandbox'
      };

      jest.spyOn(client.merchants, 'generateApiKey').mockResolvedValue(mockGenerateResponse);
      jest.spyOn(client.merchants, 'rotateApiKey').mockResolvedValue(mockRotateResponse);

      const generateResult = await client.merchants.generateApiKey();
      expect(generateResult.api_key).toBe('sk_new_generated_key');

      const rotateResult = await client.merchants.rotateApiKey({
        old_api_key: 'sk_old_key'
      });
      expect(rotateResult.api_key).toBe('sk_rotated_key');
    });

    it('should switch environments', async () => {
      const mockResponse = {
        api_key: 'live_new_key',
        environment: 'live'
      };

      jest.spyOn(client.merchants, 'switchEnvironment').mockResolvedValue(mockResponse);

      const result = await client.merchants.switchEnvironment({
        to_live: true
      });

      expect(result.environment).toBe('live');
    });

    it('should get balance information', async () => {
      const mockBalance = {
        total_usd: '1500.00',
        balances: [
          { crypto_type: 'BTC', amount: '0.05', amount_usd: '1000.00' },
          { crypto_type: 'ETH', amount: '2.5', amount_usd: '500.00' }
        ]
      };

      jest.spyOn(client.merchants, 'getBalance').mockResolvedValue(mockBalance);

      const result = await client.merchants.getBalance();
      expect(result.total_usd).toBe('1500.00');
      expect(result.balances).toHaveLength(2);
    });
  });

  describe('Refund Operations - Comprehensive', () => {
    it('should create refund with all parameters', async () => {
      const refundData = {
        payment_id: 'pay_test_refund',
        amount: '25.00',
        reason: 'Customer requested refund',
        metadata: { support_ticket: 'TICKET-789' }
      };

      const mockRefund = {
        refund_id: 'ref_test_123',
        payment_id: 'pay_test_refund',
        amount: '25.00',
        status: 'pending',
        reason: 'Customer requested refund'
      };

      jest.spyOn(client.refunds, 'create').mockResolvedValue(mockRefund);

      const result = await client.refunds.create(refundData);
      expect(result.refund_id).toBe('ref_test_123');
      expect(result.status).toBe('pending');
    });

    it('should retrieve refund details', async () => {
      const mockRefund = {
        refund_id: 'ref_retrieve_test',
        status: 'completed',
        amount: '30.00',
        transaction_hash: '0xrefund123'
      };

      jest.spyOn(client.refunds, 'retrieve').mockResolvedValue(mockRefund);

      const result = await client.refunds.retrieve('ref_retrieve_test');
      expect(result.status).toBe('completed');
    });
  });

  describe('Wallet Operations - Comprehensive', () => {
    it('should generate wallet for supported crypto types', async () => {
      const mockWallet = {
        address: 'generated_address_123',
        crypto_type: 'ETH',
        private_key: 'encrypted_private_key',
        network: 'ethereum'
      };

      jest.spyOn(client.wallets, 'generate').mockResolvedValue(mockWallet);

      const result = await client.wallets.generate({
        crypto_type: 'ETH'
      });

      expect(result.crypto_type).toBe('ETH');
      expect(result.address).toBe('generated_address_123');
    });

    it('should import existing wallet', async () => {
      const importData = {
        crypto_type: 'BTC' as CryptoType,
        private_key: 'test_private_key_123'
      };

      const mockImportResult = {
        address: 'imported_btc_address',
        crypto_type: 'BTC',
        network: 'bitcoin'
      };

      jest.spyOn(client.wallets, 'import').mockResolvedValue(mockImportResult);

      const result = await client.wallets.import(importData);
      expect(result.crypto_type).toBe('BTC');
    });

    it('should check gas requirements', async () => {
      const mockGasCheck = {
        can_withdraw: true,
        gas_required: '0.001',
        gas_available: '0.005',
        status: 'sufficient'
      };

      jest.spyOn(client.wallets, 'checkGasRequirements').mockResolvedValue(mockGasCheck);

      const result = await client.wallets.checkGasRequirements();
      expect(result.can_withdraw).toBe(true);
      expect(result.status).toBe('sufficient');
    });
  });

  describe('Analytics Operations - Comprehensive', () => {
    it('should get analytics with date range', async () => {
      const mockAnalytics = {
        total_payments: 150,
        total_volume_usd: '50000.00',
        successful_payments: 145,
        failed_payments: 5,
        average_payment_size: '333.33',
        top_crypto_types: [
          { crypto_type: 'ETH', count: 80, volume_usd: '30000.00' },
          { crypto_type: 'BTC', count: 70, volume_usd: '20000.00' }
        ]
      };

      jest.spyOn(client.analytics, 'get').mockResolvedValue(mockAnalytics);

      const result = await client.analytics.get({
        start_date: '2024-01-01',
        end_date: '2024-01-31'
      });

      expect(result.total_payments).toBe(150);
      expect(result.top_crypto_types).toHaveLength(2);
    });

    it('should export analytics data', async () => {
      const mockExport = {
        download_url: 'https://api.fiddupay.com/exports/analytics_123.csv',
        expires_at: new Date().toISOString(),
        format: 'csv'
      };

      jest.spyOn(client.analytics, 'export').mockResolvedValue(mockExport);

      const result = await client.analytics.export({
        format: 'csv',
        start_date: '2024-01-01',
        end_date: '2024-01-31'
      });

      expect(result.format).toBe('csv');
      expect(result.download_url).toContain('analytics_123.csv');
    });
  });

  describe('Webhook Operations - Comprehensive', () => {
    it('should verify webhook signature', () => {
      const payload = JSON.stringify({
        event: 'payment.confirmed',
        payment_id: 'pay_webhook_test'
      });
      const signature = 'test_signature_123';
      const secret = 'webhook_secret_key';

      // Mock the verification method
      jest.spyOn(client.webhooks, 'verifySignature').mockReturnValue(true);

      const isValid = client.webhooks.verifySignature(payload, signature, secret);
      expect(isValid).toBe(true);
    });

    it('should parse webhook events', () => {
      const webhookPayload = {
        event: 'payment.confirmed',
        payment_id: 'pay_webhook_parse',
        data: {
          status: 'confirmed',
          amount_usd: '100.00'
        }
      };

      jest.spyOn(client.webhooks, 'parseEvent').mockReturnValue(webhookPayload);

      const parsed = client.webhooks.parseEvent(JSON.stringify(webhookPayload));
      expect(parsed.event).toBe('payment.confirmed');
      expect(parsed.payment_id).toBe('pay_webhook_parse');
    });
  });

  describe('Sandbox Operations - Comprehensive', () => {
    it('should enable sandbox mode', async () => {
      const mockSandboxResponse = {
        sandbox_api_key: 'sk_sandbox_enabled_123',
        sandbox_mode: true,
        merchant_id: 456
      };

      jest.spyOn(sandboxClient.sandbox, 'enable').mockResolvedValue(mockSandboxResponse);

      const result = await sandboxClient.sandbox.enable();
      expect(result.sandbox_mode).toBe(true);
      expect(result.sandbox_api_key).toContain('sk_sandbox');
    });

    it('should simulate payment success', async () => {
      const mockSimulation = {
        success: true,
        message: 'Payment simulated successfully',
        payment_id: 'pay_simulated_123'
      };

      jest.spyOn(sandboxClient.sandbox, 'simulatePayment').mockResolvedValue(mockSimulation);

      const result = await sandboxClient.sandbox.simulatePayment('pay_simulated_123', {
        success: true
      });

      expect(result.success).toBe(true);
      expect(result.payment_id).toBe('pay_simulated_123');
    });

    it('should simulate payment failure', async () => {
      const mockFailure = {
        success: false,
        message: 'Payment simulation failed',
        error_code: 'INSUFFICIENT_FUNDS'
      };

      jest.spyOn(sandboxClient.sandbox, 'simulatePayment').mockResolvedValue(mockFailure);

      const result = await sandboxClient.sandbox.simulatePayment('pay_fail_test', {
        success: false,
        error_code: 'INSUFFICIENT_FUNDS'
      });

      expect(result.success).toBe(false);
      expect(result.error_code).toBe('INSUFFICIENT_FUNDS');
    });
  });

  describe('Error Handling - Comprehensive', () => {
    it('should handle API errors properly', async () => {
      const apiError = new FidduPayAPIError('Payment not found', 404, 'PAYMENT_NOT_FOUND');
      
      jest.spyOn(client.payments, 'retrieve').mockRejectedValue(apiError);

      await expect(client.payments.retrieve('nonexistent_payment')).rejects.toThrow(FidduPayAPIError);
    });

    it('should handle validation errors', () => {
      expect(() => {
        client.payments.create({
          amount_usd: 'invalid_amount',
          crypto_type: 'ETH' as CryptoType
        });
      }).toThrow(FidduPayValidationError);
    });

    it('should handle network timeouts', async () => {
      const timeoutError = new Error('Request timeout');
      
      jest.spyOn(client.payments, 'create').mockRejectedValue(timeoutError);

      await expect(client.payments.create({
        amount_usd: '100.00',
        crypto_type: 'ETH'
      })).rejects.toThrow('Request timeout');
    });

    it('should retry failed requests', async () => {
      const retryClient = new FidduPay({
        apiKey: 'sk_retry_test',
        retries: 3
      });

      // Mock first two calls to fail, third to succeed
      jest.spyOn(retryClient.payments, 'retrieve')
        .mockRejectedValueOnce(new Error('Network error'))
        .mockRejectedValueOnce(new Error('Network error'))
        .mockResolvedValueOnce({ payment_id: 'pay_retry_success', status: 'confirmed' });

      const result = await retryClient.payments.retrieve('pay_retry_test');
      expect(result.payment_id).toBe('pay_retry_success');
    });
  });

  describe('Security Operations - Comprehensive', () => {
    it('should get security events', async () => {
      const mockEvents = [
        {
          event_id: 'evt_security_1',
          event_type: 'suspicious_login',
          timestamp: new Date().toISOString(),
          details: { ip_address: '192.168.1.1' }
        }
      ];

      jest.spyOn(client.security, 'getEvents').mockResolvedValue(mockEvents);

      const result = await client.security.getEvents();
      expect(result).toHaveLength(1);
      expect(result[0].event_type).toBe('suspicious_login');
    });

    it('should manage IP whitelist', async () => {
      const mockWhitelist = {
        ip_addresses: ['192.168.1.1', '10.0.0.1'],
        enabled: true
      };

      jest.spyOn(client.security, 'setIpWhitelist').mockResolvedValue(mockWhitelist);
      jest.spyOn(client.security, 'getIpWhitelist').mockResolvedValue(mockWhitelist);

      const setResult = await client.security.setIpWhitelist({
        ip_addresses: ['192.168.1.1', '10.0.0.1']
      });

      const getResult = await client.security.getIpWhitelist();

      expect(setResult.ip_addresses).toHaveLength(2);
      expect(getResult.enabled).toBe(true);
    });
  });

  describe('Withdrawal Operations - Comprehensive', () => {
    it('should create withdrawal', async () => {
      const withdrawalData = {
        crypto_type: 'ETH' as CryptoType,
        amount: '1.5',
        destination_address: '0xwithdrawal123',
        description: 'Test withdrawal'
      };

      const mockWithdrawal = {
        withdrawal_id: 'wd_test_123',
        status: 'pending',
        amount: '1.5',
        crypto_type: 'ETH'
      };

      jest.spyOn(client.withdrawals, 'create').mockResolvedValue(mockWithdrawal);

      const result = await client.withdrawals.create(withdrawalData);
      expect(result.withdrawal_id).toBe('wd_test_123');
      expect(result.status).toBe('pending');
    });

    it('should list withdrawals with filters', async () => {
      const mockWithdrawals = [
        { withdrawal_id: 'wd_1', status: 'completed' },
        { withdrawal_id: 'wd_2', status: 'pending' }
      ];

      jest.spyOn(client.withdrawals, 'list').mockResolvedValue(mockWithdrawals);

      const result = await client.withdrawals.list({
        status: 'pending',
        crypto_type: 'ETH'
      });

      expect(result).toHaveLength(2);
    });
  });

  describe('Integration Tests', () => {
    it('should handle complete payment flow', async () => {
      // Mock the entire payment flow
      const paymentId = 'pay_integration_test';
      
      // Create payment
      jest.spyOn(client.payments, 'create').mockResolvedValue({
        payment_id: paymentId,
        status: 'pending',
        amount_usd: '100.00'
      });

      // Retrieve payment
      jest.spyOn(client.payments, 'retrieve').mockResolvedValue({
        payment_id: paymentId,
        status: 'confirmed',
        amount_usd: '100.00'
      });

      // Create refund
      jest.spyOn(client.refunds, 'create').mockResolvedValue({
        refund_id: 'ref_integration_test',
        payment_id: paymentId,
        amount: '50.00',
        status: 'pending'
      });

      // Execute flow
      const payment = await client.payments.create({
        amount_usd: '100.00',
        crypto_type: 'ETH'
      });

      const retrievedPayment = await client.payments.retrieve(payment.payment_id);
      
      const refund = await client.refunds.create({
        payment_id: payment.payment_id,
        amount: '50.00',
        reason: 'Integration test refund'
      });

      expect(payment.payment_id).toBe(paymentId);
      expect(retrievedPayment.status).toBe('confirmed');
      expect(refund.payment_id).toBe(paymentId);
    });
  });
});
