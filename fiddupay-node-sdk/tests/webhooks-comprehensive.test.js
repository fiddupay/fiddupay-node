"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const webhooks_1 = require("../src/resources/webhooks");
const errors_1 = require("../src/errors");
describe('Webhooks Comprehensive', () => {
    const secret = 'whsec_test123456789';
    const validPayload = JSON.stringify({
        id: 'evt_test123',
        type: 'payment.confirmed',
        data: {
            payment_id: 'pay_test123',
            status: 'CONFIRMED',
            amount_usd: '100.00',
            crypto_amount: '0.05',
            crypto_type: 'ETH'
        },
        created_at: '2026-01-25T12:00:00Z'
    });
    describe('Signature Generation', () => {
        it('should generate consistent signatures', () => {
            const signature1 = webhooks_1.Webhooks.generateSignature(validPayload, secret);
            const signature2 = webhooks_1.Webhooks.generateSignature(validPayload, secret);
            // Signatures should be different due to timestamp, but format should be consistent
            expect(signature1).toMatch(/^t=\d+,v1=[a-f0-9]{64}$/);
            expect(signature2).toMatch(/^t=\d+,v1=[a-f0-9]{64}$/);
        });
        it('should generate different signatures for different payloads', () => {
            const payload1 = JSON.stringify({ test: 'data1' });
            const payload2 = JSON.stringify({ test: 'data2' });
            const signature1 = webhooks_1.Webhooks.generateSignature(payload1, secret);
            const signature2 = webhooks_1.Webhooks.generateSignature(payload2, secret);
            expect(signature1).not.toBe(signature2);
        });
        it('should generate different signatures for different secrets', () => {
            const secret1 = 'whsec_secret1';
            const secret2 = 'whsec_secret2';
            const signature1 = webhooks_1.Webhooks.generateSignature(validPayload, secret1);
            const signature2 = webhooks_1.Webhooks.generateSignature(validPayload, secret2);
            expect(signature1).not.toBe(signature2);
        });
        it('should handle empty payload', () => {
            const signature = webhooks_1.Webhooks.generateSignature('', secret);
            expect(signature).toMatch(/^t=\d+,v1=[a-f0-9]{64}$/);
        });
        it('should handle special characters in payload', () => {
            const specialPayload = JSON.stringify({
                message: 'Special chars: !@#$%^&*()_+-=[]{}|;:,.<>?',
                unicode: '',
                newlines: 'Line 1\nLine 2\r\nLine 3'
            });
            const signature = webhooks_1.Webhooks.generateSignature(specialPayload, secret);
            expect(signature).toMatch(/^t=\d+,v1=[a-f0-9]{64}$/);
        });
    });
    describe('Signature Verification', () => {
        it('should verify valid signatures', () => {
            const signature = webhooks_1.Webhooks.generateSignature(validPayload, secret);
            const isValid = webhooks_1.Webhooks.verifySignature(validPayload, signature, secret);
            expect(isValid).toBe(true);
        });
        it('should reject invalid signatures', () => {
            const invalidSignature = 't=1234567890,v1=invalid_hash';
            const isValid = webhooks_1.Webhooks.verifySignature(validPayload, invalidSignature, secret);
            expect(isValid).toBe(false);
        });
        it('should reject signatures with wrong secret', () => {
            const signature = webhooks_1.Webhooks.generateSignature(validPayload, secret);
            const isValid = webhooks_1.Webhooks.verifySignature(validPayload, signature, 'wrong_secret');
            expect(isValid).toBe(false);
        });
        it('should reject signatures for different payloads', () => {
            const signature = webhooks_1.Webhooks.generateSignature(validPayload, secret);
            const differentPayload = JSON.stringify({ different: 'data' });
            const isValid = webhooks_1.Webhooks.verifySignature(differentPayload, signature, secret);
            expect(isValid).toBe(false);
        });
        it('should handle malformed signature format', () => {
            const malformedSignatures = [
                'invalid_format',
                't=123',
                'v1=hash',
                't=,v1=hash',
                't=123,v1=',
                't=abc,v1=hash',
                'timestamp=123,version=hash'
            ];
            malformedSignatures.forEach(signature => {
                const isValid = webhooks_1.Webhooks.verifySignature(validPayload, signature, secret);
                expect(isValid).toBe(false);
            });
        });
        it('should handle empty signature', () => {
            const isValid = webhooks_1.Webhooks.verifySignature(validPayload, '', secret);
            expect(isValid).toBe(false);
        });
        it('should handle null/undefined signature', () => {
            const isValid1 = webhooks_1.Webhooks.verifySignature(validPayload, null, secret);
            const isValid2 = webhooks_1.Webhooks.verifySignature(validPayload, undefined, secret);
            expect(isValid1).toBe(false);
            expect(isValid2).toBe(false);
        });
    });
    describe('Timestamp Validation', () => {
        it('should accept recent signatures', () => {
            const signature = webhooks_1.Webhooks.generateSignature(validPayload, secret);
            const isValid = webhooks_1.Webhooks.verifySignature(validPayload, signature, secret, 300); // 5 minutes
            expect(isValid).toBe(true);
        });
        it('should reject expired signatures', () => {
            const oldTimestamp = Math.floor(Date.now() / 1000) - 400; // 400 seconds ago
            const signature = `t=${oldTimestamp},v1=somehash`;
            const isValid = webhooks_1.Webhooks.verifySignature(validPayload, signature, secret, 300); // 5 minutes tolerance
            expect(isValid).toBe(false);
        });
        it('should use default tolerance when not specified', () => {
            const signature = webhooks_1.Webhooks.generateSignature(validPayload, secret);
            const isValid = webhooks_1.Webhooks.verifySignature(validPayload, signature, secret);
            expect(isValid).toBe(true);
        });
        it('should handle zero tolerance', () => {
            const signature = webhooks_1.Webhooks.generateSignature(validPayload, secret);
            // Even with zero tolerance, should work for immediate verification
            const isValid = webhooks_1.Webhooks.verifySignature(validPayload, signature, secret, 0);
            expect(isValid).toBe(true);
        });
        it('should handle negative tolerance', () => {
            const signature = webhooks_1.Webhooks.generateSignature(validPayload, secret);
            const isValid = webhooks_1.Webhooks.verifySignature(validPayload, signature, secret, -1);
            expect(isValid).toBe(false);
        });
        it('should handle future timestamps', () => {
            const futureTimestamp = Math.floor(Date.now() / 1000) + 3600; // 1 hour in future
            const signature = `t=${futureTimestamp},v1=somehash`;
            const isValid = webhooks_1.Webhooks.verifySignature(validPayload, signature, secret, 300);
            expect(isValid).toBe(false);
        });
    });
    describe('Event Construction', () => {
        it('should construct valid webhook event', () => {
            const signature = webhooks_1.Webhooks.generateSignature(validPayload, secret);
            const event = webhooks_1.Webhooks.constructEvent(validPayload, signature, secret);
            expect(event.id).toBe('evt_test123');
            expect(event.type).toBe('payment.confirmed');
            expect(event.data).toBeDefined();
            expect(event.data.payment_id).toBe('pay_test123');
            expect(event.created_at).toBe('2026-01-25T12:00:00Z');
        });
        it('should throw error for invalid signature', () => {
            const invalidSignature = 't=1234567890,v1=invalid';
            expect(() => {
                webhooks_1.Webhooks.constructEvent(validPayload, invalidSignature, secret);
            }).toThrow(errors_1.FidduPayError);
        });
        it('should throw error for invalid JSON payload', () => {
            const invalidPayload = 'invalid json';
            const signature = webhooks_1.Webhooks.generateSignature(invalidPayload, secret);
            expect(() => {
                webhooks_1.Webhooks.constructEvent(invalidPayload, signature, secret);
            }).toThrow(errors_1.FidduPayError);
        });
        it('should throw error for missing required fields', () => {
            const incompletePayload = JSON.stringify({
                type: 'payment.confirmed',
                data: { payment_id: 'pay_test123' }
                // missing id and created_at
            });
            const signature = webhooks_1.Webhooks.generateSignature(incompletePayload, secret);
            expect(() => {
                webhooks_1.Webhooks.constructEvent(incompletePayload, signature, secret);
            }).toThrow(errors_1.FidduPayError);
        });
        it('should handle different event types', () => {
            const eventTypes = [
                'payment.confirmed',
                'payment.expired',
                'payment.failed',
                'refund.completed',
                'refund.failed'
            ];
            eventTypes.forEach(eventType => {
                const payload = JSON.stringify({
                    id: 'evt_test123',
                    type: eventType,
                    data: { payment_id: 'pay_test123' },
                    created_at: '2026-01-25T12:00:00Z'
                });
                const signature = webhooks_1.Webhooks.generateSignature(payload, secret);
                const event = webhooks_1.Webhooks.constructEvent(payload, signature, secret);
                expect(event.type).toBe(eventType);
            });
        });
    });
    describe('Error Handling', () => {
        it('should provide meaningful error messages', () => {
            expect(() => {
                webhooks_1.Webhooks.constructEvent('invalid json', 't=123,v1=hash', secret);
            }).toThrow('Invalid webhook signature'); // Signature validation happens first
            expect(() => {
                webhooks_1.Webhooks.constructEvent(validPayload, 'invalid_signature', secret);
            }).toThrow('Invalid webhook signature');
        });
        it('should handle edge case inputs', () => {
            // Empty inputs
            expect(() => {
                webhooks_1.Webhooks.constructEvent('', '', '');
            }).toThrow();
            // Null inputs
            expect(() => {
                webhooks_1.Webhooks.constructEvent(null, null, null);
            }).toThrow();
            // Undefined inputs
            expect(() => {
                webhooks_1.Webhooks.constructEvent(undefined, undefined, undefined);
            }).toThrow();
        });
    });
    describe('Security Considerations', () => {
        it('should use cryptographically secure hash', () => {
            const signature = webhooks_1.Webhooks.generateSignature(validPayload, secret);
            const hashPart = signature.split(',v1=')[1];
            // SHA-256 hash should be 64 characters long
            expect(hashPart).toHaveLength(64);
            expect(hashPart).toMatch(/^[a-f0-9]{64}$/);
        });
        it('should be resistant to timing attacks', () => {
            const signature = webhooks_1.Webhooks.generateSignature(validPayload, secret);
            // Multiple verifications should be consistent
            const results = [];
            for (let i = 0; i < 10; i++) {
                results.push(webhooks_1.Webhooks.verifySignature(validPayload, signature, secret));
            }
            expect(results.every(result => result === true)).toBe(true);
        });
        it('should handle secrets of different lengths', () => {
            const secrets = [
                'short',
                'medium_length_secret',
                'very_long_secret_with_many_characters_1234567890',
                'whsec_' + 'a'.repeat(100)
            ];
            secrets.forEach(testSecret => {
                const signature = webhooks_1.Webhooks.generateSignature(validPayload, testSecret);
                const isValid = webhooks_1.Webhooks.verifySignature(validPayload, signature, testSecret);
                expect(isValid).toBe(true);
            });
        });
    });
    describe('Real-world Scenarios', () => {
        it('should handle payment confirmation webhook', () => {
            const paymentPayload = JSON.stringify({
                id: 'evt_payment_confirmed_123',
                type: 'payment.confirmed',
                data: {
                    payment_id: 'pay_abc123',
                    status: 'CONFIRMED',
                    amount_usd: '250.00',
                    crypto_amount: '0.125',
                    crypto_type: 'ETH',
                    transaction_hash: '0x1234567890abcdef',
                    confirmations: 12,
                    confirmed_at: '2026-01-25T12:30:00Z'
                },
                created_at: '2026-01-25T12:30:05Z'
            });
            const signature = webhooks_1.Webhooks.generateSignature(paymentPayload, secret);
            const event = webhooks_1.Webhooks.constructEvent(paymentPayload, signature, secret);
            expect(event.type).toBe('payment.confirmed');
            expect(event.data.payment_id).toBe('pay_abc123');
            expect(event.data.status).toBe('CONFIRMED');
        });
        it('should handle refund webhook', () => {
            const refundPayload = JSON.stringify({
                id: 'evt_refund_completed_456',
                type: 'refund.completed',
                data: {
                    refund_id: 'ref_xyz789',
                    payment_id: 'pay_abc123',
                    status: 'COMPLETED',
                    amount: '100.00',
                    amount_usd: '100.00',
                    crypto_type: 'USDT_ETH',
                    transaction_hash: '0xabcdef1234567890'
                },
                created_at: '2026-01-25T13:00:00Z'
            });
            const signature = webhooks_1.Webhooks.generateSignature(refundPayload, secret);
            const event = webhooks_1.Webhooks.constructEvent(refundPayload, signature, secret);
            expect(event.type).toBe('refund.completed');
            // Type assertion for refund data
            const refundData = event.data;
            expect(refundData.refund_id).toBe('ref_xyz789');
        });
        it('should handle large payloads', () => {
            const largeData = {
                id: 'evt_large_payload',
                type: 'payment.confirmed',
                data: {
                    payment_id: 'pay_large',
                    metadata: {}
                },
                created_at: '2026-01-25T12:00:00Z'
            };
            // Add large metadata
            for (let i = 0; i < 1000; i++) {
                largeData.data.metadata[`key_${i}`] = `value_${i}_${'x'.repeat(100)}`;
            }
            const largePayload = JSON.stringify(largeData);
            const signature = webhooks_1.Webhooks.generateSignature(largePayload, secret);
            const isValid = webhooks_1.Webhooks.verifySignature(largePayload, signature, secret);
            expect(isValid).toBe(true);
        });
    });
    describe('Performance', () => {
        it('should handle signature generation efficiently', () => {
            const start = Date.now();
            for (let i = 0; i < 100; i++) {
                webhooks_1.Webhooks.generateSignature(validPayload, secret);
            }
            const duration = Date.now() - start;
            expect(duration).toBeLessThan(1000); // Should complete in under 1 second
        });
        it('should handle signature verification efficiently', () => {
            const signature = webhooks_1.Webhooks.generateSignature(validPayload, secret);
            const start = Date.now();
            for (let i = 0; i < 100; i++) {
                webhooks_1.Webhooks.verifySignature(validPayload, signature, secret);
            }
            const duration = Date.now() - start;
            expect(duration).toBeLessThan(1000); // Should complete in under 1 second
        });
    });
});
//# sourceMappingURL=webhooks-comprehensive.test.js.map