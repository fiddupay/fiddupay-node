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
    async list(options?: RequestOptions): Promise<{ transactions: UnifiedTransaction[] }> {
        return this.client.request('GET', '/api/v1/merchants/transactions');
    }
}
