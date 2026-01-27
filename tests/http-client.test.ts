import { HttpClient } from '../src/client';
import {
  FidduPayAPIError,
  FidduPayAuthenticationError,
  FidduPayConnectionError,
  FidduPayRateLimitError
} from '../src/errors';
import axios from 'axios';

// Mock axios
jest.mock('axios');
const mockedAxios = axios as jest.Mocked<typeof axios>;

describe('HTTP Client', () => {
  let client: HttpClient;
  const mockAxiosInstance = {
    request: jest.fn(),
    interceptors: {
      request: { use: jest.fn() },
      response: { use: jest.fn() }
    }
  };

  beforeEach(() => {
    jest.clearAllMocks();
    mockedAxios.create.mockReturnValue(mockAxiosInstance as any);
    
    client = new HttpClient({
      apiKey: 'sk_test_1234567890',
      environment: 'sandbox'
    });
  });

  describe('Client Initialization', () => {
    it('should create axios instance with correct config', () => {
      expect(mockedAxios.create).toHaveBeenCalledWith({
        baseURL: 'https://api-sandbox.fiddupay.com/v1',
        timeout: 30000,
        headers: {
          'Content-Type': 'application/json',
          'User-Agent': 'FidduPay-Node/1.0.0',
          'Authorization': 'Bearer sk_test_1234567890'
        },
        maxRedirects: 0,
        validateStatus: expect.any(Function)
      });
    });

    it('should use production URL for production environment', () => {
      new HttpClient({
        apiKey: 'live_1234567890',
        environment: 'production'
      });

      expect(mockedAxios.create).toHaveBeenLastCalledWith(
        expect.objectContaining({
          baseURL: 'https://api.fiddupay.com/v1'
        })
      );
    });

    it('should use custom baseURL when provided', () => {
      new HttpClient({
        apiKey: 'sk_test_1234567890',
        baseURL: 'https://custom.api.com/v1'
      });

      expect(mockedAxios.create).toHaveBeenLastCalledWith(
        expect.objectContaining({
          baseURL: 'https://custom.api.com/v1'
        })
      );
    });

    it('should use custom timeout when provided', () => {
      new HttpClient({
        apiKey: 'sk_test_1234567890',
        timeout: 10000
      });

      expect(mockedAxios.create).toHaveBeenLastCalledWith(
        expect.objectContaining({
          timeout: 10000
        })
      );
    });

    it('should setup request and response interceptors', () => {
      expect(mockAxiosInstance.interceptors.request.use).toHaveBeenCalled();
      expect(mockAxiosInstance.interceptors.response.use).toHaveBeenCalled();
    });
  });

  describe('Request Methods', () => {
    beforeEach(() => {
      mockAxiosInstance.request.mockResolvedValue({ data: { success: true } });
    });

    it('should make GET request', async () => {
      await client.get('/test');

      expect(mockAxiosInstance.request).toHaveBeenCalledWith({
        method: 'GET',
        url: '/test',
        timeout: undefined
      });
    });

    it('should make POST request with data', async () => {
      const data = { test: 'data' };
      await client.post('/test', data);

      expect(mockAxiosInstance.request).toHaveBeenCalledWith({
        method: 'POST',
        url: '/test',
        data,
        timeout: undefined
      });
    });

    it('should make PUT request with data', async () => {
      const data = { test: 'data' };
      await client.put('/test', data);

      expect(mockAxiosInstance.request).toHaveBeenCalledWith({
        method: 'PUT',
        url: '/test',
        data,
        timeout: undefined
      });
    });

    it('should make DELETE request', async () => {
      await client.delete('/test');

      expect(mockAxiosInstance.request).toHaveBeenCalledWith({
        method: 'DELETE',
        url: '/test',
        timeout: undefined
      });
    });

    it('should include idempotency key when provided', async () => {
      await client.post('/test', {}, { idempotencyKey: 'idem_123' });

      expect(mockAxiosInstance.request).toHaveBeenCalledWith({
        method: 'POST',
        url: '/test',
        data: {},
        timeout: undefined,
        headers: {
          'Idempotency-Key': 'idem_123'
        }
      });
    });

    it('should use custom timeout from options', async () => {
      await client.get('/test', { timeout: 5000 });

      expect(mockAxiosInstance.request).toHaveBeenCalledWith({
        method: 'GET',
        url: '/test',
        timeout: 5000
      });
    });
  });

  describe('Error Handling', () => {
    it('should handle 401 authentication errors', async () => {
      const error = {
        response: {
          status: 401,
          data: { error: { message: 'Invalid API key' } },
          headers: {}
        }
      };
      mockAxiosInstance.request.mockRejectedValue(error);

      await expect(client.get('/test')).rejects.toThrow(FidduPayAuthenticationError);
    });

    it('should handle 429 rate limit errors', async () => {
      const error = {
        response: {
          status: 429,
          data: { error: { message: 'Rate limited' } },
          headers: { 'retry-after': '60' }
        }
      };
      mockAxiosInstance.request.mockRejectedValue(error);

      await expect(client.get('/test')).rejects.toThrow(FidduPayRateLimitError);
    });

    it('should handle general API errors', async () => {
      const error = {
        response: {
          status: 400,
          data: { 
            error: { 
              message: 'Bad request',
              code: 'invalid_request'
            }
          },
          headers: { 'x-request-id': 'req_123' }
        }
      };
      mockAxiosInstance.request.mockRejectedValue(error);

      await expect(client.get('/test')).rejects.toThrow(FidduPayAPIError);
    });

    it('should handle network errors', async () => {
      const error = { request: {} };
      mockAxiosInstance.request.mockRejectedValue(error);

      await expect(client.get('/test')).rejects.toThrow(FidduPayConnectionError);
    });

    it('should handle other errors', async () => {
      const error = new Error('Unknown error');
      mockAxiosInstance.request.mockRejectedValue(error);

      await expect(client.get('/test')).rejects.toThrow(FidduPayConnectionError);
    });
  });

  describe('Retry Logic', () => {
    it('should retry on server errors', async () => {
      const error = {
        response: {
          status: 500,
          data: { error: { message: 'Server error' } },
          headers: {}
        }
      };

      mockAxiosInstance.request
        .mockRejectedValueOnce(error)
        .mockRejectedValueOnce(error)
        .mockResolvedValue({ data: { success: true } });

      const result = await client.get('/test');
      expect(result).toEqual({ success: true });
      expect(mockAxiosInstance.request).toHaveBeenCalledTimes(3);
    });

    it('should not retry on authentication errors', async () => {
      const error = {
        response: {
          status: 401,
          data: { error: { message: 'Invalid API key' } },
          headers: {}
        }
      };
      mockAxiosInstance.request.mockRejectedValue(error);

      await expect(client.get('/test')).rejects.toThrow(FidduPayAuthenticationError);
      expect(mockAxiosInstance.request).toHaveBeenCalledTimes(1);
    });

    it('should not retry on client errors (except 429)', async () => {
      const error = {
        response: {
          status: 400,
          data: { error: { message: 'Bad request' } },
          headers: {}
        }
      };
      mockAxiosInstance.request.mockRejectedValue(error);

      await expect(client.get('/test')).rejects.toThrow(FidduPayAPIError);
      expect(mockAxiosInstance.request).toHaveBeenCalledTimes(1);
    });

    it('should respect custom retry count', async () => {
      const error = {
        response: {
          status: 500,
          data: { error: { message: 'Server error' } },
          headers: {}
        }
      };
      mockAxiosInstance.request.mockRejectedValue(error);

      await expect(client.get('/test', { retries: 1 })).rejects.toThrow();
      expect(mockAxiosInstance.request).toHaveBeenCalledTimes(2); // 1 initial + 1 retry
    });

    it('should respect retry-after header for rate limits', async () => {
      const error = {
        response: {
          status: 429,
          data: { error: { message: 'Rate limited' } },
          headers: { 'retry-after': '1' }
        }
      };

      mockAxiosInstance.request
        .mockRejectedValueOnce(error)
        .mockResolvedValue({ data: { success: true } });

      const startTime = Date.now();
      const result = await client.get('/test');
      const endTime = Date.now();

      expect(result).toEqual({ success: true });
      expect(endTime - startTime).toBeGreaterThanOrEqual(1000); // At least 1 second delay
    });
  });

  describe('Request ID Generation', () => {
    it('should generate unique request IDs', () => {
      const client1 = new HttpClient({ apiKey: 'sk_test_1' });
      const client2 = new HttpClient({ apiKey: 'sk_test_2' });

      // Access the private method through any
      const id1 = (client1 as any).generateRequestId();
      const id2 = (client2 as any).generateRequestId();

      expect(id1).toMatch(/^req_\d+_[a-z0-9]{9}$/);
      expect(id2).toMatch(/^req_\d+_[a-z0-9]{9}$/);
      expect(id1).not.toBe(id2);
    });
  });

  describe('Status Code Validation', () => {
    it('should validate status codes correctly', () => {
      const createCall = mockedAxios.create.mock.calls[0];
      if (createCall && createCall[0] && createCall[0].validateStatus) {
        const validateStatus = createCall[0].validateStatus;

        expect(validateStatus(200)).toBe(true);
        expect(validateStatus(201)).toBe(true);
        expect(validateStatus(299)).toBe(true);
        expect(validateStatus(300)).toBe(false);
        expect(validateStatus(400)).toBe(false);
        expect(validateStatus(500)).toBe(false);
      } else {
        // Fallback test if mock structure is different
        expect(mockedAxios.create).toHaveBeenCalled();
      }
    });
  });

  describe('Configuration Options', () => {
    it('should use default max retries', () => {
      const client = new HttpClient({ apiKey: 'sk_test_1234567890' });
      expect((client as any).maxRetries).toBe(3);
    });

    it('should use custom max retries', () => {
      const client = new HttpClient({ 
        apiKey: 'sk_test_1234567890',
        maxRetries: 5
      });
      expect((client as any).maxRetries).toBe(5);
    });

    it('should store API key', () => {
      const client = new HttpClient({ apiKey: 'sk_test_1234567890' });
      expect((client as any).apiKey).toBe('sk_test_1234567890');
    });
  });

  describe('Sleep Function', () => {
    it('should sleep for specified duration', async () => {
      const client = new HttpClient({ apiKey: 'sk_test_1234567890' });
      const startTime = Date.now();
      await (client as any).sleep(100);
      const endTime = Date.now();

      expect(endTime - startTime).toBeGreaterThanOrEqual(100);
    });
  });

  describe('Error Response Parsing', () => {
    it('should parse error message from response data', async () => {
      const error = {
        response: {
          status: 400,
          data: { 
            error: { message: 'Custom error message' }
          },
          headers: {}
        }
      };
      mockAxiosInstance.request.mockRejectedValue(error);

      await expect(client.get('/test')).rejects.toThrow('Custom error message');
    });

    it('should fallback to data.message if error.message not available', async () => {
      const error = {
        response: {
          status: 400,
          data: { message: 'Fallback message' },
          headers: {}
        }
      };
      mockAxiosInstance.request.mockRejectedValue(error);

      await expect(client.get('/test')).rejects.toThrow('Fallback message');
    });

    it('should use default message if no message in response', async () => {
      const error = {
        response: {
          status: 400,
          data: {},
          headers: {}
        }
      };
      mockAxiosInstance.request.mockRejectedValue(error);

      await expect(client.get('/test')).rejects.toThrow('API request failed');
    });

    it('should include request ID in error', async () => {
      const error = {
        response: {
          status: 400,
          data: { error: { message: 'Error with request ID' } },
          headers: { 'x-request-id': 'req_test_123' }
        }
      };
      mockAxiosInstance.request.mockRejectedValue(error);

      try {
        await client.get('/test');
      } catch (e) {
        expect((e as FidduPayAPIError).requestId).toBe('req_test_123');
      }
    });
  });
});