import React from 'react';

interface StatusModalProps {
  showStatusModal: string | null;
  setShowStatusModal: (val: string | null) => void;
  statusReason: string;
  setStatusReason: (val: string) => void;
  onConfirm: () => void;
}

const StatusUpdateModal: React.FC<StatusModalProps> = ({
  showStatusModal,
  setShowStatusModal,
  statusReason,
  setStatusReason,
  onConfirm
}) => {
  if (!showStatusModal) return null;

  return (
    <div
      style={{
        position: "fixed", inset: 0, zIndex: 1100, display: "flex",
        alignItems: "center", justifyContent: "center", background: "rgba(0,0,0,0.6)",
        backdropFilter: "blur(8px)"
      }}
      onClick={() => {
        setShowStatusModal(null);
        setStatusReason("");
      }}
    >
      <div
        style={{
          background: "var(--surface)", borderRadius: "24px", padding: "2rem",
          maxWidth: "420px", width: "90%", boxShadow: "0 20px 60px rgba(0,0,0,0.5)",
          border: "1px solid var(--border)", color: "var(--text-main)"
        }}
        onClick={(e) => e.stopPropagation()}
      >
        <h3 style={{ margin: "0 0 0.5rem", fontSize: "1.2rem", fontWeight: 800 }}>
          {showStatusModal === "flagged" && "🚩 Flag Customer"}
          {showStatusModal === "suspended" && "⏸️ Suspend Customer"}
          {showStatusModal === "blocked" && "🚫 Block Customer"}
        </h3>
        <p style={{ margin: "0 0 1.5rem", fontSize: "0.9rem", color: "var(--text-muted)" }}>
          {showStatusModal === "flagged" && "Customer will be limited to view-only access. They cannot withdraw or pay."}
          {showStatusModal === "suspended" && "Customer will lose all access. All operations will be rejected."}
          {showStatusModal === "blocked" && "Customer will be permanently blocked. All operations will be rejected."}
        </p>
        <div style={{ marginBottom: "1.5rem" }}>
          <label style={{ display: "block", marginBottom: "0.75rem", fontSize: "0.85rem", fontWeight: 700, color: "var(--text-muted)" }}>
            Reason (optional)
          </label>
          <textarea
            value={statusReason}
            onChange={(e) => setStatusReason(e.target.value)}
            placeholder="Provide a reason for this action..."
            rows={3}
            style={{
              width: "100%", padding: "1rem", borderRadius: "12px", border: "1px solid var(--border)",
              fontSize: "0.95rem", resize: "none", fontFamily: "inherit", boxSizing: "border-box",
              background: "var(--bg-main)", color: "var(--text-main)", outline: "none"
            }}
          ></textarea>
        </div>
        <div style={{ display: "flex", gap: "1rem" }}>
          <button
            onClick={() => {
              setShowStatusModal(null);
              setStatusReason("");
            }}
            style={{
              flex: 1, padding: "0.875rem", border: "1px solid var(--border)", borderRadius: "12px",
              background: "transparent", color: "var(--text-main)", fontWeight: 700, cursor: "pointer",
            }}
          >Cancel</button>
          <button
            onClick={onConfirm}
            style={{
              flex: 1, padding: "0.875rem", border: "none", borderRadius: "12px",
              background: "#ef4444", color: "white", fontWeight: 700, cursor: "pointer",
              boxShadow: "0 4px 15px rgba(239, 68, 68, 0.3)"
            }}
          >Confirm Action</button>
        </div>
      </div>
    </div>
  );
};

export default StatusUpdateModal;
