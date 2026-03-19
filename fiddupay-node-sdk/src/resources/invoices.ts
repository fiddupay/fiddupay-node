import { HttpClient } from '../client';
import { CreateInvoiceRequest, Invoice, RequestOptions } from '../types';

export class InvoicesResource {
  constructor(private client: HttpClient) { }

  /**
   * Create a new invoice
   */
  async create(data: CreateInvoiceRequest, options?: RequestOptions): Promise<Invoice> {
    return this.client.request<Invoice>('POST', '/api/v1/merchants/invoices', data);
  }

  /**
   * List invoices with optional limit
   */
  async list(params?: { limit?: number; [key: string]: any }, options?: RequestOptions): Promise<Invoice[]> {
    const queryParams = new URLSearchParams();

    if (params) {
      for (const [key, value] of Object.entries(params)) {
        if (value !== undefined && value !== null) {
          queryParams.append(key, value.toString());
        }
      }
    }
    const query = queryParams.toString();
    const path = query ? `/api/v1/merchants/invoices?${query}` : '/api/v1/merchants/invoices';
    return this.client.request<Invoice[]>('GET', path);
  }

  /**
   * Retrieve an invoice by ID
   */
  async retrieve(invoiceId: string, options?: RequestOptions): Promise<Invoice> {
    return this.client.request<Invoice>('GET', `/api/v1/merchants/invoices/${invoiceId}`);
  }
}
