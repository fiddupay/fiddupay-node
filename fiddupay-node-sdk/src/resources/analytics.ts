import { HttpClient } from '../client';
import { Analytics, RequestOptions } from '../types';

export class AnalyticsResource {
  constructor(private client: HttpClient) {}

  async retrieve(params?: {
    start_date?: string;
    end_date?: string;
    granularity?: 'day' | 'week' | 'month';
  }): Promise<Analytics> {
    const queryParams = new URLSearchParams();
    
    if (params?.start_date) queryParams.append('start_date', params.start_date);
    if (params?.end_date) queryParams.append('end_date', params.end_date);
    if (params?.granularity) queryParams.append('granularity', params.granularity);

    const query = queryParams.toString();
    const path = query ? `/api/v1/merchant/analytics?${query}` : '/api/v1/merchant/analytics';
    
    return this.client.request<Analytics>('GET', path);
  }

  async export(params: {
    format: 'csv' | 'json' | 'xlsx';
    start_date: string;
    end_date: string;
  }): Promise<{
    export_id: string;
    status: string;
    format: string;
    download_url: string | null;
    expires_at: string;
    created_at: string;
  }> {
    return this.client.request('POST', '/api/v1/merchant/analytics/export', params);
  }
}
