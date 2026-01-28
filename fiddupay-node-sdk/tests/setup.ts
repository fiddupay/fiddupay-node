// Test setup file
import { jest } from '@jest/globals';

// Mock axios to prevent actual HTTP requests
jest.mock('axios', () => ({
  create: jest.fn(() => ({
    request: jest.fn().mockResolvedValue({
      data: { success: true },
      status: 200,
      statusText: 'OK',
      headers: {},
      config: {}
    }),
    interceptors: {
      request: { use: jest.fn() },
      response: { use: jest.fn() }
    }
  })),
  isAxiosError: jest.fn(() => false)
}));
