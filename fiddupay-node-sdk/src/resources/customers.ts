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
    RequestOptions,
    CustomerTransaction,
    CustomerStatusRequest,
    CustomerPermissionsRequest,
    CustomerWalletsResponse
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
    async createWallets(externalId: string, data: ProvisionWalletRequest, options?: RequestOptions): Promise<CustomerWalletsResponse & { message: string }> {
        return this.client.post(`/api/v1/merchants/customers/${externalId}/wallets`, data, options);
    }

    /**
     * Retrieve the provisioned designated wallets for a customer.
     */
    async getWallets(externalId: string, options?: RequestOptions): Promise<CustomerWalletsResponse> {
        return this.client.get(`/api/v1/merchants/customers/${externalId}/wallets`, options);
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
    async list(params?: ListCustomersParams, options?: RequestOptions): Promise<{
        customers: MerchantCustomer[];
        total: number;
        has_more: boolean;
        limit: number;
        offset: number;
    }> {
        const queryParams = new URLSearchParams();

        if (params?.limit) queryParams.append('limit', params.limit.toString());
        if (params?.offset) queryParams.append('offset', params.offset.toString());

        const query = queryParams.toString();
        const path = query ? `/api/v1/merchants/customers?${query}` : '/api/v1/merchants/customers';

        return this.client.get<{
            customers: MerchantCustomer[];
            total: number;
            has_more: boolean;
            limit: number;
            offset: number;
        }>(path, options);
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

    /**
     * Get transaction history for a specific customer.
     */
    async getTransactions(externalId: string, params?: { limit?: number; offset?: number }, options?: RequestOptions): Promise<{
        transactions: CustomerTransaction[];
        total: number;
        limit: number;
        offset: number;
        external_id: string;
    }> {
        const queryParams = new URLSearchParams();
        if (params?.limit) queryParams.append('limit', params.limit.toString());
        if (params?.offset) queryParams.append('offset', params.offset.toString());
        
        const path = `/api/v1/merchants/customers/${externalId}/transactions${queryParams.toString() ? `?${queryParams.toString()}` : ''}`;
        return this.client.get<{
            transactions: CustomerTransaction[];
            total: number;
            limit: number;
            offset: number;
            external_id: string;
        }>(path, options);
    }

    /**
     * Update customer status (active, suspended, inactive).
     */
    async updateStatus(externalId: string, data: CustomerStatusRequest, options?: RequestOptions): Promise<{ message: string }> {
        return this.client.patch(`/api/v1/merchants/customers/${externalId}/status`, data, options);
    }

    /**
     * Update customer-specific permissions and limits.
     */
    async updatePermissions(externalId: string, data: CustomerPermissionsRequest, options?: RequestOptions): Promise<{ message: string }> {
        return this.client.patch(`/api/v1/merchants/customers/${externalId}/permissions`, data, options);
    }

    /**
     * Get the specific deposit address for a customer for a given cryptocurrency.
     */
    async getDepositAddress(externalId: string, cryptoType: string, options?: RequestOptions): Promise<{ address: string; crypto_type: string }> {
        const response = await this.client.get<{ deposit_address: string; crypto_type: string }>(`/api/v1/merchants/customers/${externalId}/deposit-address/${cryptoType}`, options);
        return { address: response.deposit_address, crypto_type: response.crypto_type };
    }

    /**
     * Initiate an internal payment from a customer's designated wallet balance to the merchant's master balance.
     * This is useful for charging users for services on your platform.
     */
    async payMerchant(externalId: string, data: {
        crypto_type: string;
        amount: string;
        reference_id?: string;
        description?: string;
    }, options?: RequestOptions): Promise<{ transaction: any; message: string }> {
        return this.client.post(`/api/v1/merchants/customers/${externalId}/pay-merchant`, data, options);
    }
}
