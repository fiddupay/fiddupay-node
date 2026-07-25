import { HttpClient } from '../client';
import { NotificationListResponse, NotificationActionResult } from '../types';

/**
 * Notifications Resource
 *
 * Handles merchant dashboard notifications.
 */
export class Notifications {
  constructor(private client: HttpClient) {}

  /**
   * List merchant notifications
   * @param params Pagination and limit
   */
  async list(params?: { limit?: number; offset?: number }): Promise<NotificationListResponse> {
    const queryParams = new URLSearchParams();
    if (params?.limit) queryParams.append('limit', params.limit.toString());
    if (params?.offset) queryParams.append('offset', params.offset.toString());

    const query = queryParams.toString();
    const path = query ? `/api/v1/merchants/notifications?${query}` : '/api/v1/merchants/notifications';

    return this.client.get<NotificationListResponse>(path);
  }

  /**
   * Mark notifications as read
   * @param notificationId Optional specific notification ID to mark as read. If omitted, all are marked.
   */
  async markRead(notificationId?: string): Promise<NotificationActionResult> {
    const url = notificationId
      ? `/api/v1/merchants/notifications/${notificationId}/mark-read`
      : '/api/v1/merchants/notifications/mark-read';
    return this.client.post<NotificationActionResult>(url);
  }

  /**
   * Delete notifications
   * @param notificationId Optional specific notification ID to delete. If omitted, all are deleted.
   */
  async delete(notificationId?: string): Promise<NotificationActionResult> {
    const url = notificationId
      ? `/api/v1/merchants/notifications/${notificationId}`
      : '/api/v1/merchants/notifications';
    return this.client.delete<NotificationActionResult>(url);
  }
}
