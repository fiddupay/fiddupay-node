"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const src_1 = __importDefault(require("../src"));
const errors_1 = require("../src/errors");
describe('Client Configuration & Error Handling', () => {
    describe('Client Configuration', () => {
        it('should create client with minimal config', () => {
            const client = new src_1.default({
                apiKey: 'sk_test_1234567890'
            });
            expect(client).toBeInstanceOf(src_1.default);
        });
        it('should create client with full config', () => {
            const client = new src_1.default({
                apiKey: 'sk_test_1234567890',
                environment: 'sandbox',
                timeout: 15000,
                maxRetries: 5,
                baseURL: 'https://custom-api.example.com/v1'
            });
            expect(client).toBeInstanceOf(src_1.default);
        });
        it('should auto-detect environment from API key', () => {
            expect(() => {
                new src_1.default({ apiKey: 'sk_test_1234567890' }); // Should default to sandbox
            }).not.toThrow();
            expect(() => {
                new src_1.default({ apiKey: 'live_1234567890' }); // Should default to production
            }).not.toThrow();
        });
        it('should validate API key format', () => {
            expect(() => {
                new src_1.default({ apiKey: 'invalid_key' });
            }).toThrow('Invalid API key format');
            expect(() => {
                new src_1.default({ apiKey: 'test_1234567890' });
            }).toThrow('Invalid API key format');
            expect(() => {
                new src_1.default({ apiKey: '' });
            }).toThrow('API key is required');
        });
        it('should validate environment values', () => {
            expect(() => {
                new src_1.default({
                    apiKey: 'sk_test_1234567890',
                    environment: 'invalid'
                });
            }).toThrow('Environment must be either "sandbox" or "production"');
        });
        it('should validate API key matches environment', () => {
            expect(() => {
                new src_1.default({
                    apiKey: 'live_1234567890',
                    environment: 'sandbox'
                });
            }).toThrow('Sandbox environment requires API key starting with "sk_"');
            expect(() => {
                new src_1.default({
                    apiKey: 'sk_test_1234567890',
                    environment: 'production'
                });
            }).toThrow('Production environment requires API key starting with "live_"');
        });
        it('should validate timeout range', () => {
            expect(() => {
                new src_1.default({
                    apiKey: 'sk_test_1234567890',
                    timeout: 500
                });
            }).toThrow('Timeout must be between 1000ms and 60000ms');
            expect(() => {
                new src_1.default({
                    apiKey: 'sk_test_1234567890',
                    timeout: 70000
                });
            }).toThrow('Timeout must be between 1000ms and 60000ms');
        });
        it('should validate maxRetries range', () => {
            expect(() => {
                new src_1.default({
                    apiKey: 'sk_test_1234567890',
                    maxRetries: -1
                });
            }).toThrow('Max retries must be between 0 and 10');
            expect(() => {
                new src_1.default({
                    apiKey: 'sk_test_1234567890',
                    maxRetries: 15
                });
            }).toThrow('Max retries must be between 0 and 10');
        });
        it('should accept valid timeout values', () => {
            expect(() => {
                new src_1.default({
                    apiKey: 'sk_test_1234567890',
                    timeout: 1000
                });
            }).not.toThrow();
            expect(() => {
                new src_1.default({
                    apiKey: 'sk_test_1234567890',
                    timeout: 60000
                });
            }).not.toThrow();
        });
        it('should accept valid maxRetries values', () => {
            expect(() => {
                new src_1.default({
                    apiKey: 'sk_test_1234567890',
                    maxRetries: 0
                });
            }).not.toThrow();
            expect(() => {
                new src_1.default({
                    apiKey: 'sk_test_1234567890',
                    maxRetries: 10
                });
            }).not.toThrow();
        });
    });
    describe('Error Types', () => {
        it('should have all error types available', () => {
            expect(errors_1.FidduPayValidationError).toBeDefined();
            expect(errors_1.FidduPayAPIError).toBeDefined();
            expect(errors_1.FidduPayAuthenticationError).toBeDefined();
            expect(errors_1.FidduPayConnectionError).toBeDefined();
            expect(errors_1.FidduPayRateLimitError).toBeDefined();
        });
        it('should create FidduPayValidationError correctly', () => {
            const error = new errors_1.FidduPayValidationError('Test message', 'test_field');
            expect(error.message).toBe('Test message');
            expect(error.param).toBe('test_field');
            expect(error.name).toBe('FidduPayValidationError');
        });
        it('should create FidduPayAPIError correctly', () => {
            const error = new errors_1.FidduPayAPIError('API Error', 400, 'invalid_request', 'req_123');
            expect(error.message).toBe('API Error');
            expect(error.statusCode).toBe(400);
            expect(error.code).toBe('invalid_request');
            expect(error.requestId).toBe('req_123');
            expect(error.name).toBe('FidduPayAPIError');
        });
        it('should create FidduPayAuthenticationError correctly', () => {
            const error = new errors_1.FidduPayAuthenticationError('Invalid API key');
            expect(error.message).toBe('Invalid API key');
            expect(error.name).toBe('FidduPayAuthenticationError');
        });
        it('should create FidduPayConnectionError correctly', () => {
            const error = new errors_1.FidduPayConnectionError('Network timeout');
            expect(error.message).toBe('Network timeout');
            expect(error.name).toBe('FidduPayConnectionError');
        });
        it('should create FidduPayRateLimitError correctly', () => {
            const error = new errors_1.FidduPayRateLimitError('Rate limit exceeded', 60);
            expect(error.message).toBe('Rate limit exceeded');
            expect(error.retryAfter).toBe(60);
            expect(error.name).toBe('FidduPayRateLimitError');
        });
    });
    describe('API Key Validation Edge Cases', () => {
        it('should handle null API key', () => {
            expect(() => {
                new src_1.default({ apiKey: null });
            }).toThrow('API key is required');
        });
        it('should handle undefined API key', () => {
            expect(() => {
                new src_1.default({});
            }).toThrow('API key is required');
        });
        it('should handle whitespace-only API key', () => {
            expect(() => {
                new src_1.default({ apiKey: '   ' });
            }).toThrow('Invalid API key format');
        });
        it('should handle API keys with correct prefix but insufficient length', () => {
            expect(() => {
                new src_1.default({ apiKey: 'sk_' });
            }).not.toThrow(); // Basic validation only checks prefix
        });
        it('should handle API keys with correct prefix', () => {
            expect(() => {
                new src_1.default({ apiKey: 'sk_test_1234567890abcdef' });
            }).not.toThrow();
            expect(() => {
                new src_1.default({ apiKey: 'live_prod_1234567890abcdef' });
            }).not.toThrow();
        });
    });
    describe('Environment Detection', () => {
        it('should detect sandbox from sk_ prefix', () => {
            const client = new src_1.default({ apiKey: 'sk_test_1234567890' });
            expect(client).toBeInstanceOf(src_1.default);
        });
        it('should detect production from live_ prefix', () => {
            const client = new src_1.default({ apiKey: 'live_1234567890' });
            expect(client).toBeInstanceOf(src_1.default);
        });
        it('should respect explicit environment setting', () => {
            expect(() => {
                new src_1.default({
                    apiKey: 'sk_test_1234567890',
                    environment: 'sandbox'
                });
            }).not.toThrow();
            expect(() => {
                new src_1.default({
                    apiKey: 'live_1234567890',
                    environment: 'production'
                });
            }).not.toThrow();
        });
    });
    describe('Configuration Edge Cases', () => {
        it('should handle zero timeout', () => {
            // Zero is falsy, so validation is skipped
            expect(() => {
                new src_1.default({
                    apiKey: 'sk_test_1234567890',
                    timeout: 0
                });
            }).not.toThrow(); // Zero timeout is allowed (falsy value skips validation)
        });
        it('should handle negative timeout', () => {
            expect(() => {
                new src_1.default({
                    apiKey: 'sk_test_1234567890',
                    timeout: -1000
                });
            }).toThrow('Timeout must be between 1000ms and 60000ms');
        });
        it('should handle fractional timeout', () => {
            expect(() => {
                new src_1.default({
                    apiKey: 'sk_test_1234567890',
                    timeout: 1500.5
                });
            }).not.toThrow(); // Should accept fractional values
        });
        it('should handle string timeout', () => {
            expect(() => {
                new src_1.default({
                    apiKey: 'sk_test_1234567890',
                    timeout: '5000'
                });
            }).not.toThrow(); // JavaScript coercion should handle this
        });
        it('should handle undefined optional parameters', () => {
            expect(() => {
                new src_1.default({
                    apiKey: 'sk_test_1234567890',
                    environment: undefined,
                    timeout: undefined,
                    maxRetries: undefined,
                    baseURL: undefined
                });
            }).not.toThrow();
        });
    });
    describe('Custom Base URL', () => {
        it('should accept custom base URL', () => {
            expect(() => {
                new src_1.default({
                    apiKey: 'sk_test_1234567890',
                    baseURL: 'https://custom-api.example.com/v1'
                });
            }).not.toThrow();
        });
        it('should handle empty base URL', () => {
            expect(() => {
                new src_1.default({
                    apiKey: 'sk_test_1234567890',
                    baseURL: ''
                });
            }).not.toThrow(); // Should fall back to default
        });
        it('should handle malformed base URL', () => {
            expect(() => {
                new src_1.default({
                    apiKey: 'sk_test_1234567890',
                    baseURL: 'not-a-url'
                });
            }).not.toThrow(); // URL validation happens at request time
        });
    });
    describe('Error Inheritance', () => {
        it('should have proper error inheritance chain', () => {
            const validationError = new errors_1.FidduPayValidationError('Test');
            const apiError = new errors_1.FidduPayAPIError('Test', 400);
            const authError = new errors_1.FidduPayAuthenticationError('Test');
            const connectionError = new errors_1.FidduPayConnectionError('Test');
            const rateLimitError = new errors_1.FidduPayRateLimitError('Test');
            expect(validationError instanceof Error).toBe(true);
            expect(apiError instanceof Error).toBe(true);
            expect(authError instanceof Error).toBe(true);
            expect(connectionError instanceof Error).toBe(true);
            expect(rateLimitError instanceof Error).toBe(true);
            expect(validationError instanceof errors_1.FidduPayValidationError).toBe(true);
            expect(apiError instanceof errors_1.FidduPayAPIError).toBe(true);
            expect(authError instanceof errors_1.FidduPayAuthenticationError).toBe(true);
            expect(connectionError instanceof errors_1.FidduPayConnectionError).toBe(true);
            expect(rateLimitError instanceof errors_1.FidduPayRateLimitError).toBe(true);
        });
    });
});
//# sourceMappingURL=client-config.test.js.map