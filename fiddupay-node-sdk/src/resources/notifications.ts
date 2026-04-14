import { HttpClient } from '../client';
import { NotificationListResponse } from '../types';

export class Notifications {
  constructor(private client: HttpClient) {}

  /**
   * List notification history for the merchant
   * @param params optional limit and offset
   */
  async list(params?: { limit?: number; offset?: number }): Promise<NotificationListResponse> {
    const queryParams = new URLSearchParams();
    if (params?.limit) queryParams.append('limit', params.limit.toString());
    if (params?.offset) queryParams.append('offset', params.offset.toString());
    
    const url = `/api/v1/merchants/notifications${queryParams.toString() ? `?${queryParams.toString()}` : ''}`;
    return this.client.get<NotificationListResponse>(url);
  }

  /**
   * Mark notifications as read
   * @param notificationId optional specific ID to mark as read
   */
  async markRead(notificationId?: string): Promise<{ success: boolean; message: string }> {
    const url = notificationId 
      ? `/api/v1/merchants/notifications/${notificationId}/mark-read`
      : '/api/v1/merchants/notifications/mark-read';
    return this.client.post(url, {});
  }

  /**
   * Delete notifications
   * @param notificationId optional specific ID to delete
   */
  async delete(notificationId?: string): Promise<{ success: boolean; message: string }> {
    const url = notificationId
      ? `/api/v1/merchants/notifications/${notificationId}`
      : '/api/v1/merchants/notifications';
    return this.client.delete(url);
  }
}
