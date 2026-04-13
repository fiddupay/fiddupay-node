import React, { useState, useEffect, useMemo } from "react";
import { customerAPI, publicAPI } from "@/services/apiService";
import styles from "@/styles/pages/MerchantCustomersPage.module.css";
import { useToast } from "@/contexts/ToastContext";
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
  const { showToast } = useToast();
  const [customers, setCustomers] = useState<Customer[]>([]);
  const [loading, setLoading] = useState(true);
  const [searchTerm, setSearchTerm] = useState("");
  const [statusFilter, setStatusFilter] = useState<string>("all");
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
  const [detailsLoading, setDetailsLoading] = useState(false);
  const [customerWallets, setCustomerWallets] = useState<Wallet[]>([]);
  const [customerBalances, setCustomerBalances] = useState<any>(null);
  const [customerTransactions, setCustomerTransactions] = useState<CustomerTx[]>([]);
  const [sweepMode, setSweepMode] = useState<"ALL" | "NATIVE_ONLY" | "STABLE_ONLY" | "SPECIFIC">("ALL");
  const [sweepCryptoType, setSweepCryptoType] = useState("USDT");
  const [sweepAmount, setSweepAmount] = useState("");
  const [sweepPin, setSweepPin] = useState("");
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
  const [supportedCurrencies, setSupportedCurrencies] = useState<any[]>([]);
  const [customerSummary, setCustomerSummary] = useState<any>(null);

  useEffect(() => {
    fetchCustomers();
    fetchSupportedCurrencies();
    fetchCustomerSummary();
  }, []);

  const fetchCustomerSummary = async () => {
    try {
      const res = await customerAPI.getSummary();
      if (res.data) {
        setCustomerSummary(res.data);
      }
    } catch (err) {
      console.error("Failed to fetch customer summary", err);
    }
  };

  const fetchSupportedCurrencies = async () => {
    try {
      const res = await publicAPI.getSupportedCurrencies();
      if (res.data?.currency_groups) {
        const flattened = Object.values(res.data.currency_groups).flat() as any[];
        setSupportedCurrencies(flattened);
        if (flattened.length > 0) {
          if (!sweepCryptoType) setSweepCryptoType(flattened[0].crypto_type);
          if (!payMerchantCryptoType) setPayMerchantCryptoType(flattened[0].crypto_type);
        }
      }
    } catch (err) {
      console.error("Failed to fetch currencies", err);
    }
  };

  const fetchCustomers = async () => {
    try {
      setLoading(true);
      const res = await customerAPI.list();
      if (res.data?.customers) {
        setCustomers(res.data.customers);
      }
    } catch (error: any) {
      showToast(extractErrorMessage(error, "Failed to list customers"), "error");
    } finally {
      setLoading(false);
    }
  };

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

  const filteredCustomers = useMemo(() => {
    return customers.filter((c) => {
      const matchesSearch =
        c.external_id.toLowerCase().includes(searchTerm.toLowerCase()) ||
        c.email?.toLowerCase().includes(searchTerm.toLowerCase()) ||
        `${c.first_name || ""} ${c.last_name || ""}`.toLowerCase().includes(searchTerm.toLowerCase());
      const matchesStatus = statusFilter === "all" || c.status === statusFilter || (statusFilter === "inactive" && !c.is_active);
      return matchesSearch && matchesStatus;
    });
  }, [customers, searchTerm, statusFilter]);

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

  const fetchCustomerDetails = async (externalId: string) => {
    setDetailsLoading(true);
    try {
      const [walletRes, balRes, txRes] = await Promise.allSettled([
        customerAPI.getWallets(externalId),
        customerAPI.getBalances(externalId),
        customerAPI.getTransactions(externalId, { limit: 20 }),
      ]);
      if (walletRes.status === "fulfilled") setCustomerWallets(walletRes.value.data?.wallets || []);
      if (balRes.status === "fulfilled") setCustomerBalances(balRes.value.data?.balances);
      if (txRes.status === "fulfilled") setCustomerTransactions(txRes.value.data?.transactions || []);
    } catch { /* silent */ } finally {
      setDetailsLoading(false);
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
        setCustomers((prev) => prev.map((c) => (c.id === updated.id ? updated : c)));
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
        setCustomers((prev) => prev.map((c) => (c.id === updated.id ? updated : c)));
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
    if (!selectedCustomer || !sweepPin) {
      showToast("Please enter your Merchant Transaction PIN", "warning");
      return;
    }
    try {
      setSweeping(true);
      await customerAPI.sweep(selectedCustomer.external_id, {
        sweep_mode: sweepMode,
        crypto_types: sweepMode === "SPECIFIC" ? [sweepCryptoType] : undefined,
        amount: sweepAmount ? sweepAmount : undefined,
        pin: sweepPin,
      });
      showToast("Sweep operation initiated successfully", "success");
      setSweepAmount("");
      setSweepPin("");
      fetchCustomerDetails(selectedCustomer.external_id);
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

      <CustomerFilterBar
        searchTerm={searchTerm}
        setSearchTerm={setSearchTerm}
        statusFilter={statusFilter}
        setStatusFilter={setStatusFilter}
        onRefresh={fetchCustomers}
      />

      <CustomerDirectoryTable
        loading={loading}
        filteredCustomers={filteredCustomers}
        searchTerm={searchTerm}
        selectedCustomerIds={selectedCustomerIds}
        setSelectedCustomerIds={setSelectedCustomerIds}
        onCustomerClick={openCustomerDetails}
        onBulkProvision={handleBulkProvision}
        provisioning={provisioning}
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
        sweepPin={sweepPin}
        setSweepPin={setSweepPin}
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
