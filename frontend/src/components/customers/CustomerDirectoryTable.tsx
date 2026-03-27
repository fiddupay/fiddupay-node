import React from 'react';
import styles from "./styles/CustomerDirectoryTable.module.css";
import { Customer } from "./types";
import { getStatusStyle, getInitials } from "./utils";

interface TableProps {
  loading: boolean;
  filteredCustomers: Customer[];
  searchTerm: string;
  selectedCustomerIds: string[];
  setSelectedCustomerIds: (ids: string[]) => void;
  onCustomerClick: (customer: Customer) => void;
  onBulkProvision: (all: boolean) => void;
  provisioning: boolean;
}

const CustomerDirectoryTable: React.FC<TableProps> = ({
  loading,
  filteredCustomers,
  searchTerm,
  selectedCustomerIds,
  setSelectedCustomerIds,
  onCustomerClick,
  onBulkProvision,
  provisioning
}) => {
  return (
    <div className={styles.contentCard}>
      <div className={styles.tableHeader}>
        <div style={{ display: "flex", alignItems: "center", gap: "1rem" }}>
          <h2>Registered Entities</h2>
          <span style={{ fontSize: "0.875rem", color: "#64748b", fontWeight: 600 }}>
            {filteredCustomers.length} results found
          </span>
        </div>
        {(selectedCustomerIds.length > 0 || filteredCustomers.length > 0) && (
          <div style={{ display: "flex", gap: "0.5rem" }}>
            {selectedCustomerIds.length > 0 && (
              <button
                className={styles.actionBtn}
                style={{ background: "#3b82f6", color: "white", padding: "0.5rem 1rem" }}
                onClick={() => onBulkProvision(false)}
                disabled={provisioning}
              >
                <i className={provisioning ? "fas fa-spinner fa-spin mr-2" : "fas fa-sync-alt mr-2"}></i>
                Regenerate {selectedCustomerIds.length} Selected
              </button>
            )}
            <button
              className={styles.actionBtn}
              style={{ background: "#f59e0b", color: "white", padding: "0.5rem 1rem" }}
              onClick={() => onBulkProvision(true)}
              disabled={provisioning}
            >
              <i className={provisioning ? "fas fa-spinner fa-spin mr-2" : "fas fa-magic mr-2"}></i>
              Regenerate All
            </button>
          </div>
        )}
      </div>

      {loading ? (
        <div className={styles.loadingOverlay}>
          <i className="fas fa-circle-notch fa-spin fa-3x"></i>
        </div>
      ) : filteredCustomers.length === 0 ? (
        <div className={styles.noData}>
          <i className="fas fa-users-slash"></i>
          <p>{searchTerm ? "No results match your search" : "No customers registered yet"}</p>
        </div>
      ) : (
        <div className={styles.tableContainer}>
          <table className={styles.table}>
            <thead>
              <tr>
                <th style={{ width: "40px" }}>
                  <input
                    type="checkbox"
                    checked={filteredCustomers.length > 0 && selectedCustomerIds.length === filteredCustomers.length}
                    onChange={(e) => {
                      if (e.target.checked) {
                        setSelectedCustomerIds(filteredCustomers.map((c) => c.external_id));
                      } else {
                        setSelectedCustomerIds([]);
                      }
                    }}
                  />
                </th>
                <th>Customer Identity</th>
                <th>External ID</th>
                <th>Status</th>
                <th>Withdrawals</th>
                <th>Joined Date</th>
                <th style={{ textAlign: "right" }}>Actions</th>
              </tr>
            </thead>
            <tbody>
              {filteredCustomers.map((c) => {
                const st = getStatusStyle(c.status || "active");
                return (
                  <tr key={c.id} className={styles.customerRow} onClick={() => onCustomerClick(c)}>
                    <td onClick={(e) => e.stopPropagation()}>
                      <input
                        type="checkbox"
                        checked={selectedCustomerIds.includes(c.external_id)}
                        onChange={(e) => {
                          if (e.target.checked) {
                            setSelectedCustomerIds([...selectedCustomerIds, c.external_id]);
                          } else {
                            setSelectedCustomerIds(selectedCustomerIds.filter((id) => id !== c.external_id));
                          }
                        }}
                      />
                    </td>
                    <td>
                      <div className={styles.customerInfo}>
                        <div className={styles.avatar}>{getInitials(c)}</div>
                        <div className={styles.customerMeta}>
                          <span className={styles.customerName}>
                            {c.first_name || c.last_name
                              ? `${c.first_name || ""} ${c.last_name || ""}`.trim()
                              : "Unnamed Customer"}
                          </span>
                          <span className={styles.customerEmail}>{c.email || "No email provided"}</span>
                        </div>
                      </div>
                    </td>
                    <td><span className={styles.externalId}>{c.external_id}</span></td>
                    <td>
                      <span style={{
                        display: "inline-flex",
                        alignItems: "center",
                        gap: "0.35rem",
                        padding: "0.25rem 0.75rem",
                        borderRadius: "999px",
                        fontSize: "0.8rem",
                        fontWeight: 600,
                        color: st.color,
                        background: st.bg,
                      }}>
                        <i className={`fas ${st.icon}`} style={{ fontSize: "0.7rem" }}></i>{" "}
                        {st.label}
                      </span>
                    </td>
                    <td>
                      <span style={{ color: c.can_withdraw ? "#059669" : "#dc2626", fontWeight: 600, fontSize: "0.85rem" }}>
                        {c.can_withdraw ? "✓ Enabled" : "✗ Disabled"}
                      </span>
                    </td>
                    <td>
                      {new Date(c.created_at).toLocaleDateString(undefined, {
                        month: "short",
                        day: "numeric",
                        year: "numeric",
                      })}
                    </td>
                    <td style={{ textAlign: "right" }}>
                      <button className={styles.actionBtn} style={{ padding: "0.5rem 1rem", background: "#f1f5f9", color: "#1e293b", display: "inline-flex" }}>
                        Manage <i className="fas fa-chevron-right ml-2"></i>
                      </button>
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
};

export default CustomerDirectoryTable;
