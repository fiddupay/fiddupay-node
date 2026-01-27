import { HttpClient } from './client';
import { FidduPayConfig } from './types';
import { FidduPayValidationError } from './errors';
import { Payments } from './resources/payments';
import { Merchants } from './resources/merchants';
import { Refunds } from './resources/refunds';
import { AnalyticsResource } from './resources/analytics';
import { Webhooks } from './resources/webhooks';
import { Wallets } from './resources/wallets';
import { Withdrawals } from './resources/withdrawals';
import { Security } from './resources/security';
import { Balances, AuditLogs } from './resources/balances';
import { Sandbox } from './resources/sandbox';
import { Contact, Pricing } from './resources/contact';

export class FidduPayClient {
  private client: HttpClient;
  
  public readonly payments: Payments;
  public readonly merchants: Merchants;
  public readonly refunds: Refunds;
  public readonly analytics: AnalyticsResource;
  public readonly webhooks = Webhooks;
  public readonly wallets: Wallets;
  public readonly withdrawals: Withdrawals;
  public readonly security: Security;
  public readonly balances: Balances;
  public readonly auditLogs: AuditLogs;
  public readonly sandbox: Sandbox;
  public readonly contact: Contact;
  public readonly pricing: Pricing;

  constructor(config: FidduPayConfig) {
    this.validateConfig(config);
    
    this.client = new HttpClient(config);
    
    // Initialize resource classes
    this.payments = new Payments(this.client);
    this.merchants = new Merchants(this.client);
    this.refunds = new Refunds(this.client);
    this.analytics = new AnalyticsResource(this.client);
    this.wallets = new Wallets(this.client);
    this.withdrawals = new Withdrawals(this.client);
    this.security = new Security(this.client);
    this.balances = new Balances(this.client);
    this.auditLogs = new AuditLogs(this.client);
    this.sandbox = new Sandbox(this.client);
    this.contact = new Contact(this.client);
    this.pricing = new Pricing(this.client);
  }

  private validateConfig(config: FidduPayConfig): void {
    if (!config.apiKey) {
      throw new FidduPayValidationError('API key is required');
    }

    // Allow special registration key for merchant registration
    if (config.apiKey === 'registration_key') {
      return; // Skip validation for registration
    }

    // Updated API key validation - must start with sk_ for sandbox or live_ for production
    if (!config.apiKey.startsWith('sk_') && !config.apiKey.startsWith('live_')) {
      throw new FidduPayValidationError('Invalid API key format. API key must start with "sk_" (sandbox) or "live_" (production)');
    }

    // Auto-detect environment from API key if not specified
    if (!config.environment) {
      config.environment = config.apiKey.startsWith('sk_') ? 'sandbox' : 'production';
    }

    if (config.environment && !['sandbox', 'production'].includes(config.environment)) {
      throw new FidduPayValidationError('Environment must be either "sandbox" or "production"');
    }

    // Validate API key matches environment
    if (config.environment === 'sandbox' && !config.apiKey.startsWith('sk_')) {
      throw new FidduPayValidationError('Sandbox environment requires API key starting with "sk_"');
    }

    if (config.environment === 'production' && !config.apiKey.startsWith('live_')) {
      throw new FidduPayValidationError('Production environment requires API key starting with "live_"');
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

// Backward compatibility alias
export { FidduPayClient as FidduPay };

// Default export
export default FidduPayClient;
