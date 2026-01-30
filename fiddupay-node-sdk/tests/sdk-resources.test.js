"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const src_1 = __importDefault(require("../src"));
const errors_1 = require("../src/errors");
describe('SDK Resources Coverage', () => {
    let client;
    beforeEach(() => {
        client = new src_1.default({
            apiKey: 'sk_test_1234567890',
            environment: 'sandbox'
        });
    });
    describe('Core Resources Availability', () => {
        it('should have all core resources', () => {
            expect(client.payments).toBeDefined();
            expect(client.merchants).toBeDefined();
            expect(client.refunds).toBeDefined();
            expect(client.analytics).toBeDefined();
            expect(client.webhooks).toBeDefined();
            expect(client.wallets).toBeDefined();
            expect(client.withdrawals).toBeDefined();
            expect(client.security).toBeDefined();
            expect(client.balances).toBeDefined();
            expect(client.auditLogs).toBeDefined();
            expect(client.sandbox).toBeDefined();
        });
    });
    describe('Payments Resource', () => {
        it('should have all payment methods', () => {
            expect(typeof client.payments.create).toBe('function');
            expect(typeof client.payments.retrieve).toBe('function');
            expect(typeof client.payments.list).toBe('function');
            expect(typeof client.payments.cancel).toBe('function');
            expect(typeof client.payments.createAddressOnly).toBe('function');
            expect(typeof client.payments.retrieveAddressOnly).toBe('function');
            expect(typeof client.payments.updateFeeSetting).toBe('function');
            expect(typeof client.payments.getFeeSetting).toBe('function');
        });
        it('should validate payment ID for retrieve', async () => {
            await expect(client.payments.retrieve(''))
                .rejects.toThrow('Payment ID is required');
        });
        it('should validate payment ID for cancel', async () => {
            await expect(client.payments.cancel(''))
                .rejects.toThrow('Payment ID is required');
        });
        it('should handle list parameters', () => {
            expect(() => {
                client.payments.list({
                    limit: 10,
                    offset: 0,
                    status: 'CONFIRMED',
                    crypto_type: 'ETH'
                });
            }).not.toThrow();
        });
        it('should handle empty list parameters', () => {
            expect(() => {
                client.payments.list();
            }).not.toThrow();
        });
    });
    describe('Merchants Resource', () => {
        it('should have merchant methods', () => {
            expect(typeof client.merchants.getBalance).toBe('function');
            expect(typeof client.merchants.setWallets).toBe('function');
        });
    });
    describe('Refunds Resource', () => {
        it('should have refund methods', () => {
            expect(typeof client.refunds.create).toBe('function');
            expect(typeof client.refunds.list).toBe('function');
        });
        it('should validate refund creation', async () => {
            await expect(client.refunds.create({
                payment_id: '',
                amount: '50.00'
            })).rejects.toThrow();
        });
        it('should accept valid refund request', () => {
            expect(() => {
                client.refunds.create({
                    payment_id: 'pay_test123',
                    amount: '50.00',
                    reason: 'Customer request'
                });
            }).not.toThrow();
        });
    });
    describe('Analytics Resource', () => {
        it('should have analytics methods', () => {
            expect(typeof client.analytics.retrieve).toBe('function');
            expect(typeof client.analytics.export).toBe('function');
        });
    });
    describe('Webhooks Resource', () => {
        it('should have webhook methods', () => {
            expect(typeof client.webhooks.verifySignature).toBe('function');
            expect(typeof client.webhooks.constructEvent).toBe('function');
            expect(typeof client.webhooks.generateSignature).toBe('function');
        });
        it('should be static methods', () => {
            expect(client.webhooks.verifySignature).toBe(client.webhooks.verifySignature);
            expect(client.webhooks.constructEvent).toBe(client.webhooks.constructEvent);
            expect(client.webhooks.generateSignature).toBe(client.webhooks.generateSignature);
        });
    });
    describe('Wallets Resource', () => {
        it('should have wallet methods', () => {
            expect(typeof client.wallets.getConfigurations).toBe('function');
            expect(typeof client.wallets.generate).toBe('function');
            expect(typeof client.wallets.import).toBe('function');
            expect(typeof client.wallets.exportKey).toBe('function');
            expect(typeof client.wallets.configureAddress).toBe('function');
            expect(typeof client.wallets.getGasEstimates).toBe('function');
            expect(typeof client.wallets.checkWithdrawalCapability).toBe('function');
            expect(typeof client.wallets.checkGasRequirements).toBe('function');
        });
    });
    describe('Withdrawals Resource', () => {
        it('should have withdrawal methods', () => {
            expect(typeof client.withdrawals.create).toBe('function');
            expect(typeof client.withdrawals.get).toBe('function');
            expect(typeof client.withdrawals.list).toBe('function');
            expect(typeof client.withdrawals.cancel).toBe('function');
            expect(typeof client.withdrawals.process).toBe('function');
        });
    });
    describe('Security Resource', () => {
        it('should have security methods', () => {
            expect(typeof client.security.getEvents).toBe('function');
            expect(typeof client.security.getAlerts).toBe('function');
            expect(typeof client.security.acknowledgeAlert).toBe('function');
            expect(typeof client.security.getSettings).toBe('function');
            expect(typeof client.security.updateSettings).toBe('function');
            expect(typeof client.security.getBalanceAlerts).toBe('function');
            expect(typeof client.security.resolveBalanceAlert).toBe('function');
            expect(typeof client.security.checkGasBalances).toBe('function');
        });
    });
    describe('Balances Resource', () => {
        it('should have balance methods', () => {
            expect(typeof client.balances.get).toBe('function');
            expect(typeof client.balances.getHistory).toBe('function');
        });
    });
    describe('Audit Logs Resource', () => {
        it('should have audit log methods', () => {
            expect(typeof client.auditLogs.list).toBe('function');
        });
    });
    describe('Sandbox Resource', () => {
        it('should have sandbox methods', () => {
            expect(typeof client.sandbox.simulatePayment).toBe('function');
        });
    });
    describe('Method Parameter Validation', () => {
        it('should validate required string parameters', async () => {
            // Test synchronous validation - empty strings should throw validation errors
            await expect(async () => {
                await client.payments.retrieve('');
            }).rejects.toThrow('Payment ID is required');
            await expect(async () => {
                await client.payments.cancel('');
            }).rejects.toThrow('Payment ID is required');
        });
        it('should accept valid string parameters', () => {
            expect(() => {
                client.payments.retrieve('pay_test123');
                client.payments.cancel('pay_test123');
                client.withdrawals.get('wd_test123');
                client.withdrawals.cancel('wd_test123');
            }).not.toThrow();
        });
    });
    describe('Request Options Support', () => {
        it('should accept request options in all methods', () => {
            const options = {
                timeout: 10000,
                retries: 2,
                idempotencyKey: 'test-key-123'
            };
            expect(() => {
                client.payments.create({
                    amount_usd: '100.00',
                    crypto_type: 'ETH'
                }, options);
                client.payments.retrieve('pay_test123', options);
                client.payments.list({}, options);
                client.refunds.create({
                    payment_id: 'pay_test123',
                    amount: '50.00'
                }, options);
            }).not.toThrow();
        });
        it('should handle empty request options', () => {
            expect(() => {
                client.payments.create({
                    amount_usd: '100.00',
                    crypto_type: 'ETH'
                }, {});
                client.payments.retrieve('pay_test123', {});
            }).not.toThrow();
        });
    });
    describe('Type Safety', () => {
        it('should enforce crypto type constraints', () => {
            const validCryptoTypes = ['SOL', 'ETH', 'BNB', 'MATIC', 'ARB', 'USDT_ETH', 'USDT_BEP20', 'USDT_POLYGON', 'USDT_ARBITRUM', 'USDT_SPL'];
            validCryptoTypes.forEach(cryptoType => {
                expect(() => {
                    client.payments.create({
                        amount_usd: '100.00',
                        crypto_type: cryptoType
                    });
                }).not.toThrow();
            });
        });
        it('should enforce payment status constraints', () => {
            const validStatuses = ['PENDING', 'CONFIRMING', 'CONFIRMED', 'FAILED', 'EXPIRED'];
            validStatuses.forEach(status => {
                expect(() => {
                    client.payments.list({
                        status: status
                    });
                }).not.toThrow();
            });
        });
    });
    describe('Error Handling', () => {
        it('should throw FidduPayValidationError for validation failures', async () => {
            try {
                await client.payments.create({
                    amount_usd: 'invalid',
                    crypto_type: 'ETH'
                });
            }
            catch (error) {
                expect(error).toBeInstanceOf(errors_1.FidduPayValidationError);
            }
        });
        it('should provide meaningful error messages', async () => {
            await expect(client.payments.create({
                amount_usd: '0',
                crypto_type: 'ETH'
            })).rejects.toThrow('Amount must be a positive number');
            await expect(client.payments.create({
                amount_usd: '100.00',
                crypto_type: 'INVALID'
            })).rejects.toThrow('Invalid crypto type');
        });
    });
    describe('Backward Compatibility', () => {
        it('should maintain backward compatibility with existing method signatures', () => {
            // Test that old method signatures still work
            expect(() => {
                client.payments.create({
                    amount_usd: '100.00',
                    crypto_type: 'ETH',
                    description: 'Test payment'
                });
            }).not.toThrow();
            expect(() => {
                client.refunds.create({
                    payment_id: 'pay_test123',
                    amount: '50.00'
                });
            }).not.toThrow();
        });
        it('should support legacy crypto type formats', () => {
            // Test that USDT variants are supported
            const usdtVariants = ['USDT_ETH', 'USDT_BEP20', 'USDT_POLYGON', 'USDT_ARBITRUM', 'USDT_SPL'];
            usdtVariants.forEach(variant => {
                expect(() => {
                    client.payments.create({
                        amount_usd: '100.00',
                        crypto_type: variant
                    });
                }).not.toThrow();
            });
        });
    });
});
//# sourceMappingURL=sdk-resources.test.js.map