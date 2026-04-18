import { HttpClient } from '../client';
import { RequestOptions, UnifiedTransaction, UnifiedTransactionsResponse } from '../types';

export class Transactions {
    constructor(private client: HttpClient) { }

    /**
     * List unified transactions (payments, refunds, withdrawals)
     * @returns A promise resolving to a list of {@link UnifiedTransaction} wrapped in a response object.
     */
    async list(params?: { limit?: number; [key: string]: any }, options?: RequestOptions): Promise<UnifiedTransactionsResponse> {
        const queryParams = new URLSearchParams();

        if (params) {
            for (const [key, value] of Object.entries(params)) {
                if (value !== undefined && value !== null) {
                    queryParams.append(key, value.toString());
                }
            }
        }
        
        const query = queryParams.toString();
        const path = query ? `/api/v1/merchants/transactions?${query}` : '/api/v1/merchants/transactions';
        return this.client.request<UnifiedTransactionsResponse>('GET', path);
    }
}
