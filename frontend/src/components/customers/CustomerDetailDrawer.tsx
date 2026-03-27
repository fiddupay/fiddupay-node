import React from 'react';
import styles from "./styles/CustomerDrawers.module.css";
import { Customer, Wallet, CustomerTx } from "./types";
import { getStatusStyle } from "./utils";
import OverviewTab from "./tabs/OverviewTab";
import TransactionsTab from "./tabs/TransactionsTab";
import PermissionsTab from "./tabs/PermissionsTab";
import ActionsTab from "./tabs/ActionsTab";

interface DetailDrawerProps {
  selectedCustomer: Customer | null;
  onClose: () => void;
  drawerTab: "overview" | "transactions" | "permissions" | "actions";
  setDrawerTab: (tab: "overview" | "transactions" | "permissions" | "actions") => void;
  detailsLoading: boolean;
  supportedCurrencies: any[];
  customerBalances: any;
  customerWallets: Wallet[];
  customerTransactions: CustomerTx[];
  expandedAsset: string | null;
  setExpandedAsset: (asset: string | null) => void;
  onProvisionWallets: () => void;
  provisioning: boolean;
  onCopy: (text: string) => void;
  statusUpdating: boolean;
  onStatusUpdate: (status: string) => void;
  onShowStatusModal: (status: string) => void;
  permUpdating: boolean;
  onToggleWithdraw: () => void;
  sweepMode: "ALL" | "NATIVE_ONLY" | "STABLE_ONLY" | "SPECIFIC";
  setSweepMode: (mode: "ALL" | "NATIVE_ONLY" | "STABLE_ONLY" | "SPECIFIC") => void;
  sweepCryptoType: string;
  setSweepCryptoType: (type: string) => void;
  sweepAmount: string;
  setSweepAmount: (amt: string) => void;
  sweepPin: string;
  setSweepPin: (pin: string) => void;
  sweeping: boolean;
  onSweep: (e: React.FormEvent) => void;
  payMerchantAmount: string;
  setPayMerchantAmount: (amt: string) => void;
  payMerchantCryptoType: string;
  setPayMerchantCryptoType: (type: string) => void;
  payingMerchant: boolean;
  onPayMerchant: (e: React.FormEvent) => void;
}

const CustomerDetailDrawer: React.FC<DetailDrawerProps> = ({
  selectedCustomer,
  onClose,
  drawerTab,
  setDrawerTab,
  detailsLoading,
  supportedCurrencies,
  customerBalances,
  customerWallets,
  customerTransactions,
  expandedAsset,
  setExpandedAsset,
  onProvisionWallets,
  provisioning,
  onCopy,
  statusUpdating,
  onStatusUpdate,
  onShowStatusModal,
  permUpdating,
  onToggleWithdraw,
  sweepMode,
  setSweepMode,
  sweepCryptoType,
  setSweepCryptoType,
  sweepAmount,
  setSweepAmount,
  sweepPin,
  setSweepPin,
  sweeping,
  onSweep,
  payMerchantAmount,
  setPayMerchantAmount,
  payMerchantCryptoType,
  setPayMerchantCryptoType,
  payingMerchant,
  onPayMerchant,
}) => {
  if (!selectedCustomer) return null;

  const st = getStatusStyle(selectedCustomer.status || "active");

  return (
    <div className={styles.overlay} onClick={onClose}>
      <div className={styles.drawer} onClick={(e) => e.stopPropagation()}>
        <div className={styles.drawerHeader}>
          <div style={{ display: "flex", alignItems: "center", gap: "0.75rem" }}>
            <h2>
              <i className="fas fa-id-badge" style={{ color: "#2563eb" }}></i>{" "}
              {selectedCustomer.first_name || selectedCustomer.last_name
                ? `${selectedCustomer.first_name || ""} ${selectedCustomer.last_name || ""}`.trim()
                : selectedCustomer.external_id}
            </h2>
            <span style={{
              display: "inline-flex", alignItems: "center", gap: "0.3rem", padding: "0.2rem 0.6rem", borderRadius: "999px",
              fontSize: "0.75rem", fontWeight: 700, color: st.color, background: st.bg,
            }}>
              <i className={`fas ${st.icon}`} style={{ fontSize: "0.65rem" }}></i> {st.label}
            </span>
          </div>
          <button className={styles.closeBtn} onClick={onClose}><i className="fas fa-times"></i></button>
        </div>

        <div className={styles.drawerTabs}>
          {(["overview", "transactions", "permissions", "actions"] as const).map((tab) => (
            <button
              key={tab}
              className={`${styles.tabBtn} ${drawerTab === tab ? styles.active : ""}`}
              onClick={() => setDrawerTab(tab)}
            >
              {tab === "overview" && <i className="fas fa-wallet" style={{ marginRight: "0.4rem" }}></i>}
              {tab === "transactions" && <i className="fas fa-exchange-alt" style={{ marginRight: "0.4rem" }}></i>}
              {tab === "permissions" && <i className="fas fa-shield-alt" style={{ marginRight: "0.4rem" }}></i>}
              {tab === "actions" && <i className="fas fa-hand-holding-usd" style={{ marginRight: "0.4rem" }}></i>}
              {tab === "actions" ? "Financial Actions" : tab.charAt(0).toUpperCase() + tab.slice(1)}
            </button>
          ))}
        </div>

        <div className={styles.drawerBody}>
          {detailsLoading ? (
            <div className={styles.loadingOverlay}><i className="fas fa-circle-notch fa-spin fa-2x"></i></div>
          ) : (
            <>
              {drawerTab === "overview" && (
                <OverviewTab
                  supportedCurrencies={supportedCurrencies}
                  customerBalances={customerBalances}
                  customerWallets={customerWallets}
                  expandedAsset={expandedAsset}
                  setExpandedAsset={setExpandedAsset}
                  onProvisionWallets={onProvisionWallets}
                  provisioning={provisioning}
                  onCopy={onCopy}
                />
              )}
              {drawerTab === "transactions" && <TransactionsTab customerTransactions={customerTransactions} />}
              {drawerTab === "permissions" && (
                <PermissionsTab
                  selectedCustomer={selectedCustomer}
                  statusUpdating={statusUpdating}
                  onStatusUpdate={onStatusUpdate}
                  onShowStatusModal={onShowStatusModal}
                  permUpdating={permUpdating}
                  onToggleWithdraw={onToggleWithdraw}
                />
              )}
              {drawerTab === "actions" && (
                <ActionsTab
                  sweepMode={sweepMode}
                  setSweepMode={setSweepMode}
                  sweepCryptoType={sweepCryptoType}
                  setSweepCryptoType={setSweepCryptoType}
                  sweepAmount={sweepAmount}
                  setSweepAmount={setSweepAmount}
                  sweepPin={sweepPin}
                  setSweepPin={setSweepPin}
                  sweeping={sweeping}
                  onSweep={onSweep}
                  payMerchantAmount={payMerchantAmount}
                  setPayMerchantAmount={setPayMerchantAmount}
                  payMerchantCryptoType={payMerchantCryptoType}
                  setPayMerchantCryptoType={setPayMerchantCryptoType}
                  payingMerchant={payingMerchant}
                  onPayMerchant={onPayMerchant}
                  supportedCurrencies={supportedCurrencies}
                  customerBalances={customerBalances}
                />
              )}
            </>
          )}
        </div>
      </div>
    </div>
  );
};

export default CustomerDetailDrawer;
