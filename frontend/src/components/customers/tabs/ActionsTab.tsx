import React from 'react';
import styles from "../styles/CustomerDrawers.module.css";

interface ActionsTabProps {
  sweepMode: "ALL" | "NATIVE_ONLY" | "STABLE_ONLY" | "SPECIFIC";
  setSweepMode: (val: "ALL" | "NATIVE_ONLY" | "STABLE_ONLY" | "SPECIFIC") => void;
  sweepCryptoType: string;
  setSweepCryptoType: (val: string) => void;
  sweepAmount: string;
  setSweepAmount: (val: string) => void;
  sweepPin: string;
  setSweepPin: (val: string) => void;
  sweeping: boolean;
  onSweep: (e: React.FormEvent) => void;
  payMerchantAmount: string;
  setPayMerchantAmount: (val: string) => void;
  payMerchantCryptoType: string;
  setPayMerchantCryptoType: (val: string) => void;
  payingMerchant: boolean;
  onPayMerchant: (e: React.FormEvent) => void;
  supportedCurrencies: any[];
  customerBalances: any;
}

const ActionsTab: React.FC<ActionsTabProps> = ({
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
  supportedCurrencies,
  customerBalances
}) => {
  return (
    <div style={{ display: "flex", flexDirection: "column", gap: "2rem" }}>
      {/* Sweep Sub-Wallet Balances */}
      <div className={styles.drawerSection}>
        <h3 style={{ display: "flex", alignItems: "center", gap: "0.5rem", marginBottom: "1rem" }}>
          <i className="fas fa-broom" style={{ color: "#2563eb" }}></i>
          Sweep Sub-Wallet Balances
        </h3>
        <p style={{ fontSize: "0.85rem", color: "#64748b", marginBottom: "1.5rem" }}>
          Sweep funds internally to your merchant Master Wallet. Gas fees are seamlessly deducted directly from your ledger balance.
        </p>

        <form onSubmit={onSweep} style={{ background: "#fff", border: "1px solid #e2e8f0", padding: "1.5rem", borderRadius: "12px" }}>
          <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: "1rem", marginBottom: "1rem" }}>
            <div className={styles.formGroup} style={{ marginBottom: 0 }}>
              <label>Sweep Mode</label>
              <select
                className={styles.inputStyle}
                value={sweepMode}
                onChange={(e) => setSweepMode(e.target.value as any)}
              >
                <option value="ALL">Sweep All Assets</option>
                <option value="NATIVE_ONLY">Native Coins Only</option>
                <option value="STABLE_ONLY">Stablecoins Only</option>
                <option value="SPECIFIC">Specific Asset</option>
              </select>
            </div>
            
            {sweepMode === "SPECIFIC" && (
              <div className={styles.formGroup} style={{ marginBottom: 0 }}>
                <label>Target Asset</label>
                <select
                  className={styles.inputStyle}
                  value={sweepCryptoType}
                  onChange={(e) => setSweepCryptoType(e.target.value)}
                >
                  {supportedCurrencies.map((c, idx) => (
                    <option key={idx} value={c.crypto_type}>{c.crypto_type}</option>
                  ))}
                </select>
              </div>
            )}

            <div className={styles.formGroup} style={{ marginBottom: 0 }}>
              <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '4px' }}>
                <label style={{ margin: 0 }}>{sweepMode === "SPECIFIC" ? "Amount (Optional)" : "Sweep Details"}</label>
                {customerBalances && (
                  <div style={{ textAlign: 'right', display: 'flex', flexDirection: 'column' }}>
                    {(() => {
                      let filtered = customerBalances;
                      let label = "Total to Sweep";
                      if (sweepMode === "NATIVE_ONLY") {
                        filtered = customerBalances.filter((b: any) => {
                          const ct = b.crypto_type.toUpperCase();
                          return ["BTC", "ETH", "SOL", "BNB", "MATIC"].includes(ct) || ct === "ETHEREUM" || ct === "SOLANA";
                        });
                        label = "Total Native Coins";
                      } else if (sweepMode === "STABLE_ONLY") {
                        filtered = customerBalances.filter((b: any) => {
                          const ct = b.crypto_type.toUpperCase();
                          return ct.includes("USDT") || ct.includes("BUSD") || ct.includes("USDC");
                        });
                        label = "Total Stablecoins";
                      } else if (sweepMode === "SPECIFIC") {
                        const bal = customerBalances.find((b: any) => b.crypto_type === sweepCryptoType);
                        if (!bal) return null;
                        return (
                          <>
                            <span style={{ fontSize: '0.75rem', color: '#059669', fontWeight: 600 }}>
                              Available to Sweep: {parseFloat(bal.locked_balance).toFixed(6)}
                            </span>
                            <span style={{ fontSize: '0.65rem', color: '#94a3b8' }}>
                              ≈ ${parseFloat(bal.locked_balance_usd).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 })} USD
                            </span>
                          </>
                        );
                      }
                      
                      const totalUsd = filtered.reduce((sum: number, b: any) => sum + parseFloat(b.locked_balance_usd || "0"), 0);
                      return (
                        <>
                          <span style={{ fontSize: '0.75rem', color: '#059669', fontWeight: 600 }}>
                            {label}: ${totalUsd.toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 })} USD
                          </span>
                          <span style={{ fontSize: '0.65rem', color: '#94a3b8' }}>
                            {filtered.filter((b: any) => parseFloat(b.locked_balance) > 0).length} assets with balance
                          </span>
                        </>
                      );
                    })()}
                  </div>
                )}
              </div>
              
              {sweepMode === "SPECIFIC" ? (
                <div style={{ position: 'relative' }}>
                  <input
                    className={styles.inputStyle}
                    type="number"
                    step="any"
                    placeholder="Leave blank for MAX"
                    value={sweepAmount}
                    onChange={(e) => setSweepAmount(e.target.value)}
                    style={{ paddingRight: '3.5rem' }}
                  />
                  <button
                    type="button"
                    onClick={() => {
                      const bal = customerBalances?.find((b: any) => b.crypto_type === sweepCryptoType);
                      if (bal) setSweepAmount(bal.locked_balance);
                    }}
                    style={{
                      position: 'absolute', right: '8px', top: '50%', transform: 'translateY(-50%)',
                      padding: '4px 8px', background: '#f1f5f9', border: '1px solid #e2e8f0',
                      borderRadius: '4px', fontSize: '0.7rem', fontWeight: 700, color: '#475569', cursor: 'pointer'
                    }}
                  >MAX</button>
                </div>
              ) : (
                <div style={{ padding: '1rem', background: '#f8fafc', border: '1px dashed #cbd5e1', borderRadius: '10px', textAlign: 'center', color: '#64748b', fontSize: '0.85rem' }}>
                  <i className="fas fa-info-circle" style={{ marginRight: '0.5rem' }}></i>
                  Bulk sweep will process all assets in this category.
                </div>
              )}
            </div>
          </div>
          
          <div className={styles.formGroup}>
            <label>Merchant Transaction PIN</label>
            <input
              className={styles.inputStyle}
              type="password"
              maxLength={4}
              pattern="\d*"
              style={{ letterSpacing: "0.5rem", textAlign: "center" }}
              placeholder="••••"
              value={sweepPin}
              onChange={(e) => setSweepPin(e.target.value.replace(/\D/g, ""))}
              required
            />
          </div>
          
          <button type="submit" className={styles.addBtn} style={{ width: "100%", background: "#2563eb" }} disabled={sweeping}>
            {sweeping ? <i className="fas fa-spinner fa-spin"></i> : "Execute Sweep"}
          </button>
          
          <p style={{ color: "#64748b", fontSize: "0.75rem", marginTop: "0.75rem", textAlign: "center" }}>
            <i className="fas fa-info-circle" style={{ marginRight: '0.25rem' }}></i>
            Required gas limits will be discounted by any native dust already present in the sub-wallet.
          </p>
        </form>
      </div>

      {/* Pay Merchant */}
      <div className={styles.drawerSection}>
        <h3 style={{ display: "flex", alignItems: "center", gap: "0.5rem", marginBottom: "1rem" }}>
          <i className="fas fa-university" style={{ color: "#10b981" }}></i>
          Move to Merchant Balance (Pay Merchant)
        </h3>
        <p style={{ fontSize: "0.85rem", color: "#64748b", marginBottom: "1.5rem" }}>
          Transfer funds from the customer's wallet to your main merchant account immediately.
        </p>

        <form onSubmit={onPayMerchant} style={{ background: "#fff", border: "1px solid #e2e8f0", padding: "1.5rem", borderRadius: "12px" }}>
          <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: "1rem", marginBottom: "1rem" }}>
            <div className={styles.formGroup} style={{ marginBottom: 0 }}>
              <label>Asset</label>
              <select
                className={styles.inputStyle}
                value={payMerchantCryptoType}
                onChange={(e) => setPayMerchantCryptoType(e.target.value)}
              >
                {supportedCurrencies.map((c, idx) => (
                  <option key={idx} value={c.crypto_type}>{c.crypto_type}</option>
                ))}
              </select>
            </div>
            <div className={styles.formGroup} style={{ marginBottom: 0 }}>
              <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '4px' }}>
                <label style={{ margin: 0 }}>Amount</label>
                {customerBalances?.find((b: any) => b.crypto_type === payMerchantCryptoType) && (
                  <div style={{ textAlign: 'right', display: 'flex', flexDirection: 'column' }}>
                    <span style={{ fontSize: '0.75rem', color: '#059669', fontWeight: 600 }}>
                      Customer Balance: {parseFloat(customerBalances.find((b: any) => b.crypto_type === payMerchantCryptoType).available_balance).toFixed(6)}
                    </span>
                    <span style={{ fontSize: '0.65rem', color: '#94a3b8' }}>
                      ≈ ${parseFloat(customerBalances.find((b: any) => b.crypto_type === payMerchantCryptoType).available_balance_usd).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 })} USD
                    </span>
                  </div>
                )}
              </div>
              <div style={{ position: 'relative' }}>
                <input
                  className={styles.inputStyle}
                  type="number"
                  step="any"
                  placeholder="0.00"
                  value={payMerchantAmount}
                  onChange={(e) => setPayMerchantAmount(e.target.value)}
                  required
                  style={{ paddingRight: '3.5rem' }}
                />
                <button
                  type="button"
                  onClick={() => {
                    const bal = customerBalances?.find((b: any) => b.crypto_type === payMerchantCryptoType);
                    if (bal) setPayMerchantAmount(bal.available_balance);
                  }}
                  style={{
                    position: 'absolute', right: '8px', top: '50%', transform: 'translateY(-50%)',
                    padding: '4px 8px', background: '#f1f5f9', border: '1px solid #e2e8f0',
                    borderRadius: '4px', fontSize: '0.7rem', fontWeight: 700, color: '#475569', cursor: 'pointer'
                  }}
                >MAX</button>
              </div>
            </div>
          </div>
          <button type="submit" className={styles.addBtn} style={{ width: "100%", background: "#10b981" }} disabled={payingMerchant}>
            {payingMerchant ? <i className="fas fa-spinner fa-spin"></i> : "Transfer to Merchant Balance"}
          </button>
        </form>
      </div>
    </div>
  );
};

export default ActionsTab;
