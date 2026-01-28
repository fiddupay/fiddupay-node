import { HttpClient } from '../client';

export class Contact {
  constructor(private client: HttpClient) {}

  /**
   * Submit contact form (public endpoint - no auth required)
   */
  async submit(data: {
    name: string;
    email: string;
    subject: string;
    message: string;
  }): Promise<{
    message: string;
    status: string;
    id?: number;
  }> {
    // Note: This is a public endpoint but included in SDK for convenience
    return this.client.request('POST', '/api/v1/contact', data);
  }
}
