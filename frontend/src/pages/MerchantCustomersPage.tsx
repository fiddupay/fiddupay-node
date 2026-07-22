import React, { useState, useEffect, useMemo, useCallback } from "react";
import { useSearchParams } from "react-router-dom";
import { customerAPI } from "@/services/apiService";
import { useDataStore } from "@/stores/dataStore";
import { useBalanceStore } from "@/stores/balanceStore";
import styles from "@/styles/pages/MerchantCustomersPage.module.css";
import { useToast } from "@/contexts/ToastContext";
import { useAuthStore } from "@/stores/authStore";
import { extractErrorMessage } from "@/utils/errorUtils";

// Modular Components
import CustomerStatsCards from "@/components/customers/CustomerStatsCards";
import CustomerFilterBar from "@/components/customers/CustomerFilterBar";
import CustomerDirectoryTable from "@/components/customers/CustomerDirectoryTable";
import CreateCustomerDrawer from "@/components/customers/CreateCustomerDrawer";
import CustomerDetailDrawer from "@/components/customers/CustomerDetailDrawer";
import StatusUpdateModal from "@/components/customers/StatusUpdateModal";

// Types & Utils
import { Customer, Wallet, CustomerTx } from "@/components/customers/types";

const MerchantCustomersPage: React.FC = () => {
  const { user } = useAuthStore();
  const { showToast } = useToast();

  // --- URL-driven state (page, search, status) ---
  const [searchParams, setSearchParams] = useSearchParams();

  const searchTerm = searchParams.get("search") ?? "";
  const statusFilter = searchParams.get("status") ?? "all";
  const page = Math.max(1, parseInt(searchParams.get("page") ?? "1", 10));

  const setSearchTerm = useCallback((val: string) => {
    setSearchParams((prev) => {
      const next = new URLSearchParams(prev);
      if (val) next.set("search", val); else next.delete("search");
      next.set("page", "1"); // reset to page 1 on new search
      return next;
    }, { replace: true });
  }, [setSearchParams]);

  const setStatusFilter = useCallback((val: string) => {
    setSearchParams((prev) => {
      const next = new URLSearchParams(prev);
      if (val && val !== "all") next.set("status", val); else next.delete("status");
      next.set("page", "1"); // reset to page 1 on filter change
      return next;
    }, { replace: true });
  }, [setSearchParams]);

  const setPage = useCallback((p: number) => {
    setSearchParams((prev) => {
      const next = new URLSearchParams(prev);
      next.set("page", String(p));
      return next;
    }, { replace: false }); // push so Back button navigates pages
  }, [setSearchParams]);

  const [selectedCustomerIds, setSelectedCustomerIds] = useState<string[]>([]);

  // Drawer States
  const [isCreateDrawerOpen, setIsCreateDrawerOpen] = useState(false);
  const [selectedCustomer, setSelectedCustomer] = useState<Customer | null>(null);
  const [drawerTab, setDrawerTab] = useState<"overview" | "transactions" | "permissions" | "actions">("overview");
  const [expandedAsset, setExpandedAsset] = useState<string | null>(null);

  // Form States
  const [newCustomer, setNewCustomer] = useState({
    external_id: "",
    email: "",
    first_name: "",
    last_name: "",
  });
  const [submitting, setSubmitting] = useState(false);
  const [customerWallets, setCustomerWallets] = useState<Wallet[]>([]);
  const [customerBalances, setCustomerBalances] = useState<any>(null);
  const [customerTransactions, setCustomerTransactions] = useState<CustomerTx[]>([]);
  const [sweepMode, setSweepMode] = useState<"ALL" | "NATIVE_ONLY" | "STABLE_ONLY" | "SPECIFIC">("ALL");
  const [sweepCryptoType, setSweepCryptoType] = useState("USDT");
  const [sweepAmount, setSweepAmount] = useState("");
  const [sweeping, setSweeping] = useState(false);
  const [provisioning, setProvisioning] = useState(false);

  // Financial Actions State
  const [payMerchantAmount, setPayMerchantAmount] = useState("");
  const [payMerchantCryptoType, setPayMerchantCryptoType] = useState("USDT");
  const [payingMerchant, setPayingMerchant] = useState(false);

  // Status update states
  const [statusUpdating, setStatusUpdating] = useState(false);
  const [statusReason, setStatusReason] = useState("");
  const [showStatusModal, setShowStatusModal] = useState<string | null>(null);

  // Permission states
  const [permUpdating, setPermUpdating] = useState(false);
  const [customerSummary, setCustomerSummary] = useState<any>(null);

  // Wallet Health Panel State
  const [lookupAddress, setLookupAddress] = useState("");
  const [lookupResult, setLookupResult] = useState<any>(null);
  const [lookupLoading, setLookupLoading] = useState(false);
  const [repairLoading, setRepairLoading] = useState(false);
  const [repairResult, setRepairResult] = useState<any>(null);
  const [showWalletHealth, setShowWalletHealth] = useState(false);

  // Use global dataStore for currencies, customers, customer summary
  const {
    currencies: currenciesCache,
    customers: customersCache,
    customersTotal,
    customerDetails: detailsMap,
    fetchCurrencies,
    fetchCustomers: fetchCustomersFromStore,
    fetchCustomerSummary: fetchSummaryFromStore,
    setCustomers: setCustomersInStore,
  } = useDataStore();
  const supportedCurrencies = currenciesCache.data || [];
  const customers = customersCache.data || [];

  // Determine global loading state
  const loading = customersCache.loading && customers.length === 0;
  
  // Use store's detail loading state for the specific customer
  const detailsLoading = selectedCustomer 
    ? (detailsMap[selectedCustomer.external_id]?.loading || false) 
    : false;

  // Fetch when page, search, status, or sandbox mode changes
  useEffect(() => {
    fetchCustomersFromStore(page, 10, searchTerm || undefined, statusFilter !== 'all' ? statusFilter : undefined);
    fetchCurrencies();
    fetchSummaryFromStore().then((data: any) => {
      if (data) setCustomerSummary(data);
    });
  }, [page, searchTerm, statusFilter, user?.sandbox_mode]);


  const fetchCustomers = async () => {
    try {
      await fetchCustomersFromStore(page, 10, searchTerm || undefined, statusFilter !== 'all' ? statusFilter : undefined, true);
    } catch (error: any) {
      showToast(extractErrorMessage(error, "Failed to list customers"), "error");
    }
  };

  // Stats are computed from customerSummary (server-provided) or from the current customers list
  const stats = useMemo(() => {
    if (customerSummary) {
      return {
        total: customerSummary.total_customers,
        active: customerSummary.active_customers,
        flagged: customerSummary.flagged_customers,
        recent: customerSummary.recent_customers,
        totalBalanceUsd: customerSummary.total_balance_usd,
      };
    }

    const total = customers.length;
    const active = customers.filter((c) => c.status === "active" && c.is_active).length;
    const flagged = customers.filter((c) => c.status === "flagged").length;
    const recent = customers.filter((c) => {
      const diff = Date.now() - new Date(c.created_at).getTime();
      return diff < 7 * 24 * 60 * 60 * 1000;
    }).length;
    return { total, active, flagged, recent, totalBalanceUsd: 0 };
  }, [customers, customerSummary]);

  // No client-side filtering — search/status filtering is done server-side via the API.
  // customers from the store is already the correct page of filtered results.

  const handleCreateCustomer = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!newCustomer.external_id) {
      showToast("External ID is required", "error");
      return;
    }
    try {
      setSubmitting(true);
      await customerAPI.create(newCustomer);
      showToast("Customer registered with wallets provisioned", "success");
      setIsCreateDrawerOpen(false);
      setNewCustomer({ external_id: "", email: "", first_name: "", last_name: "" });
      fetchCustomers();
    } catch (error: any) {
      showToast(extractErrorMessage(error, "Failed to register customer"), "error");
    } finally {
      setSubmitting(false);
    }
  };

  const fetchCustomerDetails = async (externalId: string, force: boolean = false) => {
    try {
      // Use global store for caching customer details
      const data = await useDataStore.getState().fetchCustomerDetails(externalId, force);
      if (data) {
        setCustomerWallets(data.wallets);
        setCustomerBalances(data.balances);
        setCustomerTransactions(data.transactions);
      }
    } catch (error) {
      console.error("Failed to fetch customer details", error);
      showToast("Failed to load customer details", "error");
    }
  };

  const openCustomerDetails = async (customer: Customer) => {
    setSelectedCustomer(customer);
    setDrawerTab("overview");
    setCustomerWallets([]);
    setCustomerBalances(null);
    setCustomerTransactions([]);
    fetchCustomerDetails(customer.external_id);
  };

  const handleStatusUpdate = async (newStatus: string) => {
    if (!selectedCustomer) return;
    try {
      setStatusUpdating(true);
      const res = await customerAPI.updateStatus(selectedCustomer.external_id, {
        status: newStatus,
        reason: statusReason || undefined,
      });
      const updated = res.data?.customer;
      if (updated) {
        setSelectedCustomer(updated);
        setCustomersInStore(customers.map((c) => (c.id === updated.id ? updated : c)));
      }
      showToast(`Customer status changed to ${newStatus}`, "success");
      setShowStatusModal(null);
      setStatusReason("");
    } catch (error: any) {
      showToast(extractErrorMessage(error, "Failed to update status"), "error");
    } finally {
      setStatusUpdating(false);
    }
  };

  const handleToggleWithdraw = async () => {
    if (!selectedCustomer) return;
    try {
      setPermUpdating(true);
      const res = await customerAPI.updatePermissions(selectedCustomer.external_id, {
        can_withdraw: !selectedCustomer.can_withdraw
      });
      const updated = res.data?.customer;
      if (updated) {
        setSelectedCustomer(updated);
        setCustomersInStore(customers.map((c) => (c.id === updated.id ? updated : c)));
      }
      showToast(`Withdrawals ${!selectedCustomer.can_withdraw ? "enabled" : "disabled"}`, "success");
    } catch (error: any) {
      showToast(extractErrorMessage(error, "Failed to update permissions"), "error");
    } finally {
      setPermUpdating(false);
    }
  };

  const handleSweep = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!selectedCustomer) {
      return;
    }
    try {
      setSweeping(true);
      await customerAPI.sweep(selectedCustomer.external_id, {
        sweep_mode: sweepMode,
        crypto_types: sweepMode === "SPECIFIC" ? [sweepCryptoType] : undefined,
        amount: sweepAmount ? sweepAmount : undefined,
      });
      showToast("Settlement operation initiated successfully", "success");
      setSweepAmount("");
      // Force refresh customer details, customer lists, and merchant balance immediately
      fetchCustomerDetails(selectedCustomer.external_id, true);
      fetchCustomersFromStore(page, 10, searchTerm || undefined, statusFilter !== 'all' ? statusFilter : undefined, true);
      fetchSummaryFromStore(true);
      useBalanceStore.getState().fetchBalance(true);
      const dataStore = useDataStore.getState();
      dataStore.invalidate('analytics');
      dataStore.invalidate('balanceHistory');
      dataStore.invalidate('recentActivity');
    } catch (error: any) {
      showToast(extractErrorMessage(error, "Failed to sweep funds"), "error");
    } finally {
      setSweeping(false);
    }
  };

  const handleProvisionWallets = async () => {
    if (!selectedCustomer) return;
    try {
      setProvisioning(true);
      await customerAPI.createWallets(selectedCustomer.external_id, {
        networks: ["EVM", "SOLANA", "BITCOIN"],
      });
      showToast("Wallets provisioned successfully", "success");
      fetchCustomerDetails(selectedCustomer.external_id);
    } catch (error: any) {
      showToast(extractErrorMessage(error, "Failed to provision wallets"), "error");
    } finally {
      setProvisioning(false);
    }
  };

  const handlePayMerchant = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!selectedCustomer || !payMerchantAmount) {
      showToast("Please fill all required fields", "warning");
      return;
    }
    try {
      setPayingMerchant(true);
      await customerAPI.payMerchant(selectedCustomer.external_id, {
        crypto_type: payMerchantCryptoType,
        amount: payMerchantAmount,
      });
      showToast("Payment to merchant successful", "success");
      setPayMerchantAmount("");
      fetchCustomerDetails(selectedCustomer.external_id);
    } catch (error: any) {
      showToast(extractErrorMessage(error, "Failed to process payment"), "error");
    } finally {
      setPayingMerchant(false);
    }
  };

  const handleBulkProvision = async (isAll: boolean = false) => {
    if (!isAll && selectedCustomerIds.length === 0) return;
    try {
      setProvisioning(true);
      await customerAPI.bulkProvisionWallets({
        customer_ids: isAll ? undefined : selectedCustomerIds,
        all_customers: isAll,
      });
      showToast(`Wallets regenerated successfully for ${isAll ? "all" : selectedCustomerIds.length} customers`, "success");
      if (!isAll) setSelectedCustomerIds([]);
    } catch (error: any) {
      showToast(extractErrorMessage(error, "Failed to bulk regenerate wallets"), "error");
    } finally {
      setProvisioning(false);
    }
  };

  const handleCopy = (text: string) => {
    navigator.clipboard.writeText(text);
    showToast("Copied to clipboard", "success");
  };

  const handleLookupAddress = async () => {
    if (!lookupAddress.trim()) return;
    setLookupLoading(true);
    setLookupResult(null);
    try {
      const res = await customerAPI.lookupAddress(lookupAddress.trim());
      setLookupResult(res.data);
    } catch (err: any) {
      if (err?.response?.status === 404) {
        setLookupResult({ found: false, message: "Address not found for any of your customers" });
      } else {
        showToast(extractErrorMessage(err, "Lookup failed"), "error");
      }
    } finally {
      setLookupLoading(false);
    }
  };

  const handleVerifyAndRepair = async () => {
    setRepairLoading(true);
    setRepairResult(null);
    try {
      const res = await customerAPI.verifyAndRepairWallets();
      setRepairResult(res.data);
      showToast(`Repair complete — ${res.data.repaired_wallets} wallets provisioned across ${res.data.checked_customers} customers`, "success");
    } catch (err: any) {
      showToast(extractErrorMessage(err, "Verify & repair failed"), "error");
    } finally {
      setRepairLoading(false);
    }
  };

  return (
    <div className={styles.page}>
      <header className={styles.header}>
        <div className={styles.headerInfo}>
          <h1>Customer Directory</h1>
          <p>Manage your ecosystem of sub-accounts and dedicated wallets</p>
        </div>
        <div className={styles.headerActions}>
          <button className={styles.addBtn} onClick={() => setIsCreateDrawerOpen(true)}>
            <i className="fas fa-user-plus"></i> Register Customer
          </button>
        </div>
      </header>

      <CustomerStatsCards stats={stats} />

      {/* Wallet Health Panel */}
      <div style={{ margin: "0 0 1.5rem 0", background: "var(--surface)", border: "1px solid var(--border)", borderRadius: "12px", overflow: "hidden" }}>
        <button
          onClick={() => setShowWalletHealth(v => !v)}
          style={{ width: "100%", background: "none", border: "none", padding: "0.9rem 1.5rem", display: "flex", alignItems: "center", gap: "0.75rem", cursor: "pointer", color: "var(--text-main)" }}
        >
          <i className="fas fa-shield-alt" style={{ color: "#f59e0b" }}></i>
          <span style={{ fontWeight: 700, fontSize: "0.95rem" }}>Wallet Health Tools</span>
          <span style={{ fontSize: "0.8rem", color: "var(--text-muted)", marginLeft: "auto" }}>{showWalletHealth ? "Hide" : "Show"}</span>
        </button>
        {showWalletHealth && (
          <div style={{ padding: "0 1.5rem 1.5rem", display: "flex", flexDirection: "column", gap: "1.25rem", borderTop: "1px solid var(--border)" }}>
            {/* Address Lookup */}
            <div style={{ paddingTop: "1.25rem" }}>
              <p style={{ fontWeight: 600, marginBottom: "0.5rem", fontSize: "0.875rem" }}>
                <i className="fas fa-search" style={{ marginRight: "0.5rem", color: "#3b82f6" }}></i>Address Lookup
              </p>
              <p style={{ fontSize: "0.8rem", color: "var(--text-muted)", marginBottom: "0.75rem" }}>Check if a wallet address is linked to any of your customers (active or historical).</p>
              <div style={{ display: "flex", gap: "0.75rem" }}>
                <input
                  id="wallet-lookup-input"
                  type="text"
                  value={lookupAddress}
                  onChange={e => setLookupAddress(e.target.value)}
                  onKeyDown={e => e.key === "Enter" && handleLookupAddress()}
                  placeholder="Paste a wallet address..."
                  style={{ flex: 1, padding: "0.6rem 1rem", borderRadius: "8px", border: "1px solid var(--border)", background: "var(--bg-main)", color: "var(--text-main)", fontSize: "0.875rem", fontFamily: "monospace" }}
                />
                <button
                  id="wallet-lookup-btn"
                  onClick={handleLookupAddress}
                  disabled={lookupLoading}
                  style={{ padding: "0.6rem 1.25rem", borderRadius: "8px", border: "none", background: "#3b82f6", color: "white", fontWeight: 600, cursor: "pointer", opacity: lookupLoading ? 0.6 : 1 }}
                >
                  {lookupLoading ? <i className="fas fa-spinner fa-spin"></i> : "Lookup"}
                </button>
              </div>
              {lookupResult && (
                <div style={{ marginTop: "0.75rem", padding: "0.9rem 1rem", borderRadius: "8px", border: `1px solid ${lookupResult.found ? (lookupResult.status === "ACTIVE" ? "#059669" : "#f59e0b") : "#dc2626"}30`, background: `${lookupResult.found ? (lookupResult.status === "ACTIVE" ? "#05966910" : "#f59e0b10") : "#dc262610"}` }}>
                  {lookupResult.found ? (
                    <>
                      <div style={{ display: "flex", alignItems: "center", gap: "0.5rem", marginBottom: "0.5rem" }}>
                        <span style={{ fontWeight: 700, fontSize: "0.875rem", color: lookupResult.status === "ACTIVE" ? "#059669" : "#f59e0b" }}>
                          <i className={`fas fa-${lookupResult.status === "ACTIVE" ? "check-circle" : "history"}`} style={{ marginRight: "0.4rem" }}></i>
                          {lookupResult.status}
                        </span>
                      </div>
                      <p style={{ fontSize: "0.825rem", margin: "0.25rem 0" }}><b>Customer:</b> {lookupResult.customer?.email} ({lookupResult.customer?.external_id})</p>
                      <p style={{ fontSize: "0.825rem", margin: "0.25rem 0" }}><b>Network:</b> {lookupResult.wallet?.network} / {lookupResult.wallet?.crypto_type}</p>
                      {lookupResult.status === "HISTORICAL" && <p style={{ fontSize: "0.8rem", color: "#f59e0b", marginTop: "0.4rem" }}>⚠ This is an old address. The customer has a new active address.</p>}
                    </>
                  ) : (
                    <p style={{ fontSize: "0.875rem", color: "#dc2626" }}><i className="fas fa-times-circle" style={{ marginRight: "0.4rem" }}></i>Not found for any of your customers.</p>
                  )}
                </div>
              )}
            </div>

            {/* Verify & Repair */}
            <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", padding: "0.9rem 1rem", background: "var(--bg-main)", borderRadius: "10px", border: "1px solid var(--border)" }}>
              <div>
                <p style={{ fontWeight: 600, fontSize: "0.875rem", margin: 0 }}>
                  <i className="fas fa-wrench" style={{ marginRight: "0.5rem", color: "#8b5cf6" }}></i>Verify & Repair Wallets
                </p>
                <p style={{ fontSize: "0.8rem", color: "var(--text-muted)", margin: "0.25rem 0 0" }}>Auto-provision any missing wallets for all customers across all active networks.</p>
                {repairResult && <p style={{ fontSize: "0.8rem", color: "#059669", marginTop: "0.4rem" }}>✓ Checked {repairResult.checked_customers} customers — {repairResult.repaired_wallets} wallets provisioned.</p>}
              </div>
              <button
                id="verify-repair-btn"
                onClick={handleVerifyAndRepair}
                disabled={repairLoading}
                style={{ padding: "0.6rem 1.25rem", borderRadius: "8px", border: "none", background: "#8b5cf6", color: "white", fontWeight: 600, cursor: "pointer", whiteSpace: "nowrap", opacity: repairLoading ? 0.6 : 1 }}
              >
                {repairLoading ? <><i className="fas fa-spinner fa-spin" style={{ marginRight: "0.4rem" }}></i>Running...</> : "Run Repair"}
              </button>
            </div>
          </div>
        )}
      </div>

      <CustomerFilterBar
        searchTerm={searchTerm}
        setSearchTerm={setSearchTerm}
        statusFilter={statusFilter}
        setStatusFilter={setStatusFilter}
        onRefresh={fetchCustomers}
      />

      <CustomerDirectoryTable
        loading={loading}
        customers={customers}
        total={customersTotal}
        pageSize={10}
        searchTerm={searchTerm}
        selectedCustomerIds={selectedCustomerIds}
        setSelectedCustomerIds={setSelectedCustomerIds}
        onCustomerClick={openCustomerDetails}
        onBulkProvision={handleBulkProvision}
        provisioning={provisioning}
        page={page}
        onPageChange={setPage}
      />

      <CreateCustomerDrawer
        isOpen={isCreateDrawerOpen}
        onClose={() => setIsCreateDrawerOpen(false)}
        newCustomer={newCustomer}
        setNewCustomer={setNewCustomer}
        submitting={submitting}
        onSubmit={handleCreateCustomer}
      />

      <CustomerDetailDrawer
        selectedCustomer={selectedCustomer}
        onClose={() => setSelectedCustomer(null)}
        drawerTab={drawerTab}
        setDrawerTab={setDrawerTab}
        detailsLoading={detailsLoading}
        supportedCurrencies={supportedCurrencies}
        customerBalances={customerBalances}
        customerWallets={customerWallets}
        customerTransactions={customerTransactions}
        expandedAsset={expandedAsset}
        setExpandedAsset={setExpandedAsset}
        onProvisionWallets={handleProvisionWallets}
        provisioning={provisioning}
        onCopy={handleCopy}
        statusUpdating={statusUpdating}
        onStatusUpdate={handleStatusUpdate}
        onShowStatusModal={setShowStatusModal}
        permUpdating={permUpdating}
        onToggleWithdraw={handleToggleWithdraw}
        sweepMode={sweepMode}
        setSweepMode={setSweepMode}
        sweepCryptoType={sweepCryptoType}
        setSweepCryptoType={setSweepCryptoType}
        sweepAmount={sweepAmount}
        setSweepAmount={setSweepAmount}
        sweeping={sweeping}
        onSweep={handleSweep}
        payMerchantAmount={payMerchantAmount}
        setPayMerchantAmount={setPayMerchantAmount}
        payMerchantCryptoType={payMerchantCryptoType}
        setPayMerchantCryptoType={setPayMerchantCryptoType}
        payingMerchant={payingMerchant}
        onPayMerchant={handlePayMerchant}
      />

      <StatusUpdateModal
        showStatusModal={showStatusModal}
        setShowStatusModal={setShowStatusModal}
        statusReason={statusReason}
        setStatusReason={setStatusReason}
        onConfirm={() => handleStatusUpdate(showStatusModal!)}
      />
    </div>
  );
};

export default MerchantCustomersPage;
