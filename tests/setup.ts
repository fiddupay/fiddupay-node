// Mock axios globally to prevent network calls
const mockAxios = {
  create: jest.fn(() => ({
    request: jest.fn().mockResolvedValue({
      data: { success: true, message: 'Mock response' },
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
};

jest.mock('axios', () => mockAxios);

// Set test timeout to prevent hanging tests
jest.setTimeout(10000);

// Mock console methods to reduce test noise
global.console = {
  ...console,
  log: jest.fn(),
  warn: jest.fn(),
  error: jest.fn()
};
