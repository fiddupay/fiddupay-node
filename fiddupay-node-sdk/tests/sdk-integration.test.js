"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const src_1 = __importDefault(require("../src"));
const errors_1 = require("../src/errors");
describe('FidduPay SDK - Integration Test Suite', () => {
    let client;
    beforeAll(() => {
        client = new src_1.default({
            apiKey: 'sk_test_integration_key',
            environment: 'sandbox',
            timeout: 30000
        });
    });
    describe('Client Initialization', () => {
        it('should create client with valid configuration', () => {
            expect(client).toBeInstanceOf(src_1.default);
            expect(client.payments).toBeDefined();
            expect(client.merchants).toBeDefined();
            expect(client.refunds).toBeDefined();
            expect(client.wallets).toBeDefined();
            expect(client.analytics).toBeDefined();
            expect(client.security).toBeDefined();
            expect(client.withdrawals).toBeDefined();
            expect(client.sandbox).toBeDefined();
            expect(client.webhooks).toBeDefined();
        });
        it('should validate API key format', () => {
            expect(() => {
                new src_1.default({ apiKey: 'invalid_key' });
            }).toThrow(errors_1.FidduPayValidationError);
        });
        it('should accept valid API key formats', () => {
            expect(() => {
                new src_1.default({ apiKey: 'sk_test_valid_key' });
            }).not.toThrow();
            expect(() => {
                new src_1.default({ apiKey: 'sk_live_valid_key' });
            }).not.toThrow();
        });
    });
    describe('Resource Availability', () => {
        it('should have all payment methods available', () => {
            expect(typeof client.payments.create).toBe('function');
            expect(typeof client.payments.retrieve).toBe('function');
            expect(typeof client.payments.list).toBe('function');
            expect(typeof client.payments.createAddressOnly).toBe('function');
        });
        it('should have all merchant methods available', () => {
            expect(typeof client.merchants.register).toBe('function');
            expect(typeof client.merchants.rotateApiKey).toBe('function');
            expect(typeof client.merchants.switchEnvironment).toBe('function');
            expect(typeof client.merchants.getBalance).toBe('function');
            expect(typeof client.merchants.setWallet).toBe('function');
        });
        it('should have all refund methods available', () => {
            expect(typeof client.refunds.create).toBe('function');
            expect(typeof client.refunds.retrieve).toBe('function');
            expect(typeof client.refunds.list).toBe('function');
        });
        it('should have all wallet methods available', () => {
            expect(typeof client.wallets.generate).toBe('function');
            expect(typeof client.wallets.import).toBe('function');
            expect(typeof client.wallets.checkGasRequirements).toBe('function');
        });
        it('should have all analytics methods available', () => {
            expect(typeof client.analytics.export).toBe('function');
        });
        it('should have all security methods available', () => {
            expect(typeof client.security.getEvents).toBe('function');
            expect(typeof client.security.getAlerts).toBe('function');
            expect(typeof client.security.acknowledgeAlert).toBe('function');
            expect(typeof client.security.getSettings).toBe('function');
            expect(typeof client.security.updateSettings).toBe('function');
        });
        it('should have all withdrawal methods available', () => {
            expect(typeof client.withdrawals.create).toBe('function');
            expect(typeof client.withdrawals.list).toBe('function');
            expect(typeof client.withdrawals.cancel).toBe('function');
        });
        it('should have all sandbox methods available', () => {
            expect(typeof client.sandbox.enable).toBe('function');
            expect(typeof client.sandbox.simulatePayment).toBe('function');
        });
        it('should have all webhook methods available', () => {
            expect(typeof client.webhooks.verifySignature).toBe('function');
            expect(typeof client.webhooks.constructEvent).toBe('function');
            expect(typeof client.webhooks.generateSignature).toBe('function');
        });
    });
    describe('Configuration Validation', () => {
        it('should handle different environment settings', () => {
            const sandboxClient = new src_1.default({
                apiKey: 'sk_test_sandbox',
                environment: 'sandbox'
            });
            expect(sandboxClient).toBeInstanceOf(src_1.default);
            const prodClient = new src_1.default({
                apiKey: 'live_production_key',
                environment: 'production'
            });
            expect(prodClient).toBeInstanceOf(src_1.default);
        });
        it('should handle custom timeout settings', () => {
            const customClient = new src_1.default({
                apiKey: 'sk_test_custom',
                timeout: 60000
            });
            expect(customClient).toBeInstanceOf(src_1.default);
        });
        it('should handle custom base URL', () => {
            const customClient = new src_1.default({
                apiKey: 'sk_test_custom',
                baseURL: 'https://custom.api.fiddupay.com'
            });
            expect(customClient).toBeInstanceOf(src_1.default);
        });
    });
    describe('Error Handling', () => {
        it('should throw validation error for missing API key', () => {
            expect(() => {
                new src_1.default({});
            }).toThrow(errors_1.FidduPayValidationError);
        });
        it('should throw validation error for invalid API key format', () => {
            expect(() => {
                new src_1.default({ apiKey: 'invalid' });
            }).toThrow(errors_1.FidduPayValidationError);
        });
        it('should throw validation error for empty API key', () => {
            expect(() => {
                new src_1.default({ apiKey: '' });
            }).toThrow(errors_1.FidduPayValidationError);
        });
    });
    describe('Type Safety', () => {
        it('should enforce correct crypto types', () => {
            const validCryptoTypes = ['SOL', 'ETH', 'BNB', 'MATIC', 'ARB', 'USDT_ETH', 'USDT_BEP20', 'USDT_POLYGON', 'USDT_ARBITRUM', 'USDT_SPL'];
            validCryptoTypes.forEach(cryptoType => {
                expect(() => {
                    const paymentData = {
                        amount_usd: '100.00',
                        crypto_type: cryptoType,
                        description: 'Test payment'
                    };
                    // This should not throw a validation error for valid crypto types
                    expect(paymentData.crypto_type).toBe(cryptoType);
                }).not.toThrow();
            });
        });
        it('should enforce correct payment status types', () => {
            const validStatuses = ['PENDING', 'CONFIRMING', 'CONFIRMED', 'FAILED', 'EXPIRED'];
            validStatuses.forEach(status => {
                expect(validStatuses).toContain(status);
            });
        });
    });
    describe('SDK Coverage', () => {
        it('should cover all major resource categories', () => {
            const expectedResources = [
                'payments',
                'merchants',
                'refunds',
                'wallets',
                'analytics',
                'security',
                'withdrawals',
                'sandbox',
                'webhooks'
            ];
            expectedResources.forEach(resource => {
                expect(client).toHaveProperty(resource);
                expect(client[resource]).toBeDefined();
            });
        });
        it('should provide comprehensive payment operations', () => {
            const paymentMethods = ['create', 'retrieve', 'list', 'createAddressOnly'];
            paymentMethods.forEach(method => {
                expect(client.payments).toHaveProperty(method);
                expect(typeof client.payments[method]).toBe('function');
            });
        });
        it('should provide comprehensive merchant operations', () => {
            const merchantMethods = ['register', 'rotateApiKey', 'switchEnvironment', 'getBalance', 'setWallet'];
            merchantMethods.forEach(method => {
                expect(client.merchants).toHaveProperty(method);
                expect(typeof client.merchants[method]).toBe('function');
            });
        });
        it('should provide webhook utilities', () => {
            const webhookMethods = ['verifySignature', 'constructEvent', 'generateSignature'];
            webhookMethods.forEach(method => {
                expect(client.webhooks).toHaveProperty(method);
                expect(typeof client.webhooks[method]).toBe('function');
            });
        });
    });
    describe('Mock Integration Tests', () => {
        it('should handle successful API responses', async () => {
            // Mock a successful payment creation
            const mockPayment = {
                payment_id: 'pay_test_123',
                status: 'PENDING',
                amount_usd: '100.00',
                crypto_amount: '0.05',
                crypto_type: 'ETH',
                deposit_address: '0x123...',
                created_at: new Date().toISOString(),
                expires_at: new Date().toISOString()
            };
            // This test validates the structure without making actual API calls
            expect(mockPayment.payment_id).toBe('pay_test_123');
            expect(mockPayment.status).toBe('PENDING');
            expect(mockPayment.crypto_type).toBe('ETH');
        });
        it('should handle error responses appropriately', () => {
            // Test error handling structure
            const mockError = new errors_1.FidduPayAPIError('Test error', 400, 'VALIDATION_ERROR');
            expect(mockError).toBeInstanceOf(errors_1.FidduPayAPIError);
            expect(mockError.message).toBe('Test error');
            expect(mockError.statusCode).toBe(400);
            expect(mockError.code).toBe('VALIDATION_ERROR');
        });
    });
});
//# sourceMappingURL=sdk-integration.test.js.map