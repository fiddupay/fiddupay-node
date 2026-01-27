import { HttpClient } from '../client';
import { RequestOptions } from '../types';

export class Contact {
  constructor(private client: HttpClient) {}

  /**
   * Submit contact form
   */
  async submit(data: {
    name: string;
    email: string;
    subject: string;
    message: string;
  }, options?: RequestOptions): Promise<{
    message: string;
    status: string;
    id?: number;
  }> {
    return this.client.request('POST', '/api/v1/contact', data);
  }
}

export class Pricing {
  constructor(private client: HttpClient) {}

  /**
   * Get pricing information
   */
  async get(options?: RequestOptions): Promise<{
    transaction_fee_percentage: number;
    daily_volume_limit_non_kyc_usd: string;
    supported_networks: number;
    supported_cryptocurrencies: string[];
    limits: {
      kyc_verified: {
        daily_volume_limit: string;
        transaction_limit: string;
      };
      non_kyc: {
        daily_volume_limit: string;
        transaction_limit: string;
      };
    };
    features: {
      api_access: boolean;
      dashboard_analytics: boolean;
      instant_settlements: boolean;
      real_time_notifications: boolean;
      sandbox_testing: boolean;
      webhook_support: boolean;
    };
  }> {
    return this.client.request('GET', '/api/v1/pricing');
  }
}
