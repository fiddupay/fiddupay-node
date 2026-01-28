import { HttpClient } from '../client';

export class InvoicesResource {
  constructor(private client: HttpClient) {}

  async create(data: {
    amount_usd: string;
    description: string;
    due_date?: string;
    customer_email?: string;
  }) {
    return this.client.request('POST', '/api/v1/merchant/invoices', data);
  }

  async list() {
    return this.client.request('GET', '/api/v1/merchant/invoices');
  }

  async retrieve(invoiceId: string) {
    return this.client.request('GET', `/api/v1/merchant/invoices/${invoiceId}`);
  }
}
