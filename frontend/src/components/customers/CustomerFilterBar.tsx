import React from 'react';
import styles from "./styles/CustomerFilterBar.module.css";

interface FilterBarProps {
  searchTerm: string;
  setSearchTerm: (val: string) => void;
  statusFilter: string;
  setStatusFilter: (val: string) => void;
  onRefresh: () => void;
}

const CustomerFilterBar: React.FC<FilterBarProps> = ({
  searchTerm,
  setSearchTerm,
  statusFilter,
  setStatusFilter,
  onRefresh
}) => {
  return (
    <section className={styles.filterBar}>
      <div className={styles.searchWrapper}>
        <i className="fas fa-search"></i>
        <input
          className={styles.searchInput}
          placeholder="Search ID, name, or email..."
          value={searchTerm}
          onChange={(e) => setSearchTerm(e.target.value)}
        />
      </div>
      <div className={styles.filterActions}>
        <select
          className={styles.filterSelect}
          value={statusFilter}
          onChange={(e: any) => setStatusFilter(e.target.value)}
        >
          <option value="all">All Statuses</option>
          <option value="active">Active</option>
          <option value="flagged">Flagged</option>
          <option value="suspended">Suspended</option>
          <option value="blocked">Blocked</option>
        </select>
        <button
          className={styles.actionBtn}
          style={{
            background: "white",
            color: "#1e293b",
            border: "1px solid #e2e8f0",
          }}
          onClick={onRefresh}
        >
          <i className="fas fa-sync-alt"></i>
        </button>
      </div>
    </section>
  );
};

export default CustomerFilterBar;
