import { HttpClient } from './client';
import { FidduPayConfig } from './types';
import { FidduPayValidationError } from './errors';
import { Payments } from './resources/payments';
import { Merchants } from './resources/merchants';
import { Refunds } from './resources/refunds';
import { AnalyticsResource } from './resources/analytics';
import { InvoicesResource } from './resources/invoices';
import { Webhooks } from './resources/webhooks';
import { Wallets } from './resources/wallets';
import { Withdrawals } from './resources/withdrawals';
import { Security } from './resources/security';
import { Balances, AuditLogs } from './resources/balances';
import { Customers } from './resources/customers';
import { WebSockets } from './resources/websockets';
import { Sandbox } from './resources/sandbox';
import { Contact } from './resources/contact';
import { Transactions } from './resources/transactions';
import { Public } from './resources/public';
import { AddressOnly } from './resources/address_only';
import { Notifications } from './resources/notifications';

export class FidduPayClient {
  private client: HttpClient;

  public readonly payments: Payments;
  public readonly merchants: Merchants;
  public readonly refunds: Refunds;
  public readonly analytics: AnalyticsResource;
  public readonly invoices: InvoicesResource;
  public readonly webhooks = Webhooks;
  public readonly wallets: Wallets;
  /**
   * Merchant's own withdrawal operations (withdraw from merchant main balance
   * to an external destination). This is *not* related to customer sub-wallets.
   * To move funds from a customer sub-wallet use `customers.sweep()` instead.
   */
  public readonly withdrawals: Withdrawals;
  public readonly security: Security;
  /**
   * Customer sub-account management.
   *
   * Key operations:
   *  - `payMerchant()` — locks customer funds into the merchant's reserved balance
   *  - `sweep(sweep_mode)` — sweeps locked funds to the merchant's Master Wallet
   *    on-chain. Replaces the removed `withdraw()` method (v2.5.8+).
   *  - Available sweep modes: `ALL` | `NATIVE_ONLY` | `STABLE_ONLY` | `SPECIFIC`
   */
  public readonly customers: Customers;
  public readonly balances: Balances;
  public readonly auditLogs: AuditLogs;
  public readonly sandbox: Sandbox;
  public readonly contact: Contact;
  public readonly transactions: Transactions;
  public readonly public: Public;
  public readonly addressOnly: AddressOnly;
  public readonly websockets: WebSockets;
  public readonly notifications: Notifications;

  constructor(config: FidduPayConfig) {
    this.validateConfig(config);

    this.client = new HttpClient(config);

    // Initialize resource classes
    this.payments = new Payments(this.client);
    this.merchants = new Merchants(this.client);
    this.refunds = new Refunds(this.client);
    this.analytics = new AnalyticsResource(this.client);
    this.invoices = new InvoicesResource(this.client);
    this.wallets = new Wallets(this.client);
    this.withdrawals = new Withdrawals(this.client);
    this.security = new Security(this.client);
    this.customers = new Customers(this.client);
    this.balances = new Balances(this.client);
    this.auditLogs = new AuditLogs(this.client);
    this.sandbox = new Sandbox(this.client);
    this.contact = new Contact(this.client);
    this.transactions = new Transactions(this.client);
    this.public = new Public(this.client);
    this.addressOnly = new AddressOnly(this.client);
    this.websockets = new WebSockets(this.client);
    this.notifications = new Notifications(this.client);
  }

  private validateConfig(config: FidduPayConfig): void {
    if (!config.apiKey) {
      throw new FidduPayValidationError('API key is required');
    }

    // Allow special registration key for merchant registration
    if (config.apiKey === 'registration_key') {
      return; // Skip validation for registration
    }

    // Validate API key format
    if (!config.apiKey.startsWith('sk_') && !config.apiKey.startsWith('live_')) {
      throw new FidduPayValidationError('Invalid API key format. API key must start with "sk_" (sandbox) or "live_" (production)');
    }

    if (config.timeout && (config.timeout < 1000 || config.timeout > 60000)) {
      throw new FidduPayValidationError('Timeout must be between 1000ms and 60000ms');
    }

    if (config.maxRetries && (config.maxRetries < 0 || config.maxRetries > 10)) {
      throw new FidduPayValidationError('Max retries must be between 0 and 10');
    }
  }
}

// Export everything
export * from './types';
export * from './errors';
export { Webhooks } from './resources/webhooks';
export { Public } from './resources/public';

// Backward compatibility alias
export { FidduPayClient as FidduPay };

// Default export
export default FidduPayClient;
