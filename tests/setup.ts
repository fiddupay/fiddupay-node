// Mock axios to prevent real HTTP calls in tests
jest.mock('axios', () => ({
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
}));

// Set test timeout to prevent hanging tests
jest.setTimeout(10000);
