import * as crypto from 'crypto';
import { WebhookEvent, WebhookEventType } from '../types';
import { FidduPayError } from '../errors';

export class Webhooks {
  /**
   * Construct and verify a webhook event from the request body and signature
   */
  static constructEvent(
    payload: string | Buffer,
    signature: string,
    secret: string,
    tolerance: number = 300 // 5 minutes
  ): WebhookEvent {
    const payloadString = typeof payload === 'string' ? payload : payload.toString('utf8');
    
    if (!this.verifySignature(payloadString, signature, secret, tolerance)) {
      throw new FidduPayError('Invalid webhook signature', 'webhook_signature_verification_failed');
    }

    try {
      const event = JSON.parse(payloadString) as WebhookEvent;
      this.validateWebhookEvent(event);
      return event;
    } catch (error) {
      if (error instanceof FidduPayError) {
        throw error;
      }
      throw new FidduPayError('Invalid webhook payload', 'webhook_payload_invalid');
    }
  }

  /**
   * Verify webhook signature
   */
  static verifySignature(
    payload: string,
    signature: string,
    secret: string,
    tolerance: number = 300
  ): boolean {
    try {
      // Parse signature header (format: "t=timestamp,v1=signature")
      const elements = signature.split(',');
      const timestamp = elements.find(el => el.startsWith('t='))?.split('=')[1];
      const sig = elements.find(el => el.startsWith('v1='))?.split('=')[1];

      if (!timestamp || !sig) {
        return false;
      }

      // Check timestamp tolerance
      const timestampNum = parseInt(timestamp, 10);
      const now = Math.floor(Date.now() / 1000);
      
      if (Math.abs(now - timestampNum) > tolerance) {
        return false;
      }

      // Compute expected signature
      const signedPayload = `${timestamp}.${payload}`;
      const expectedSig = crypto
        .createHmac('sha256', secret)
        .update(signedPayload, 'utf8')
        .digest('hex');

      // Compare signatures using constant-time comparison
      return crypto.timingSafeEqual(
        Buffer.from(sig, 'hex'),
        Buffer.from(expectedSig, 'hex')
      );
    } catch (error) {
      return false;
    }
  }

  /**
   * Generate webhook signature for testing
   */
  static generateSignature(payload: string, secret: string): string {
    const timestamp = Math.floor(Date.now() / 1000);
    const signedPayload = `${timestamp}.${payload}`;
    const signature = crypto
      .createHmac('sha256', secret)
      .update(signedPayload, 'utf8')
      .digest('hex');
    
    return `t=${timestamp},v1=${signature}`;
  }

  private static validateWebhookEvent(event: any): void {
    if (!event.id || typeof event.id !== 'string') {
      throw new FidduPayError('Invalid webhook event: missing or invalid id', 'webhook_event_invalid');
    }

    if (!event.type || typeof event.type !== 'string') {
      throw new FidduPayError('Invalid webhook event: missing or invalid type', 'webhook_event_invalid');
    }

    const validTypes: WebhookEventType[] = [
      'payment.confirmed',
      'payment.expired', 
      'payment.failed',
      'refund.completed',
      'refund.failed'
    ];

    if (!validTypes.includes(event.type as WebhookEventType)) {
      throw new FidduPayError(`Invalid webhook event type: ${event.type}`, 'webhook_event_invalid');
    }

    if (!event.data) {
      throw new FidduPayError('Invalid webhook event: missing data', 'webhook_event_invalid');
    }

    if (!event.created_at || typeof event.created_at !== 'string') {
      throw new FidduPayError('Invalid webhook event: missing or invalid created_at', 'webhook_event_invalid');
    }
  }
}
