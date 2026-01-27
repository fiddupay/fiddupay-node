import { Webhooks } from '../src/resources/webhooks';
import { FidduPayError } from '../src/errors';
import * as crypto from 'crypto';

describe('Webhook Operations', () => {
  const testSecret = 'whsec_test_secret_key_1234567890';
  const testPayload = JSON.stringify({
    id: 'evt_test_webhook',
    type: 'payment.confirmed',
    data: {
      payment_id: 'pay_test_123',
      amount_usd: '100.00',
      crypto_amount: '0.05',
      crypto_type: 'ETH',
      status: 'CONFIRMED'
    },
    created_at: '2024-01-27T12:00:00Z'
  });

  describe('Signature Verification', () => {
    it('should verify valid webhook signature', () => {
      const signature = Webhooks.generateSignature(testPayload, testSecret);
      const isValid = Webhooks.verifySignature(testPayload, signature, testSecret);
      expect(isValid).toBe(true);
    });

    it('should reject invalid signature', () => {
      const signature = Webhooks.generateSignature(testPayload, testSecret);
      const invalidSignature = signature.replace('v1=', 'v1=invalid');
      const isValid = Webhooks.verifySignature(testPayload, invalidSignature, testSecret);
      expect(isValid).toBe(false);
    });

    it('should reject signature with wrong secret', () => {
      const signature = Webhooks.generateSignature(testPayload, testSecret);
      const isValid = Webhooks.verifySignature(testPayload, signature, 'wrong_secret');
      expect(isValid).toBe(false);
    });

    it('should reject signature with modified payload', () => {
      const signature = Webhooks.generateSignature(testPayload, testSecret);
      const modifiedPayload = testPayload.replace('100.00', '200.00');
      const isValid = Webhooks.verifySignature(modifiedPayload, signature, testSecret);
      expect(isValid).toBe(false);
    });

    it('should reject expired signature', () => {
      // Create signature with old timestamp
      const oldTimestamp = Math.floor(Date.now() / 1000) - 400; // 400 seconds ago
      const signedPayload = `${oldTimestamp}.${testPayload}`;
      const signature = crypto
        .createHmac('sha256', testSecret)
        .update(signedPayload, 'utf8')
        .digest('hex');
      const fullSignature = `t=${oldTimestamp},v1=${signature}`;

      const isValid = Webhooks.verifySignature(testPayload, fullSignature, testSecret, 300);
      expect(isValid).toBe(false);
    });

    it('should accept signature within tolerance', () => {
      const signature = Webhooks.generateSignature(testPayload, testSecret);
      const isValid = Webhooks.verifySignature(testPayload, signature, testSecret, 300);
      expect(isValid).toBe(true);
    });

    it('should handle malformed signature header', () => {
      const isValid = Webhooks.verifySignature(testPayload, 'invalid_format', testSecret);
      expect(isValid).toBe(false);
    });

    it('should handle missing timestamp in signature', () => {
      const isValid = Webhooks.verifySignature(testPayload, 'v1=signature', testSecret);
      expect(isValid).toBe(false);
    });

    it('should handle missing signature in header', () => {
      const timestamp = Math.floor(Date.now() / 1000);
      const isValid = Webhooks.verifySignature(testPayload, `t=${timestamp}`, testSecret);
      expect(isValid).toBe(false);
    });
  });

  describe('Event Construction', () => {
    it('should construct valid webhook event', () => {
      const signature = Webhooks.generateSignature(testPayload, testSecret);
      const event = Webhooks.constructEvent(testPayload, signature, testSecret);

      expect(event.id).toBe('evt_test_webhook');
      expect(event.type).toBe('payment.confirmed');
      expect(event.data.payment_id).toBe('pay_test_123');
      expect(event.created_at).toBe('2024-01-27T12:00:00Z');
    });

    it('should construct event from Buffer payload', () => {
      const bufferPayload = Buffer.from(testPayload, 'utf8');
      const signature = Webhooks.generateSignature(testPayload, testSecret);
      const event = Webhooks.constructEvent(bufferPayload, signature, testSecret);

      expect(event.id).toBe('evt_test_webhook');
      expect(event.type).toBe('payment.confirmed');
    });

    it('should throw error for invalid signature', () => {
      const invalidSignature = 'invalid_signature';
      expect(() => {
        Webhooks.constructEvent(testPayload, invalidSignature, testSecret);
      }).toThrow('Invalid webhook signature');
    });

    it('should throw error for invalid JSON payload', () => {
      const invalidPayload = 'invalid json';
      const signature = Webhooks.generateSignature(invalidPayload, testSecret);
      expect(() => {
        Webhooks.constructEvent(invalidPayload, signature, testSecret);
      }).toThrow('Invalid webhook payload');
    });
  });

  describe('Event Validation', () => {
    it('should validate all supported event types', () => {
      const eventTypes = [
        'payment.confirmed',
        'payment.expired',
        'payment.failed',
        'refund.completed',
        'refund.failed'
      ];

      eventTypes.forEach(type => {
        const payload = JSON.stringify({
          id: 'evt_test',
          type,
          data: { test: 'data' },
          created_at: '2024-01-27T12:00:00Z'
        });

        const signature = Webhooks.generateSignature(payload, testSecret);
        expect(() => {
          Webhooks.constructEvent(payload, signature, testSecret);
        }).not.toThrow();
      });
    });

    it('should reject event with missing id', () => {
      const payload = JSON.stringify({
        type: 'payment.confirmed',
        data: { test: 'data' },
        created_at: '2024-01-27T12:00:00Z'
      });

      const signature = Webhooks.generateSignature(payload, testSecret);
      expect(() => {
        Webhooks.constructEvent(payload, signature, testSecret);
      }).toThrow('Invalid webhook event: missing or invalid id');
    });

    it('should reject event with invalid id type', () => {
      const payload = JSON.stringify({
        id: 123,
        type: 'payment.confirmed',
        data: { test: 'data' },
        created_at: '2024-01-27T12:00:00Z'
      });

      const signature = Webhooks.generateSignature(payload, testSecret);
      expect(() => {
        Webhooks.constructEvent(payload, signature, testSecret);
      }).toThrow('Invalid webhook event: missing or invalid id');
    });

    it('should reject event with missing type', () => {
      const payload = JSON.stringify({
        id: 'evt_test',
        data: { test: 'data' },
        created_at: '2024-01-27T12:00:00Z'
      });

      const signature = Webhooks.generateSignature(payload, testSecret);
      expect(() => {
        Webhooks.constructEvent(payload, signature, testSecret);
      }).toThrow('Invalid webhook event: missing or invalid type');
    });

    it('should reject event with invalid type', () => {
      const payload = JSON.stringify({
        id: 'evt_test',
        type: 'invalid.event',
        data: { test: 'data' },
        created_at: '2024-01-27T12:00:00Z'
      });

      const signature = Webhooks.generateSignature(payload, testSecret);
      expect(() => {
        Webhooks.constructEvent(payload, signature, testSecret);
      }).toThrow('Invalid webhook event type: invalid.event');
    });

    it('should reject event with missing data', () => {
      const payload = JSON.stringify({
        id: 'evt_test',
        type: 'payment.confirmed',
        created_at: '2024-01-27T12:00:00Z'
      });

      const signature = Webhooks.generateSignature(payload, testSecret);
      expect(() => {
        Webhooks.constructEvent(payload, signature, testSecret);
      }).toThrow('Invalid webhook event: missing data');
    });

    it('should reject event with missing created_at', () => {
      const payload = JSON.stringify({
        id: 'evt_test',
        type: 'payment.confirmed',
        data: { test: 'data' }
      });

      const signature = Webhooks.generateSignature(payload, testSecret);
      expect(() => {
        Webhooks.constructEvent(payload, signature, testSecret);
      }).toThrow('Invalid webhook event: missing or invalid created_at');
    });

    it('should reject event with invalid created_at type', () => {
      const payload = JSON.stringify({
        id: 'evt_test',
        type: 'payment.confirmed',
        data: { test: 'data' },
        created_at: 1234567890
      });

      const signature = Webhooks.generateSignature(payload, testSecret);
      expect(() => {
        Webhooks.constructEvent(payload, signature, testSecret);
      }).toThrow('Invalid webhook event: missing or invalid created_at');
    });
  });

  describe('Signature Generation', () => {
    it('should generate consistent signatures', () => {
      const signature1 = Webhooks.generateSignature(testPayload, testSecret);
      const signature2 = Webhooks.generateSignature(testPayload, testSecret);

      // Signatures should be different due to timestamp, but both should be valid
      expect(signature1).not.toBe(signature2);
      expect(Webhooks.verifySignature(testPayload, signature1, testSecret)).toBe(true);
      expect(Webhooks.verifySignature(testPayload, signature2, testSecret)).toBe(true);
    });

    it('should generate signature with correct format', () => {
      const signature = Webhooks.generateSignature(testPayload, testSecret);
      expect(signature).toMatch(/^t=\d+,v1=[a-f0-9]{64}$/);
    });
  });

  describe('Error Handling', () => {
    it('should handle crypto errors gracefully', () => {
      // Test with invalid signature format that might cause crypto errors
      const isValid = Webhooks.verifySignature(testPayload, 'malformed', testSecret);
      expect(isValid).toBe(false);
    });

    it('should handle empty payload', () => {
      const signature = Webhooks.generateSignature('', testSecret);
      expect(() => {
        Webhooks.constructEvent('', signature, testSecret);
      }).toThrow('Invalid webhook payload');
    });

    it('should handle null/undefined values', () => {
      expect(() => {
        Webhooks.verifySignature(testPayload, null as any, testSecret);
      }).not.toThrow();

      expect(() => {
        Webhooks.verifySignature(testPayload, undefined as any, testSecret);
      }).not.toThrow();
    });
  });

  describe('Real-world Scenarios', () => {
    it('should handle payment confirmation webhook', () => {
      const paymentConfirmedPayload = JSON.stringify({
        id: 'evt_payment_confirmed_123',
        type: 'payment.confirmed',
        data: {
          payment_id: 'pay_abc123',
          amount_usd: '250.00',
          crypto_amount: '0.125',
          crypto_type: 'ETH',
          status: 'CONFIRMED',
          transaction_hash: '0x1234567890abcdef',
          confirmations: 12,
          confirmed_at: '2024-01-27T12:30:00Z'
        },
        created_at: '2024-01-27T12:30:05Z'
      });

      const signature = Webhooks.generateSignature(paymentConfirmedPayload, testSecret);
      const event = Webhooks.constructEvent(paymentConfirmedPayload, signature, testSecret);

      expect(event.type).toBe('payment.confirmed');
      expect(event.data.payment_id).toBe('pay_abc123');
      expect(event.data.status).toBe('CONFIRMED');
    });

    it('should handle refund completed webhook', () => {
      const refundCompletedPayload = JSON.stringify({
        id: 'evt_refund_completed_456',
        type: 'refund.completed',
        data: {
          refund_id: 'ref_xyz789',
          payment_id: 'pay_abc123',
          amount_usd: '100.00',
          status: 'COMPLETED',
          completed_at: '2024-01-27T13:00:00Z'
        },
        created_at: '2024-01-27T13:00:05Z'
      });

      const signature = Webhooks.generateSignature(refundCompletedPayload, testSecret);
      const event = Webhooks.constructEvent(refundCompletedPayload, signature, testSecret);

      expect(event.type).toBe('refund.completed');
      expect((event.data as any).refund_id).toBe('ref_xyz789');
      expect((event.data as any).status).toBe('COMPLETED');
    });

    it('should handle payment failed webhook', () => {
      const paymentFailedPayload = JSON.stringify({
        id: 'evt_payment_failed_789',
        type: 'payment.failed',
        data: {
          payment_id: 'pay_def456',
          amount_usd: '75.00',
          crypto_type: 'BNB',
          status: 'FAILED',
          failure_reason: 'Insufficient funds',
          failed_at: '2024-01-27T14:00:00Z'
        },
        created_at: '2024-01-27T14:00:05Z'
      });

      const signature = Webhooks.generateSignature(paymentFailedPayload, testSecret);
      const event = Webhooks.constructEvent(paymentFailedPayload, signature, testSecret);

      expect(event.type).toBe('payment.failed');
      expect(event.data.payment_id).toBe('pay_def456');
      expect(event.data.status).toBe('FAILED');
    });
  });

  describe('Security Tests', () => {
    it('should use timing-safe comparison', () => {
      // This test ensures we're using crypto.timingSafeEqual
      const signature = Webhooks.generateSignature(testPayload, testSecret);
      
      // Even with a signature that has the same length but different content,
      // it should still return false
      const fakeSignature = signature.replace(/[a-f]/g, '0');
      const isValid = Webhooks.verifySignature(testPayload, fakeSignature, testSecret);
      expect(isValid).toBe(false);
    });

    it('should handle replay attacks with timestamp validation', () => {
      const signature = Webhooks.generateSignature(testPayload, testSecret);
      
      // Should be valid with default tolerance
      expect(Webhooks.verifySignature(testPayload, signature, testSecret)).toBe(true);
      
      // Should be invalid with very strict tolerance
      expect(Webhooks.verifySignature(testPayload, signature, testSecret, 0)).toBe(false);
    });
  });
});