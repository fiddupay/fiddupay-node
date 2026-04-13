import React from 'react';
import { useNotificationStore } from '@/stores/notificationStore';
import { MdNotificationsNone, MdCheckCircle, MdError, MdInfo, MdClose, MdDoneAll, MdDeleteSweep } from 'react-icons/md';
import styles from '@/styles/components/layout/Header.module.css';

const NotificationPanel: React.FC = () => {
  const { 
    notifications, 
    isPanelOpen, 
    loading,
    markAsRead, 
    markAllAsRead, 
    clearAll, 
    togglePanel,
    deleteNotification
  } = useNotificationStore();

  if (!isPanelOpen) return null;

  const formatTime = (ts: string | number) => {
    const date = typeof ts === 'string' ? new Date(ts).getTime() : ts;
    const diff = Date.now() - date;
    if (diff < 60000) return 'Just now';
    if (diff < 3600000) return `${Math.floor(diff / 60000)}m ago`;
    if (diff < 86400000) return `${Math.floor(diff / 3600000)}h ago`;
    return new Date(date).toLocaleDateString();
  };

  return (
    <>
      {/* Backdrop for closing */}
      <div 
        style={{ position: 'fixed', inset: 0, zIndex: 998 }} 
        onClick={() => togglePanel(false)}
      />
      
      <div className={styles.notificationPanel}>
        <div className={styles.panelHeader}>
          <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
            <MdNotificationsNone size={20} />
            <h3 style={{ margin: 0, fontSize: '15px', fontWeight: 700 }}>Notifications</h3>
          </div>
          <div style={{ display: 'flex', gap: '4px' }}>
            <button 
              className={styles.iconActionBtn} 
              onClick={markAllAsRead} 
              title="Mark all as read"
              disabled={notifications.filter(n => !n.isRead).length === 0}
            >
              <MdDoneAll size={18} />
            </button>
            <button 
              className={styles.iconActionBtn} 
              onClick={clearAll} 
              title="Clear all"
              disabled={notifications.length === 0}
            >
              <MdDeleteSweep size={18} />
            </button>
            <button className={styles.iconActionBtn} onClick={() => togglePanel(false)}>
              <MdClose size={18} />
            </button>
          </div>
        </div>

        <div className={styles.panelBody}>
          {loading ? (
            <div className={styles.notificationList} style={{ opacity: 0.6 }}>
              {[1, 2, 3].map(i => (
                <div key={i} className={styles.notificationItem} style={{ borderBottom: '1px solid var(--fiddu-border-subtle)' }}>
                  <div className={styles.statusIcon} style={{ background: 'rgba(255, 255, 255, 0.05)', animation: 'pulse 1.5s infinite' }} />
                  <div style={{ flex: 1 }}>
                    <div style={{ width: '60%', height: '12px', background: 'rgba(255, 255, 255, 0.05)', marginBottom: '8px', borderRadius: '4px' }} />
                    <div style={{ width: '90%', height: '10px', background: 'rgba(255, 255, 255, 0.03)', borderRadius: '4px' }} />
                  </div>
                </div>
              ))}
            </div>
          ) : notifications.length === 0 ? (
            <div className={styles.emptyState}>
              <div className={styles.emptyIcon}>
                <MdNotificationsNone size={48} />
              </div>
              <p>No notifications yet</p>
              <span>We'll notify you when something important happens</span>
            </div>
          ) : (
            <div className={styles.notificationList}>
              {notifications.map((notif) => (
                <div 
                  key={notif.id} 
                  className={`${styles.notificationItem} ${!notif.isRead ? styles.unread : ''}`}
                  onClick={() => markAsRead(notif.id)}
                >
                  <div className={`${styles.statusIcon} ${styles[notif.type]}`}>
                    {notif.type === 'success' && <MdCheckCircle />}
                    {notif.type === 'error' && <MdError />}
                    {(notif.type === 'info' || notif.type === 'warning') && <MdInfo />}
                  </div>
                  <div style={{ flex: 1 }}>
                    <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start' }}>
                      <h4 style={{ margin: 0, fontSize: '13px', fontWeight: 600 }}>{notif.title}</h4>
                      <span className={styles.timestamp}>{formatTime(notif.timestamp)}</span>
                    </div>
                    <p style={{ margin: '4px 0 0', fontSize: '12px', color: 'var(--fiddu-text-secondary)', lineHeight: 1.4 }}>
                      {notif.message}
                    </p>
                  </div>
                  <button 
                    className={styles.itemDeleteBtn}
                    onClick={(e) => {
                      e.stopPropagation();
                      deleteNotification(notif.id);
                    }}
                  >
                    <MdClose size={14} />
                  </button>
                </div>
              ))}
            </div>
          )}
        </div>

        <div className={styles.panelFooter}>
          <button className={styles.viewAllBtn}>View System Logs</button>
        </div>
      </div>
    </>
  );
};

export default NotificationPanel;
