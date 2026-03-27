import React from 'react';
import styles from "../styles/CustomerDrawers.module.css";
import { CustomerTx, TX_BADGES } from "../types";

interface TransactionsTabProps {
  customerTransactions: CustomerTx[];
}

const TransactionsTab: React.FC<TransactionsTabProps> = ({ customerTransactions }) => {
  return (
    <div className={styles.drawerSection}>
      <h3>
        <i className="fas fa-history" style={{ color: "#2563eb" }}></i>{" "}
        Activity Log
      </h3>
      {customerTransactions.length > 0 ? (
        <div className={styles.transactionList}>
          {customerTransactions.map((tx) => {
            const badge = TX_BADGES[tx.type] || {
              color: "#64748b",
              bg: "#f1f5f9",
              icon: "fa-question-circle",
            };
            return (
              <div key={tx.id} className={styles.transactionItem}>
                <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "0.5rem" }}>
                  <span style={{
                    display: "inline-flex",
                    alignItems: "center",
                    gap: "0.3rem",
                    padding: "0.2rem 0.6rem",
                    borderRadius: "6px",
                    fontSize: "0.75rem",
                    fontWeight: 700,
                    color: badge.color,
                    background: badge.bg,
                  }}>
                    <i className={`fas ${badge.icon}`}></i>{" "}
                    {tx.type.replace("_", " ")}
                  </span>
                  <span style={{ fontSize: "0.75rem", color: "#94a3b8" }}>
                    {new Date(tx.created_at).toLocaleString()}
                  </span>
                </div>
                <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
                  <div style={{ display: "flex", flexDirection: "column", alignItems: "flex-start" }}>
                    <span style={{ fontWeight: 700, fontSize: "1rem", color: "#0f172a" }}>
                      {parseFloat(tx.amount).toFixed(6)} {tx.crypto_type}
                    </span>
                    <span style={{ fontSize: "0.75rem", color: "#94a3b8" }}>
                      ≈ ${parseFloat(tx.amount_usd || "0").toLocaleString(undefined, {
                        minimumFractionDigits: 2,
                        maximumFractionDigits: 2,
                      })} USD
                    </span>
                  </div>
                  <span style={{
                    padding: "0.15rem 0.5rem",
                    borderRadius: "6px",
                    fontSize: "0.7rem",
                    fontWeight: 600,
                    color: tx.status === "COMPLETED" ? "#059669" : "#d97706",
                    background: tx.status === "COMPLETED" ? "#d1fae5" : "#fef3c7",
                  }}>
                    {tx.status}
                  </span>
                </div>
                {tx.description && <p style={{ margin: "0.5rem 0 0", fontSize: "0.8rem", color: "#64748b" }}>{tx.description}</p>}
                {tx.transaction_hash && <p style={{ margin: "0.25rem 0 0", fontSize: "0.75rem", color: "#94a3b8", fontFamily: "monospace", wordBreak: "break-all" }}>TX: {tx.transaction_hash}</p>}
              </div>
            );
          })}
        </div>
      ) : (
        <div style={{ textAlign: "center", padding: "3rem 1rem", color: "#94a3b8" }}>
          <i className="fas fa-inbox" style={{ fontSize: "2rem", marginBottom: "0.75rem", display: "block" }}></i>
          <p style={{ margin: 0 }}>No transactions yet</p>
        </div>
      )}
    </div>
  );
};

export default TransactionsTab;
