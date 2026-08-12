import { Webhooks } from '../src/resources/webhooks';
import { FidduPayError } from '../src/errors';

describe('Webhooks', () => {
  const secret = 'whsec_test123';
  const payload = JSON.stringify({
    id: 'evt_test123',
    type: 'payment.confirmed',
    data: {
      payment_id: 'pay_test123',
      status: 'CONFIRMED'
    },
    created_at: '2026-01-25T12:00:00Z'
  });

  describe('generateSignature', () => {
    it('should generate valid signature', () => {
      const signature = Webhooks.generateSignature(payload, secret);
      expect(signature).toMatch(/^t=\d+,v1=[a-f0-9]{64}$/);
    });
  });

  describe('verifySignature', () => {
    it('should verify valid signature', () => {
      const signature = Webhooks.generateSignature(payload, secret);
      const isValid = Webhooks.verifySignature(payload, signature, secret);
      expect(isValid).toBe(true);
    });

    it('should reject invalid signature', () => {
      const invalidSignature = 't=1234567890,v1=invalid';
      expect(() => {
        Webhooks.verifySignature(payload, invalidSignature, secret);
      }).toThrow(FidduPayError);
    });

    it('should reject expired signature', () => {
      const oldTimestamp = Math.floor(Date.now() / 1000) - 400; // 400 seconds ago
      const signature = `t=${oldTimestamp},v1=somehash`;
      expect(() => {
        Webhooks.verifySignature(payload, signature, secret, 300);
      }).toThrow(FidduPayError);
    });
  });

  describe('constructEvent', () => {
    it('should construct valid webhook event', () => {
      const signature = Webhooks.generateSignature(payload, secret);
      const event = Webhooks.constructEvent(payload, signature, secret);
      
      expect(event.id).toBe('evt_test123');
      expect(event.type).toBe('payment.confirmed');
      expect(event.data).toBeDefined();
    });

    it('should construct valid direct WebhookPayload with sandbox_mode', () => {
      const directPayload = JSON.stringify({
        event_type: 'payment.confirmed',
        payment_id: 'pay_test_999',
        merchant_id: 22,
        status: 'Confirmed',
        amount: '100.00',
        crypto_type: 'USDT_SPL',
        transaction_hash: '0xhash123',
        customer_external_id: 'cust_101',
        timestamp: 1774900000,
        sandbox_mode: true
      });

      const signature = Webhooks.generateSignature(directPayload, secret);
      const event = Webhooks.constructEvent(directPayload, signature, secret) as any;

      expect(event.event_type).toBe('payment.confirmed');
      expect(event.payment_id).toBe('pay_test_999');
      expect(event.sandbox_mode).toBe(true);
      expect(event.amount).toBe('100.00');
    });

    it('should throw error for invalid signature', () => {
      const invalidSignature = 't=1234567890,v1=invalid';
      
      expect(() => {
        Webhooks.constructEvent(payload, invalidSignature, secret);
      }).toThrow(FidduPayError);
    });

    it('should throw error for invalid payload', () => {
      const invalidPayload = 'invalid json';
      const signature = Webhooks.generateSignature(invalidPayload, secret);
      
      expect(() => {
        Webhooks.constructEvent(invalidPayload, signature, secret);
      }).toThrow(FidduPayError);
    });
  });

  describe('listDeliveries and retryDelivery', () => {
    let mockClient: any;
    let webhooks: Webhooks;

    beforeEach(() => {
      mockClient = {
        get: jest.fn().mockResolvedValue({ deliveries: [], status: 'success' }),
        post: jest.fn().mockResolvedValue({ status: 'success', message: 'Webhook delivery re-queued for retry' })
      };
      webhooks = new Webhooks(mockClient);
    });

    it('should call get on listDeliveries', async () => {
      const result = await webhooks.listDeliveries({ limit: 10, offset: 0 });
      expect(mockClient.get).toHaveBeenCalledWith('/api/v1/merchants/webhooks/deliveries?limit=10&offset=0', undefined);
      expect(result.status).toBe('success');
    });

    it('should call post on retryDelivery', async () => {
      const result = await webhooks.retryDelivery(180);
      expect(mockClient.post).toHaveBeenCalledWith('/api/v1/merchants/webhooks/deliveries/180/retry', {}, undefined);
      expect(result.status).toBe('success');
    });

    it('should throw error if called without HttpClient', async () => {
      const uninitializedWebhooks = new Webhooks();
      await expect(uninitializedWebhooks.listDeliveries()).rejects.toThrow(FidduPayError);
      await expect(uninitializedWebhooks.retryDelivery(180)).rejects.toThrow(FidduPayError);
    });
  });
});
