import axios, { AxiosInstance, AxiosRequestConfig, AxiosResponse } from 'axios';
import { FidduPayConfig, RequestOptions } from './types';
import {
  FidduPayAPIError,
  FidduPayAuthenticationError,
  FidduPayConnectionError,
  FidduPayRateLimitError
} from './errors';

export class HttpClient {
  private client: AxiosInstance;
  private apiKey: string;
  private maxRetries: number;

  constructor(config: FidduPayConfig) {
    this.apiKey = config.apiKey;
    this.maxRetries = config.maxRetries || 3;

    const baseURL = config.baseURL || this.getBaseURL(config.environment || 'sandbox');

    this.client = axios.create({
      baseURL,
      timeout: config.timeout || 30000,
      headers: {
        'Content-Type': 'application/json',
        'User-Agent': 'FidduPay-Node/1.0.0'
      },
      // Security configurations
      maxRedirects: 0, // Prevent redirect attacks
      validateStatus: (status) => status >= 200 && status < 300,
    });

    this.setupInterceptors();
  }

  private getBaseURL(environment: string): string {
    return environment === 'production' 
      ? 'https://api.fiddupay.com/v1'
      : 'https://api-sandbox.fiddupay.com/v1';
  }

  private setupInterceptors(): void {
    // Request interceptor
    this.client.interceptors.request.use(
      (config) => {
        // Add request ID for tracking
        config.headers['X-Request-ID'] = this.generateRequestId();
        return config;
      },
      (error) => Promise.reject(error)
    );

    // Response interceptor
    this.client.interceptors.response.use(
      (response) => response,
      (error) => {
        if (error.response) {
          return Promise.reject(this.handleAPIError(error.response));
        } else if (error.request) {
          return Promise.reject(new FidduPayConnectionError('Network request failed'));
        } else {
          return Promise.reject(new FidduPayConnectionError((error as Error).message));
        }
      }
    );
  }

  private handleAPIError(response: AxiosResponse): Error {
    const { status, data } = response;
    const message = data?.error?.message || data?.message || 'API request failed';
    const code = data?.error?.code || data?.code;
    const requestId = response.headers['x-request-id'];

    switch (status) {
      case 401:
        return new FidduPayAuthenticationError(message);
      case 429:
        const retryAfter = parseInt(response.headers['retry-after']) || undefined;
        return new FidduPayRateLimitError(message, retryAfter);
      default:
        return new FidduPayAPIError(message, status, code, requestId);
    }
  }

  private generateRequestId(): string {
    return `req_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  }

  private async sleep(ms: number): Promise<void> {
    return new Promise(resolve => setTimeout(resolve, ms));
  }

  async request<T>(
    method: 'GET' | 'POST' | 'PUT' | 'DELETE',
    path: string,
    data?: any,
    options: RequestOptions = {}
  ): Promise<T> {
    const config: AxiosRequestConfig = {
      method,
      url: path,
      timeout: options.timeout,
    };

    if (data) {
      config.data = data;
    }

    // Don't add Authorization header for registration endpoint or registration key
    if (path !== '/api/v1/merchant/register' && this.apiKey !== 'registration_key') {
      config.headers = {
        'Authorization': `Bearer ${this.apiKey}`
      };
    }

    if (options.idempotencyKey) {
      config.headers = {
        ...config.headers,
        'Idempotency-Key': options.idempotencyKey
      };
    }

    const maxRetries = options.retries !== undefined ? options.retries : this.maxRetries;
    let lastError: Error;

    for (let attempt = 0; attempt <= maxRetries; attempt++) {
      try {
        const response = await this.client.request<T>(config);
        return response.data;
      } catch (error: any) {
        lastError = error;

        // Don't retry on authentication errors or client errors (4xx except 429)
        if (error instanceof FidduPayAuthenticationError) {
          throw error;
        }

        if (error instanceof FidduPayAPIError && error.statusCode) {
          if (error.statusCode >= 400 && error.statusCode < 500 && error.statusCode !== 429) {
            throw error;
          }
        }

        // Don't retry on the last attempt
        if (attempt === maxRetries) {
          break;
        }

        // Calculate backoff delay
        const baseDelay = Math.pow(2, attempt) * 1000; // Exponential backoff
        const jitter = Math.random() * 1000; // Add jitter
        const delay = baseDelay + jitter;

        // For rate limit errors, respect the Retry-After header
        if (error instanceof FidduPayRateLimitError && error.retryAfter) {
          await this.sleep(error.retryAfter * 1000);
        } else {
          await this.sleep(delay);
        }
      }
    }

    throw lastError!;
  }

  async get<T>(path: string, options?: RequestOptions): Promise<T> {
    return this.request<T>('GET', path, undefined, options);
  }

  async post<T>(path: string, data?: any, options?: RequestOptions): Promise<T> {
    return this.request<T>('POST', path, data, options);
  }

  async put<T>(path: string, data?: any, options?: RequestOptions): Promise<T> {
    return this.request<T>('PUT', path, data, options);
  }

  async delete<T>(path: string, options?: RequestOptions): Promise<T> {
    return this.request<T>('DELETE', path, undefined, options);
  }
}
