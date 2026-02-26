import { HttpClient } from '../client';
import { Analytics, RequestOptions } from '../types';

export class AnalyticsResource {
  constructor(private client: HttpClient) { }

  /**
   * Get analytics data
   */
  async retrieve(params?: {
    from_date?: string;
    to_date?: string;
    granularity?: 'day' | 'week' | 'month';
  }, options?: RequestOptions): Promise<Analytics> {
    const queryParams = new URLSearchParams();

    if (params?.from_date) queryParams.append('from_date', params.from_date);
    if (params?.to_date) queryParams.append('to_date', params.to_date);
    if (params?.granularity) queryParams.append('granularity', params.granularity);

    const query = queryParams.toString();
    const path = query ? `/api/v1/merchants/analytics?${query}` : '/api/v1/merchants/analytics';

    return this.client.request<Analytics>('GET', path);
  }

  /**
   * Export analytics data
   */
  async export(params: {
    format?: 'csv' | 'json' | 'xlsx';
    from_date: string;
    to_date: string;
  }, options?: RequestOptions): Promise<string> {
    const queryParams = new URLSearchParams();
    if (params.format) queryParams.append('format', params.format);
    queryParams.append('from_date', params.from_date);
    queryParams.append('to_date', params.to_date);

    return this.client.request('GET', `/api/v1/merchants/analytics/export?${queryParams.toString()}`);
  }
}
