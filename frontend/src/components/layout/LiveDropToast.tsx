import React, { useEffect, useState } from 'react';
import { MdCheckCircle, MdClose } from 'react-icons/md';

interface ToastMessage {
  id: string;
  amount: string;
  crypto_type: string;
  event: string;
}

export const LiveDropToast: React.FC = () => {
  const [toasts, setToasts] = useState<ToastMessage[]>([]);

  useEffect(() => {
    const token = localStorage.getItem('fiddupay_dashboard_token') || sessionStorage.getItem('fiddupay_dashboard_token');
    if (!token) return;

    let socket: WebSocket | null = null;
    let reconnectTimeout: ReturnType<typeof setTimeout>;

    const connect = () => {
      const apiUrl = import.meta.env.VITE_API_URL || 'https://api.fiddupay.com';
      const wsUrl = apiUrl.replace(/^http/, 'ws') + `/api/v1/merchants/ws`;

      console.log('Connecting to notification stream...');
      const ws = new WebSocket(wsUrl, token);
      socket = ws;

      ws.onmessage = (event) => {
        try {
          const data = JSON.parse(event.data);
          if (data.event === 'merchant.deposit' || data.event === 'customer.deposit') {
            const newToast: ToastMessage = {
              id: Math.random().toString(),
              amount: data.amount,
              crypto_type: data.crypto_type,
              event: data.event,
            };
            setToasts((prev) => [...prev, newToast]);

            // Auto-remove after 6 seconds
            setTimeout(() => {
              setToasts((prev) => prev.filter((t) => t.id !== newToast.id));
            }, 6000);
          }
        } catch (err) {
          // Silent catch for malformed JSON
        }
      };

      ws.onclose = () => {
        console.warn('Notification stream closed. Retrying in 5s...');
        reconnectTimeout = setTimeout(connect, 5000);
      };

      ws.onerror = (err) => {
        console.error('Notification stream error:', err);
        ws.close();
      };
    };

    connect();

    return () => {
      if (socket) {
        socket.onclose = null; // Prevent reconnect on intentional close
        socket.close();
      }
      clearTimeout(reconnectTimeout);
    };
  }, []);

  if (toasts.length === 0) return null;

  return (
    <div style={{
      position: 'fixed',
      top: '20px',
      left: '50%',
      transform: 'translateX(-50%)',
      zIndex: 9999,
      display: 'flex',
      flexDirection: 'column',
      gap: '10px',
      pointerEvents: 'none',
    }}>
      {toasts.map((toast) => (
        <div key={toast.id} style={{
          backgroundColor: '#0a0b0d',
          color: '#ffffff',
          padding: '16px 24px',
          borderRadius: '12px',
          boxShadow: '0 10px 30px rgba(0,0,0,0.5)',
          border: '1px solid rgba(255,255,255,0.05)',
          display: 'flex',
          alignItems: 'center',
          gap: '12px',
          minWidth: '320px',
          animation: 'dropIn 0.5s cubic-bezier(0.175, 0.885, 0.32, 1.275)',
          pointerEvents: 'auto',
        }}>
          <MdCheckCircle style={{ color: '#00D588', fontSize: '24px' }} />
          <div style={{ flex: 1 }}>
            <h4 style={{ margin: 0, fontSize: '13px', fontWeight: 600, color: '#e2e8f0' }}>
              {toast.event === 'merchant.deposit' ? '🎉 A New Deposit Just Arrived' : '🛍️ Client Payment'}
            </h4>
            <p style={{ margin: '4px 0 0', fontSize: '16px', fontWeight: 700 }}>
              +{toast.amount} <span style={{ color: '#00D588' }}>{toast.crypto_type}</span>
            </p>
          </div>
          <button
            onClick={() => setToasts((prev) => prev.filter((t) => t.id !== toast.id))}
            style={{
              background: 'none',
              border: 'none',
              color: '#495670',
              cursor: 'pointer',
              padding: '4px',
              display: 'flex',
              alignItems: 'center',
            }}
          >
            <MdClose style={{ fontSize: '18px' }} />
          </button>
        </div>
      ))}

      <style>{`
        @keyframes dropIn {
          0% { transform: translateY(-50px) translateX(0px); opacity: 0; }
          100% { transform: translateY(0) translateX(0px); opacity: 1; }
        }
      `}</style>
    </div>
  );
};
