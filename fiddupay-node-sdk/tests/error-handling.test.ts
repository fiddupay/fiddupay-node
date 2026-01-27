import FidduPay from '../src';
import {
  FidduPayError,
  FidduPayAPIError,
  FidduPayValidationError,
  FidduPayAuthenticationError,
  FidduPayRateLimitError,
  FidduPayConnectionError
} from '../src/errors';

describe('Error Handling', () => {
  describe('FidduPayError', () => {
    it('should create base error with message and type', () => {
      const error = new FidduPayError('Test error', 'test_error');
      expect(error.message).toBe('Test error');
      expect(error.type).toBe('test_error');
      expect(error.name).toBe('FidduPayError');
      expect(error).toBeInstanceOf(Error);
    });

    it('should use default type if not provided', () => {
      const error = new FidduPayError('Test error');
      expect(error.type).toBe('fiddupay_error');
    });
  });

  describe('FidduPayAPIError', () => {
    it('should create API error with status code', () => {
      const error = new FidduPayAPIError('API error', 400, 'invalid_request', 'req_123');
      expect(error.message).toBe('API error');
      expect(error.statusCode).toBe(400);
      expect(error.code).toBe('invalid_request');
      expect(error.requestId).toBe('req_123');
      expect(error.type).toBe('api_error');
      expect(error.name).toBe('FidduPayAPIError');
    });

    it('should work without optional parameters', () => {
      const error = new FidduPayAPIError('API error', 500);
      expect(error.statusCode).toBe(500);
      expect(error.code).toBeUndefined();
      expect(error.requestId).toBeUndefined();
    });
  });

  describe('FidduPayValidationError', () => {
    it('should create validation error with parameter', () => {
      const error = new FidduPayValidationError('Invalid amount', 'amount');
      expect(error.message).toBe('Invalid amount');
      expect(error.param).toBe('amount');
      expect(error.type).toBe('validation_error');
      expect(error.name).toBe('FidduPayValidationError');
    });

    it('should work without parameter', () => {
      const error = new FidduPayValidationError('Validation failed');
      expect(error.param).toBeUndefined();
    });
  });

  describe('FidduPayAuthenticationError', () => {
    it('should create authentication error with default message', () => {
      const error = new FidduPayAuthenticationError();
      expect(error.message).toBe('Invalid API key provided');
      expect(error.type).toBe('authentication_error');
      expect(error.name).toBe('FidduPayAuthenticationError');
    });

    it('should accept custom message', () => {
      const error = new FidduPayAuthenticationError('Custom auth error');
      expect(error.message).toBe('Custom auth error');
    });
  });

  describe('FidduPayRateLimitError', () => {
    it('should create rate limit error with retry after', () => {
      const error = new FidduPayRateLimitError('Rate limited', 60);
      expect(error.message).toBe('Rate limited');
      expect(error.retryAfter).toBe(60);
      expect(error.type).toBe('rate_limit_error');
      expect(error.name).toBe('FidduPayRateLimitError');
    });

    it('should use default message and no retry after', () => {
      const error = new FidduPayRateLimitError();
      expect(error.message).toBe('Too many requests');
      expect(error.retryAfter).toBeUndefined();
    });
  });

  describe('FidduPayConnectionError', () => {
    it('should create connection error with default message', () => {
      const error = new FidduPayConnectionError();
      expect(error.message).toBe('Network connection failed');
      expect(error.type).toBe('connection_error');
      expect(error.name).toBe('FidduPayConnectionError');
    });

    it('should accept custom message', () => {
      const error = new FidduPayConnectionError('Custom connection error');
      expect(error.message).toBe('Custom connection error');
    });
  });

  describe('Client Configuration Validation', () => {
    it('should throw validation error for missing API key', () => {
      expect(() => {
        new FidduPay({} as any);
      }).toThrow(FidduPayValidationError);
    });

    it('should throw validation error for invalid API key format', () => {
      expect(() => {
        new FidduPay({ apiKey: 'invalid_key' });
      }).toThrow('Invalid API key format');
    });

    it('should throw validation error for invalid environment', () => {
      expect(() => {
        new FidduPay({ 
          apiKey: 'sk_test_1234567890',
          environment: 'invalid' as any
        });
      }).toThrow('Environment must be either "sandbox" or "production"');
    });

    it('should throw validation error for mismatched API key and environment', () => {
      expect(() => {
        new FidduPay({ 
          apiKey: 'sk_test_1234567890',
          environment: 'production'
        });
      }).toThrow('Production environment requires API key starting with "live_"');

      expect(() => {
        new FidduPay({ 
          apiKey: 'live_1234567890',
          environment: 'sandbox'
        });
      }).toThrow('Sandbox environment requires API key starting with "sk_"');
    });

    it('should throw validation error for invalid timeout', () => {
      expect(() => {
        new FidduPay({ 
          apiKey: 'sk_test_1234567890',
          timeout: 500
        });
      }).toThrow('Timeout must be between 1000ms and 60000ms');

      expect(() => {
        new FidduPay({ 
          apiKey: 'sk_test_1234567890',
          timeout: 70000
        });
      }).toThrow('Timeout must be between 1000ms and 60000ms');
    });

    it('should throw validation error for invalid max retries', () => {
      expect(() => {
        new FidduPay({ 
          apiKey: 'sk_test_1234567890',
          maxRetries: -1
        });
      }).toThrow('Max retries must be between 0 and 10');

      expect(() => {
        new FidduPay({ 
          apiKey: 'sk_test_1234567890',
          maxRetries: 15
        });
      }).toThrow('Max retries must be between 0 and 10');
    });

    it('should auto-detect environment from API key', () => {
      const sandboxClient = new FidduPay({ apiKey: 'sk_test_1234567890' });
      expect(sandboxClient).toBeInstanceOf(FidduPay);

      const prodClient = new FidduPay({ apiKey: 'live_1234567890' });
      expect(prodClient).toBeInstanceOf(FidduPay);
    });

    it('should accept valid configuration', () => {
      const client = new FidduPay({
        apiKey: 'sk_test_1234567890',
        environment: 'sandbox',
        timeout: 5000,
        maxRetries: 3
      });
      expect(client).toBeInstanceOf(FidduPay);
    });
  });
});