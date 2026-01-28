import { HttpClient } from '../client';
import { 
  SandboxPaymentSimulation,
  SimulatePaymentRequest
} from '../types';

export class Sandbox {
  constructor(private client: HttpClient) {}

  /**
   * Enable sandbox mode
   */
  async enable(): Promise<{ success: boolean }> {
    return this.client.request<{ success: boolean }>('POST', '/api/v1/merchant/sandbox/enable');
  }

  /**
   * Simulate payment status
   */
  async simulatePayment(paymentId: string, data: SimulatePaymentRequest): Promise<SandboxPaymentSimulation> {
    return this.client.request<SandboxPaymentSimulation>('POST', `/api/v1/merchant/sandbox/payments/${paymentId}/simulate`, data);
  }
}
