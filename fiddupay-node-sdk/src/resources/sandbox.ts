import { HttpClient } from '../client';
import {
  SandboxPaymentSimulation,
  SimulatePaymentRequest
} from '../types';

export class Sandbox {
  constructor(private client: HttpClient) { }


  /**
   * Simulate payment status
   */
  async simulatePayment(paymentId: string, data: SimulatePaymentRequest): Promise<SandboxPaymentSimulation> {
    return this.client.request<SandboxPaymentSimulation>('POST', `/api/v1/merchants/sandbox/payments/${paymentId}/simulate`, data);
  }
}
