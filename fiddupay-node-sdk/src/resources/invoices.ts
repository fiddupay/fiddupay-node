import { HttpClient } from '../client';
import { CreateInvoiceRequest, Invoice, PaginatedResponse } from '../types';

/**
 * Invoices Resource
 *
 * Handles merchant invoice creation and management.
 */
export class Invoices {
  constructor(private client: HttpClient) {}

  /**
   * Create a new invoice
   * @param data Invoice details
   */
  async create(data: CreateInvoiceRequest): Promise<Invoice> {
    return this.client.post<Invoice>('/api/v1/merchants/invoices', data);
  }

  /**
   * List invoices
   * @param params Filtering and pagination parameters
   */
  async list(params?: {
    page?: number;
    page_size?: number;
    status?: string;
  }): Promise<PaginatedResponse<Invoice>> {
    const queryParams = new URLSearchParams();
    if (params?.page) queryParams.append('page', params.page.toString());
    if (params?.page_size) queryParams.append('page_size', params.page_size.toString());
    if (params?.status) queryParams.append('status', params.status);

    const query = queryParams.toString();
    const path = query ? `/api/v1/merchants/invoices?${query}` : '/api/v1/merchants/invoices';
    
    return this.client.get<PaginatedResponse<Invoice>>(path);
  }

  /**
   * Get an invoice by ID
   * @param invoiceId The invoice ID
   */
  async retrieve(invoiceId: string): Promise<Invoice> {
    return this.client.get<Invoice>(`/api/v1/merchants/invoices/${invoiceId}`);
  }
}
