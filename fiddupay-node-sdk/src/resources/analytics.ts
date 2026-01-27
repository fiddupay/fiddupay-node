import { HttpClient } from '../client';
import { Analytics, RequestOptions } from '../types';

export class AnalyticsResource {
  constructor(private client: HttpClient) {}

  /**
   * Get analytics data
   */
  async retrieve(params?: {
    start_date?: string;
    end_date?: string;
    granularity?: 'day' | 'week' | 'month';
  }, options?: RequestOptions): Promise<Analytics> {
    const queryParams = new URLSearchParams();
    
    if (params?.start_date) queryParams.append('start_date', params.start_date);
    if (params?.end_date) queryParams.append('end_date', params.end_date);
    if (params?.granularity) queryParams.append('granularity', params.granularity);

    const query = queryParams.toString();
    const path = query ? `/api/v1/analytics?${query}` : '/api/v1/analytics';
    
    return this.client.request<Analytics>('GET', path);
  }

  /**
   * Export analytics data
   */
  async export(params: {
    format: 'csv' | 'json' | 'xlsx';
    start_date: string;
    end_date: string;
  }, options?: RequestOptions): Promise<{
    export_id: string;
    status: string;
    format: string;
    download_url: string | null;
    expires_at: string;
    created_at: string;
  }> {
    return this.client.request('POST', '/api/v1/analytics/export', params);
  }
}
