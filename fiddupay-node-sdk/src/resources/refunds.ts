import { HttpClient } from '../client';
import { CreateRefundRequest, Refund, RequestOptions } from '../types';
import { FidduPayValidationError } from '../errors';

export class Refunds {
  constructor(private client: HttpClient) { }

  /**
   * Create a refund for a confirmed payment
   */
  async create(data: CreateRefundRequest, options?: RequestOptions): Promise<Refund> {
    this.validateCreateRefund(data);
    return this.client.post<Refund>('/api/v1/merchants/refunds', data, options);
  }

  /**
   * Retrieve a refund by ID
   */
  async retrieve(refundId: string, options?: RequestOptions): Promise<Refund> {
    if (!refundId) {
      throw new FidduPayValidationError('Refund ID is required', 'refund_id');
    }
    return this.client.get<Refund>(`/api/v1/merchants/refunds/${refundId}`, options);
  }

  /**
   * Complete a refund
   */
  async complete(refundId: string, transactionHash: string, options?: RequestOptions): Promise<{ success: boolean }> {
    if (!refundId) {
      throw new FidduPayValidationError('Refund ID is required', 'refund_id');
    }
    if (!transactionHash) {
      throw new FidduPayValidationError('Transaction hash is required', 'transaction_hash');
    }
    return this.client.request<{ success: boolean }>('POST', `/api/v1/merchants/refunds/${refundId}/complete`, { transaction_hash: transactionHash });
  }

  /**
   * List refunds
   */
  async list(params?: { limit?: number; offset?: number; [key: string]: any }, options?: RequestOptions): Promise<{
    refunds: Refund[];
    total: number;
    has_more: boolean;
  }> {
    const queryParams = new URLSearchParams();

    if (params) {
      for (const [key, value] of Object.entries(params)) {
        if (value !== undefined && value !== null) {
          queryParams.append(key, value.toString());
        }
      }
    }

    const query = queryParams.toString();
    const path = query ? `/api/v1/merchants/refunds?${query}` : '/api/v1/merchants/refunds';

    return this.client.get(path, options);
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
