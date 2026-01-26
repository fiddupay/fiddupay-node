import { HttpClient } from './client';
import { FidduPayConfig } from './types';
import { FidduPayValidationError } from './errors';
import { Payments } from './resources/payments';
import { Merchants } from './resources/merchants';
import { Refunds } from './resources/refunds';
import { AnalyticsResource } from './resources/analytics';
import { Webhooks } from './resources/webhooks';

export class FidduPayClient {
  private client: HttpClient;
  
  public readonly payments: Payments;
  public readonly merchants: Merchants;
  public readonly refunds: Refunds;
  public readonly analytics: AnalyticsResource;
  public readonly webhooks = Webhooks;

  constructor(config: FidduPayConfig) {
    this.validateConfig(config);
    
    this.client = new HttpClient(config);
    
    // Initialize resource classes
    this.payments = new Payments(this.client);
    this.merchants = new Merchants(this.client);
    this.refunds = new Refunds(this.client);
    this.analytics = new AnalyticsResource(this.client);
  }

  private validateConfig(config: FidduPayConfig): void {
    if (!config.apiKey) {
      throw new FidduPayValidationError('API key is required');
    }

    // Validate API key format - must start with sk_ or live_
    if (!config.apiKey.startsWith('sk_') && !config.apiKey.startsWith('live_')) {
      throw new FidduPayValidationError('Invalid API key format. API key must start with "sk_" or "live_"');
    }

    if (config.environment && !['sandbox', 'production'].includes(config.environment)) {
      throw new FidduPayValidationError('Environment must be either "sandbox" or "production"');
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
