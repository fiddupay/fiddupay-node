import { create } from 'zustand'
import { notificationAPI } from '@/services/apiService'

export interface Notification {
  id: string
  title: string
  message: string
  type: 'success' | 'error' | 'info' | 'warning'
  event: string
  timestamp: string | number
  isRead: boolean
}

interface NotificationState {
  notifications: Notification[]
  isPanelOpen: boolean
  unreadCount: number
  loading: boolean
}

interface NotificationActions {
  fetchNotifications: () => Promise<void>
  addNotification: (notif: Omit<Notification, 'id' | 'timestamp' | 'isRead'>) => void
  markAsRead: (id: string) => Promise<void>
  markAllAsRead: () => Promise<void>
  deleteNotification: (id: string) => Promise<void>
  clearAll: () => Promise<void>
  togglePanel: (open?: boolean) => void
}

const mapBackendNotification = (n: any): Notification => ({
  id: n.id,
  title: n.title,
  message: n.message,
  type: n.notification_type as any,
  event: n.event_type,
  timestamp: n.created_at,
  isRead: n.is_read
})

export const useNotificationStore = create<NotificationState & NotificationActions>((set, get) => ({
  notifications: [],
  isPanelOpen: false,
  unreadCount: 0,
  loading: false,

  fetchNotifications: async () => {
    set({ loading: true })
    try {
      const res = await notificationAPI.list({ limit: 50 })
      const backendNotifs = res.data.notifications || []
      const mapped = backendNotifs.map(mapBackendNotification)
      
      set({ 
        notifications: mapped, 
        unreadCount: res.data.unread_count || 0,
        loading: false 
      })
    } catch (error) {
      console.error('Failed to fetch notifications:', error)
      set({ loading: false })
    }
  },

  addNotification: (notif) => {
    const newNotif: Notification = {
      ...notif,
      id: `local-${Math.random().toString(36).slice(2, 11)}`,
      timestamp: new Date().toISOString(),
      isRead: false
    }

    set((state) => {
      const updated = [newNotif, ...state.notifications].slice(0, 50)
      return {
        notifications: updated,
        unreadCount: updated.filter(n => !n.isRead).length
      }
    })
  },

  markAsRead: async (id) => {
    // Optimistic update
    set((state) => {
      const updated = state.notifications.map(n =>
        n.id === id ? { ...n, isRead: true } : n
      )
      return {
        notifications: updated,
        unreadCount: updated.filter(n => !n.isRead).length
      }
    })

    if (!id.startsWith('local-')) {
      try {
        await notificationAPI.markRead(id)
      } catch (error) {
        console.error('Failed to mark notification as read on backend:', error)
      }
    }
  },

  markAllAsRead: async () => {
    const originalNotifications = get().notifications
    const originalUnread = get().unreadCount

    // Optimistic update
    set((state) => ({
      notifications: state.notifications.map(n => ({ ...n, isRead: true })),
      unreadCount: 0
    }))

    try {
      await notificationAPI.markRead()
    } catch (error) {
      console.error('Failed to mark all as read on backend:', error)
      // Rollback on error
      set({ notifications: originalNotifications, unreadCount: originalUnread })
    }
  },

  deleteNotification: async (id) => {
    const originalNotifications = get().notifications

    // Optimistic update
    set((state) => {
      const updated = state.notifications.filter(n => n.id !== id)
      return {
        notifications: updated,
        unreadCount: updated.filter(n => !n.isRead).length
      }
    })

    if (!id.startsWith('local-')) {
      try {
        await notificationAPI.delete(id)
      } catch (error) {
        console.error('Failed to delete notification on backend:', error)
        set({ notifications: originalNotifications })
      }
    }
  },

  clearAll: async () => {
    const originalNotifications = get().notifications
    
    set({ notifications: [], unreadCount: 0 })

    try {
      await notificationAPI.delete()
    } catch (error) {
      console.error('Failed to clear notifications on backend:', error)
      set({ notifications: originalNotifications })
    }
  },

  togglePanel: (open) => {
    set((state) => ({
      isPanelOpen: open !== undefined ? open : !state.isPanelOpen
    }))
  }
}))
