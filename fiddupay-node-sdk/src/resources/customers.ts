import { HttpClient } from '../client';
import {
    MerchantCustomer,
    CreateCustomerRequest,
    ProvisionWalletRequest,
    CustomerBalanceResponse,
    CustomerWithdrawalRequest,
    CustomerSweepRequest,
    ListCustomersParams,
    PaginatedResponse,
    RequestOptions
} from '../types';

/**
 * Customers Resource
 * 
 * Allows merchants to manage their platform users and provision unique
 * designated deposit wallets for them.
 */
export class Customers {
    constructor(private client: HttpClient) { }

    /**
     * Register a new customer in the gateway.
     * This is required before provisioning wallets.
     */
    async register(data: CreateCustomerRequest, options?: RequestOptions): Promise<{ customer: MerchantCustomer; message: string }> {
        return this.client.post('/api/v1/merchants/customers', data, options);
    }

    /**
     * Provision designated wallets for a customer.
     * You can request "evm" (covers ETH, BSC, Polygon, Arb) and/or "solana".
     */
    async createWallets(externalId: string, data: ProvisionWalletRequest, options?: RequestOptions): Promise<{
        external_id: string;
        wallets: Array<{ crypto_type: string; network: string; address: string; created_at: string }>;
        message: string;
    }> {
        return this.client.post(`/api/v1/merchants/customers/${externalId}/wallets`, data, options);
    }

    /**
     * Retrieve the current balances for a customer across all their designated wallets.
     */
    async getBalances(externalId: string, options?: RequestOptions): Promise<CustomerBalanceResponse> {
        return this.client.get(`/api/v1/merchants/customers/${externalId}/balances`, options);
    }

    /**
     * List all registered customers for the merchant with pagination.
     */
    async list(params?: ListCustomersParams, options?: RequestOptions): Promise<PaginatedResponse<MerchantCustomer>> {
        const queryParams = new URLSearchParams();

        if (params?.limit) queryParams.append('limit', params.limit.toString());
        if (params?.offset) queryParams.append('offset', params.offset.toString());

        const query = queryParams.toString();
        const path = query ? `/api/v1/merchants/customers?${query}` : '/api/v1/merchants/customers';

        return this.client.get<PaginatedResponse<MerchantCustomer>>(path, options);
    }

    /**
     * Withdraw funds directly from a customer's designated wallet balance.
     */
    async withdraw(externalId: string, data: CustomerWithdrawalRequest, options?: RequestOptions): Promise<{ withdrawal: any; message: string }> {
        return this.client.post(`/api/v1/merchants/customers/${externalId}/withdraw`, data, options);
    }

    /**
     * Sweep funds from a user's sub-account balance directly into the merchant's master balance.
     * If amount is omitted, sweeps the entire available balance.
     */
    async sweep(externalId: string, data: CustomerSweepRequest, options?: RequestOptions): Promise<{ swept_amount: string; message: string }> {
        return this.client.post(`/api/v1/merchants/customers/${externalId}/sweep`, data, options);
    }

    /**
     * Deactivate a customer.
     * This preserves their history but prevents further activity.
     */
    async deactivate(externalId: string, options?: RequestOptions): Promise<{ message: string }> {
        return this.client.post(`/api/v1/merchants/customers/${externalId}/deactivate`, {}, options);
    }
}
