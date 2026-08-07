import * as crypto from 'crypto';
import { WebhookEvent, WebhookEventType } from '../types';
import { FidduPayError } from '../errors';

export interface WebhookSignatureComponents {
  timestamp: number;
  signatures: string[];
}

export class Webhooks {
  /**
   * Construct and verify a webhook event from the request body and signature.
   * 
   * @example
   * ```typescript
   * // In your Express webhook handler:
   * app.post('/webhook', express.raw({ type: 'application/json' }), (req, res) => {
   *   try {
   *     const event = Webhooks.constructEvent(
   *       req.body,
   *       req.headers['signature'] as string,
   *       process.env.WEBHOOK_SECRET!
   *     );
   *     // Handle the event...
   *     res.json({ received: true });
   *   } catch (err) {
   *     res.status(400).send(`Webhook Error: ${err.message}`);
   *   }
   * });
   * ```
   */
  static constructEvent(
    payload: string | Buffer,
    signature: string,
    secret: string,
    tolerance: number = 300 // 5 minutes
  ): WebhookEvent {
    const payloadString = typeof payload === 'string' ? payload : payload.toString('utf8');
    
    // Verify signature (throws descriptive errors on failure)
    this.verifySignature(payloadString, signature, secret, tolerance);

    try {
      const event = JSON.parse(payloadString) as WebhookEvent;
      this.validateWebhookEvent(event);
      return event;
    } catch (error) {
      if (error instanceof FidduPayError) {
        throw error;
      }
      throw new FidduPayError('Invalid webhook payload: failed to parse JSON', 'webhook_payload_invalid');
    }
  }

  /**
   * Parse the signature header into its timestamp and signature components.
   */
  static parseSignatureHeader(header: string): WebhookSignatureComponents {
    const elements = header.split(',');
    const timestampStr = elements.find(el => el.startsWith('t='))?.slice(2);
    const signatures = elements
      .filter(el => el.startsWith('v1='))
      .map(el => el.slice(3));

    if (!timestampStr) {
      throw new FidduPayError(
        'Invalid signature header: missing timestamp (t=...)',
        'webhook_signature_missing_timestamp'
      );
    }

    const timestamp = parseInt(timestampStr, 10);
    if (isNaN(timestamp)) {
      throw new FidduPayError(
        'Invalid signature header: timestamp is not a valid integer',
        'webhook_signature_invalid_timestamp'
      );
    }

    if (signatures.length === 0) {
      throw new FidduPayError(
        'Invalid signature header: no v1 signatures found',
        'webhook_signature_missing_v1'
      );
    }

    return { timestamp, signatures };
  }

  /**
   * Verify webhook signature. Throws descriptive errors on failure.
   */
  static verifySignature(
    payload: string,
    signature: string,
    secret: string,
    tolerance: number = 300
  ): boolean {
    if (!signature) {
      throw new FidduPayError(
        'No signature header provided. Ensure you are passing the raw "signature" header.',
        'webhook_signature_missing'
      );
    }

    if (!secret) {
      throw new FidduPayError(
        'No webhook secret provided. Set your webhook signing secret from the dashboard.',
        'webhook_secret_missing'
      );
    }

    const { timestamp, signatures } = this.parseSignatureHeader(signature);

    // Check timestamp tolerance (replay attack prevention)
    const now = Math.floor(Date.now() / 1000);
    if (Math.abs(now - timestamp) > tolerance) {
      throw new FidduPayError(
        `Webhook timestamp too old. The event was signed ${Math.abs(now - timestamp)}s ago (tolerance: ${tolerance}s). This could indicate a replay attack or a clock drift issue.`,
        'webhook_timestamp_expired'
      );
    }

    // Compute expected signature
    const signedPayload = `${timestamp}.${payload}`;
    const expectedSig = crypto
      .createHmac('sha256', secret)
      .update(signedPayload, 'utf8')
      .digest('hex');

    // Check if ANY of the v1 signatures match (supports key rotation)
    const matched = signatures.some(sig => {
      try {
        const sigBuf = Buffer.from(sig, 'hex');
        const expectedBuf = Buffer.from(expectedSig, 'hex');
        // Constant-time comparison requires equal length buffers
        if (sigBuf.length !== expectedBuf.length) return false;
        return crypto.timingSafeEqual(sigBuf, expectedBuf);
      } catch {
        return false;
      }
    });

    if (!matched) {
      throw new FidduPayError(
        'Webhook signature verification failed. Ensure you are using the correct signing secret and passing the raw request body (not parsed JSON).',
        'webhook_signature_verification_failed'
      );
    }

    return true;
  }

  /**
   * Generate a webhook signature for testing purposes.
   */
  static generateSignature(payload: string, secret: string, timestamp?: number): string {
    const ts = timestamp ?? Math.floor(Date.now() / 1000);
    const signedPayload = `${ts}.${payload}`;
    const signature = crypto
      .createHmac('sha256', secret)
      .update(signedPayload, 'utf8')
      .digest('hex');
    
    return `t=${ts},v1=${signature}`;
  }

  private static validateWebhookEvent(event: any): void {
    const eventType = event.type || event.event_type;

    if (!eventType || typeof eventType !== 'string') {
      throw new FidduPayError('Invalid webhook event: missing or invalid type or event_type', 'webhook_event_invalid');
    }

    const validTypes: WebhookEventType[] = [
      'payment.confirmed',
      'payment.expired', 
      'refund.completed',
      'merchant.deposit',
      'customer.deposit',
      'withdrawal.completed',
      'withdrawal.failed',
      'withdrawal_failed',
      'address_only_payment_status',
      'webhook.test'
    ];

    if (!validTypes.includes(eventType as WebhookEventType)) {
      throw new FidduPayError(`Invalid webhook event type: ${eventType}`, 'webhook_event_invalid');
    }
  }
}

