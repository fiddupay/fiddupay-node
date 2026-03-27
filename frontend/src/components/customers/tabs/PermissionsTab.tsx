import React from 'react';
import styles from "../styles/CustomerDrawers.module.css";
import { Customer } from "../types";
import { getStatusStyle } from "../utils";

interface PermissionsTabProps {
  selectedCustomer: Customer;
  statusUpdating: boolean;
  onStatusUpdate: (status: string) => void;
  onShowStatusModal: (status: string) => void;
  permUpdating: boolean;
  onToggleWithdraw: () => void;
}

const PermissionsTab: React.FC<PermissionsTabProps> = ({
  selectedCustomer,
  statusUpdating,
  onStatusUpdate,
  onShowStatusModal,
  permUpdating,
  onToggleWithdraw
}) => {
  const st = getStatusStyle(selectedCustomer.status || "active");

  return (
    <>
      <div className={styles.drawerSection}>
        <h3>
          <i className="fas fa-user-shield" style={{ color: "#2563eb" }}></i>{" "}
          Account Status
        </h3>
        <p style={{ fontSize: "0.8rem", color: "#64748b", marginBottom: "1rem" }}>
          Current:{" "}
          <strong style={{ color: st.color }}>
            {(selectedCustomer.status || "active").toUpperCase()}
          </strong>
          {selectedCustomer.status_reason && <span> — {selectedCustomer.status_reason}</span>}
        </p>

        <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: "0.75rem" }}>
          {selectedCustomer.status !== "active" && (
            <button
              onClick={() => onStatusUpdate("active")}
              disabled={statusUpdating}
              style={{
                padding: "0.5rem", border: "1px solid #d1fae5", borderRadius: "10px",
                background: "#f0fdf4", color: "#059669", fontWeight: 600, cursor: "pointer", fontSize: "0.8rem",
              }}
            >
              <i className="fas fa-check-circle" style={{ marginRight: "0.3rem" }}></i> Activate
            </button>
          )}
          {selectedCustomer.status !== "flagged" && (
            <button
              onClick={() => onShowStatusModal("flagged")}
              disabled={statusUpdating}
              style={{
                padding: "0.5rem", border: "1px solid #fef3c7", borderRadius: "10px",
                background: "#fffbeb", color: "#d97706", fontWeight: 600, cursor: "pointer", fontSize: "0.8rem",
              }}
            >
              <i className="fas fa-flag" style={{ marginRight: "0.3rem" }}></i> Flag
            </button>
          )}
          {selectedCustomer.status !== "suspended" && (
            <button
              onClick={() => onShowStatusModal("suspended")}
              disabled={statusUpdating}
              style={{
                padding: "0.5rem", border: "1px solid #fee2e2", borderRadius: "10px",
                background: "#fef2f2", color: "#dc2626", fontWeight: 600, cursor: "pointer", fontSize: "0.8rem",
              }}
            >
              <i className="fas fa-pause-circle" style={{ marginRight: "0.3rem" }}></i> Suspend
            </button>
          )}
          {selectedCustomer.status !== "blocked" && (
            <button
              onClick={() => onShowStatusModal("blocked")}
              disabled={statusUpdating}
              style={{
                padding: "0.5rem", border: "1px solid #f3f4f6", borderRadius: "10px",
                background: "#f9fafb", color: "#6b7280", fontWeight: 600, cursor: "pointer", fontSize: "0.8rem",
              }}
            >
              <i className="fas fa-ban" style={{ marginRight: "0.3rem" }}></i> Block
            </button>
          )}
        </div>
      </div>

      <div className={styles.drawerSection}>
        <h3>
          <i className="fas fa-shield-alt" style={{ color: "#7c3aed" }}></i> Withdrawal Permissions
        </h3>
        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", padding: "0.75rem", background: "#f8fafc", borderRadius: "12px", marginBottom: "1rem" }}>
          <div>
            <p style={{ margin: 0, fontWeight: 600, color: "#334155" }}>Allow Withdrawals</p>
            <p style={{ margin: "0.25rem 0 0", fontSize: "0.8rem", color: "#94a3b8" }}>Customer can withdraw funds to external wallets</p>
          </div>
          <button
            onClick={onToggleWithdraw}
            disabled={permUpdating}
            style={{
              width: "52px", height: "28px", borderRadius: "14px", border: "none", cursor: "pointer",
              background: selectedCustomer.can_withdraw ? "#059669" : "#d1d5db", position: "relative", transition: "background 0.2s",
            }}
          >
            <span style={{
              width: "22px", height: "22px", borderRadius: "50%", background: "white",
              position: "absolute", top: "3px", transition: "left 0.2s", boxShadow: "0 1px 3px rgba(0,0,0,0.2)",
              left: selectedCustomer.can_withdraw ? "27px" : "3px",
            }}></span>
          </button>
        </div>
        {selectedCustomer.withdrawal_limit && (
          <p style={{ fontSize: "0.85rem", color: "#64748b" }}>
            Per-transaction limit: <strong>{selectedCustomer.withdrawal_limit}</strong>
          </p>
        )}
      </div>
    </>
  );
};

export default PermissionsTab;
