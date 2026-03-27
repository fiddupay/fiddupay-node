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
        alignItems: "center", justifyContent: "center", background: "rgba(0,0,0,0.5)",
      }}
      onClick={() => {
        setShowStatusModal(null);
        setStatusReason("");
      }}
    >
      <div
        style={{
          background: "white", borderRadius: "16px", padding: "2rem",
          maxWidth: "420px", width: "90%", boxShadow: "0 20px 60px rgba(0,0,0,0.15)",
        }}
        onClick={(e) => e.stopPropagation()}
      >
        <h3 style={{ margin: "0 0 0.5rem", fontSize: "1.1rem" }}>
          {showStatusModal === "flagged" && "🚩 Flag Customer"}
          {showStatusModal === "suspended" && "⏸️ Suspend Customer"}
          {showStatusModal === "blocked" && "🚫 Block Customer"}
        </h3>
        <p style={{ margin: "0 0 1.25rem", fontSize: "0.85rem", color: "#64748b" }}>
          {showStatusModal === "flagged" && "Customer will be limited to view-only access. They cannot withdraw or pay."}
          {showStatusModal === "suspended" && "Customer will lose all access. All operations will be rejected."}
          {showStatusModal === "blocked" && "Customer will be permanently blocked. All operations will be rejected."}
        </p>
        <div style={{ marginBottom: "1.25rem" }}>
          <label style={{ display: "block", marginBottom: "0.5rem", fontSize: "0.85rem", fontWeight: 600, color: "#334155" }}>
            Reason (optional)
          </label>
          <textarea
            value={statusReason}
            onChange={(e) => setStatusReason(e.target.value)}
            placeholder="Provide a reason for this action..."
            rows={3}
            style={{
              width: "100%", padding: "0.75rem", borderRadius: "10px", border: "1px solid #e2e8f0",
              fontSize: "0.9rem", resize: "none", fontFamily: "inherit", boxSizing: "border-box",
            }}
          ></textarea>
        </div>
        <div style={{ display: "flex", gap: "0.75rem" }}>
          <button
            onClick={() => {
              setShowStatusModal(null);
              setStatusReason("");
            }}
            style={{
              flex: 1, padding: "0.75rem", border: "1px solid #e2e8f0", borderRadius: "10px",
              background: "white", color: "#64748b", fontWeight: 600, cursor: "pointer",
            }}
          >Cancel</button>
          <button
            onClick={onConfirm}
            style={{
              flex: 1, padding: "0.75rem", border: "none", borderRadius: "10px",
              background: "#1e293b", color: "white", fontWeight: 600, cursor: "pointer",
            }}
          >Confirm Action</button>
        </div>
      </div>
    </div>
  );
};

export default StatusUpdateModal;
