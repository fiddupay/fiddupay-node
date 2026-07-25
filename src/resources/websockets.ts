import { HttpClient } from '../client';

/**
 * WebSockets Resource
 * 
 * Provides helpers for connecting to the FidduPay real-time notification system.
 */
export class WebSockets {
  constructor(private client: HttpClient) {}

  /**
   * Generate an authenticated WebSocket URL for real-time dashboard notifications.
   * 
   * The returned URL includes the authentication token in the query parameters,
   * which is the most compatible way to connect using the browser's native 
   * WebSocket API.
   * 
   * @returns The full WebSocket URL (wss://...)
   */
  getNotificationUrl(): string {
    const baseURL = this.client.getBaseURL();
    const apiKey = this.client.getApiKey();
    
    // Replace http/https with ws/wss
    const wsBaseURL = baseURL.replace(/^http/, 'ws');
    
    // The backend auth middleware supports ?token=... for WebSockets
    // We sanitize the base URL to ensure it doesn't have a trailing slash before appending path
    const sanitizedBase = wsBaseURL.endsWith('/') ? wsBaseURL.slice(0, -1) : wsBaseURL;
    
    return `${sanitizedBase}/api/v1/merchants/ws?token=${apiKey}`;
  }
}
