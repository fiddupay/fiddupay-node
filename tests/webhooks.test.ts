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
      const isValid = Webhooks.verifySignature(payload, invalidSignature, secret);
      expect(isValid).toBe(false);
    });

    it('should reject expired signature', () => {
      const oldTimestamp = Math.floor(Date.now() / 1000) - 400; // 400 seconds ago
      const signature = `t=${oldTimestamp},v1=somehash`;
      const isValid = Webhooks.verifySignature(payload, signature, secret, 300);
      expect(isValid).toBe(false);
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
});
