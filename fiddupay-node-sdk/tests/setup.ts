import { jest } from '@jest/globals';

// Mock axios to prevent actual HTTP requests
const mockAxios: any = {
    create: jest.fn(() => ({
        request: jest.fn(() => Promise.resolve({
            data: { success: true },
            status: 200,
            statusText: 'OK',
            headers: {},
            config: {}
        })),
        interceptors: {
            request: { use: jest.fn() },
            response: { use: jest.fn() }
        }
    })),
    isAxiosError: jest.fn(() => false)
};

jest.mock('axios', () => mockAxios);
