import { HttpClient } from '../client';
import { RequestOptions } from '../types';

export interface UnifiedTransaction {
    type: 'payment' | 'refund' | 'withdrawal';
    id: string;
    crypto_amount: string;
    usd_amount: string;
    crypto_type: string;
    status: string;
    transaction_hash?: string;
    created_at: string;
}

export class Transactions {
    constructor(private client: HttpClient) { }

    /**
     * List unified transactions (payments, refunds, withdrawals)
     */
    async list(params?: { limit?: number; [key: string]: any }, options?: RequestOptions): Promise<{ transactions: UnifiedTransaction[] }> {
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
        return this.client.request('GET', path);
    }
}
