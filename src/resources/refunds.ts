import { HttpClient } from '../client';
import { CreateRefundRequest, Refund } from '../types';
import { FidduPayValidationError } from '../errors';

export class Refunds {
  constructor(private client: HttpClient) {}

  /**
   * Create a refund for a confirmed payment
   */
  async create(data: CreateRefundRequest): Promise<Refund> {
    this.validateCreateRefund(data);
    return this.client.post<Refund>('/api/v1/refunds', data);
  }

  /**
   * Retrieve a refund by ID
   */
  async retrieve(refundId: string): Promise<Refund> {
    if (!refundId) {
      throw new FidduPayValidationError('Refund ID is required', 'refund_id');
    }
    return this.client.get<Refund>(`/api/v1/refunds/${refundId}`);
  }

  /**
   * Complete a refund
   */
  async complete(refundId: string): Promise<any> {
    if (!refundId) {
      throw new FidduPayValidationError('Refund ID is required', 'refund_id');
    }
    return this.client.request('POST', `/api/v1/refunds/${refundId}/complete`);
  }

  /**
   * List refunds
   */
  async list(params?: { limit?: number; offset?: number }): Promise<{
    refunds: Refund[];
    total: number;
    has_more: boolean;
  }> {
    const queryParams = new URLSearchParams();
    
    if (params?.limit) queryParams.append('limit', params.limit.toString());
    if (params?.offset) queryParams.append('offset', params.offset.toString());

    const query = queryParams.toString();
    const path = query ? `/refunds?${query}` : '/refunds';
    
    return this.client.get(path);
  }

  private validateCreateRefund(data: CreateRefundRequest): void {
    if (!data.payment_id) {
      throw new FidduPayValidationError('Payment ID is required', 'payment_id');
    }

    if (data.amount !== undefined) {
      const amount = parseFloat(data.amount);
      if (isNaN(amount) || amount <= 0) {
        throw new FidduPayValidationError('Refund amount must be a positive number', 'amount');
      }
    }

    if (data.reason && data.reason.length > 500) {
      throw new FidduPayValidationError(
        'Refund reason must be 500 characters or less',
        'reason'
      );
    }
  }
}
