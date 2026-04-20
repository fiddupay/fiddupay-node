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
    <div style={{ display: "flex", flexDirection: "column", gap: "2.5rem" }}>
      {/* Sweep Sub-Wallet Balances */}
      <div className={styles.drawerSection} style={{ padding: 0, border: 'none', background: 'transparent', boxShadow: 'none' }}>
        <h3 style={{ display: "flex", alignItems: "center", gap: "0.75rem", marginBottom: "1.25rem", padding: '0 1rem' }}>
          <div style={{ width: '32px', height: '32px', borderRadius: '10px', background: 'rgba(37, 99, 235, 0.1)', display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
            <i className="fas fa-broom" style={{ color: "#3b82f6", fontSize: '0.9rem' }}></i>
          </div>
          Sweep Sub-Wallet Balances
        </h3>
        <p style={{ fontSize: "0.85rem", color: "var(--text-muted)", marginBottom: "1.5rem", padding: '0 1rem', lineHeight: 1.5 }}>
          Sweep funds internally to your merchant Master Wallet. Gas fees are seamlessly deducted directly from your ledger balance.
        </p>

        <form onSubmit={onSweep} className={styles.financialForm}>
          <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: "1.5rem", marginBottom: "1.5rem" }}>
            <div className={styles.formGroup} style={{ marginBottom: 0 }}>
              <label>Sweep Mode</label>
              <div className={styles.inputGroup} style={{ position: 'relative' }}>
                <i className="fas fa-layer-group" style={{ position: 'absolute', left: '1.25rem', top: '50%', transform: 'translateY(-50%)', color: 'var(--text-muted)', fontSize: '0.9rem' }}></i>
                <select
                  className={styles.inputStyle}
                  value={sweepMode}
                  onChange={(e) => setSweepMode(e.target.value as any)}
                  style={{ paddingLeft: '3rem' }}
                >
                  <option value="ALL">Sweep All Assets</option>
                  <option value="NATIVE_ONLY">Native Coins Only</option>
                  <option value="STABLE_ONLY">Stablecoins Only</option>
                  <option value="SPECIFIC">Specific Asset</option>
                </select>
              </div>
            </div>
            
            <div className={styles.formGroup} style={{ marginBottom: 0 }}>
              <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '8px' }}>
                <label style={{ margin: 0 }}>{sweepMode === "SPECIFIC" ? "Target Asset" : "Sweep Summary"}</label>
              </div>
              
              {sweepMode === "SPECIFIC" ? (
                <div className={styles.inputGroup} style={{ position: 'relative' }}>
                  <i className="fas fa-coins" style={{ position: 'absolute', left: '1.25rem', top: '50%', transform: 'translateY(-50%)', color: 'var(--text-muted)', fontSize: '0.9rem' }}></i>
                  <select
                    className={styles.inputStyle}
                    value={sweepCryptoType}
                    onChange={(e) => setSweepCryptoType(e.target.value)}
                    style={{ paddingLeft: '3rem' }}
                  >
                    {supportedCurrencies.map((c, idx) => (
                      <option key={idx} value={c.crypto_type}>{c.crypto_type}</option>
                    ))}
                  </select>
                </div>
              ) : (
                <div style={{ height: '52px', display: 'flex', alignItems: 'center' }}>
                   {customerBalances && (
                    <div style={{ display: 'flex', alignItems: 'center', gap: '0.75rem' }}>
                      {(() => {
                        let filtered = customerBalances;
                        if (sweepMode === "NATIVE_ONLY") {
                          filtered = customerBalances.filter((b: any) => {
                            const ct = b.crypto_type.toUpperCase();
                            return ["BTC", "ETH", "SOL", "BNB", "MATIC"].includes(ct) || ct === "ETHEREUM" || ct === "SOLANA";
                          });
                        } else if (sweepMode === "STABLE_ONLY") {
                          filtered = customerBalances.filter((b: any) => {
                            const ct = b.crypto_type.toUpperCase();
                            return ct.includes("USDT") || ct.includes("BUSD") || ct.includes("USDC");
                          });
                        }
                        
                        const totalUsd = filtered.reduce((sum: number, b: any) => sum + parseFloat(b.locked_balance_usd || "0"), 0);
                        return (
                          <>
                            <span className={styles.valueBadge}>
                              ${totalUsd.toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 })} USD
                            </span>
                            <span className={styles.usdSmall}>
                              across {filtered.filter((b: any) => parseFloat(b.locked_balance) > 0).length} assets
                            </span>
                          </>
                        );
                      })()}
                    </div>
                  )}
                </div>
              )}
            </div>
          </div>

          <div className={styles.formGroup} style={{ marginBottom: sweepMode === "SPECIFIC" ? '2rem' : '1.5rem' }}>
              <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '8px' }}>
                <label style={{ margin: 0 }}>{sweepMode === "SPECIFIC" ? "Amount to Sweep" : "Instructions"}</label>
                {sweepMode === "SPECIFIC" && customerBalances && (
                   <span className={styles.usdSmall} style={{ color: 'var(--primary)' }}>
                      Available: {parseFloat(customerBalances.find((b: any) => b.crypto_type === sweepCryptoType)?.locked_balance || "0").toFixed(6)}
                   </span>
                )}
              </div>

              {sweepMode === "SPECIFIC" ? (
                <div className={styles.inputGroup} style={{ position: 'relative' }}>
                   <i className="fas fa-hand-holding-usd" style={{ position: 'absolute', left: '1.25rem', top: '50%', transform: 'translateY(-50%)', color: 'var(--text-muted)', fontSize: '0.9rem' }}></i>
                   <input
                    className={styles.inputStyle}
                    type="number"
                    step="any"
                    placeholder="Leave blank for MAX"
                    value={sweepAmount}
                    onChange={(e) => setSweepAmount(e.target.value)}
                    style={{ paddingLeft: '3rem', paddingRight: '4rem' }}
                  />
                  <button
                    type="button"
                    className={styles.maxBtn}
                    onClick={() => {
                      const bal = customerBalances?.find((b: any) => b.crypto_type === sweepCryptoType);
                      if (bal) setSweepAmount(bal.locked_balance);
                    }}
                  >MAX</button>
                  {sweepAmount && (
                    <div style={{ position: 'absolute', right: '0', bottom: '-22px' }}>
                       {(() => {
                          const bal = customerBalances?.find((b: any) => b.crypto_type === sweepCryptoType);
                          if (!bal) return null;
                          const rate = parseFloat(bal.locked_balance_usd) / parseFloat(bal.locked_balance);
                          const usdVal = parseFloat(sweepAmount) * (rate || 0);
                          return <span className={styles.usdSmall}>≈ ${usdVal.toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 })} USD</span>
                       })()}
                    </div>
                  )}
                </div>
              ) : (
                <div className={styles.infoBox}>
                  <i className="fas fa-info-circle"></i>
                  Bulk sweep will process all confirmed sub-wallet balances in the selected category.
                </div>
              )}
          </div>
          
          <div className={styles.formGroup}>
            <label>Merchant Transaction PIN</label>
            <div className={styles.inputGroup} style={{ position: 'relative' }}>
              <i className="fas fa-lock" style={{ position: 'absolute', left: '1.25rem', top: '50%', transform: 'translateY(-50%)', color: 'var(--text-muted)', fontSize: '0.9rem' }}></i>
              <input
                className={styles.inputStyle}
                type="password"
                maxLength={4}
                pattern="\d*"
                style={{ letterSpacing: "0.8rem", textAlign: "center", paddingLeft: '1.5rem', paddingRight: '1.5rem' }}
                placeholder="••••"
                value={sweepPin}
                onChange={(e) => setSweepPin(e.target.value.replace(/\D/g, ""))}
                required
              />
            </div>
          </div>
          
          <button type="submit" className={styles.addBtn} style={{ width: "100%", height: '56px' }} disabled={sweeping}>
            {sweeping ? <i className="fas fa-spinner fa-spin"></i> : (
              <>
                <i className="fas fa-rocket"></i>
                Execute Sweep
              </>
            )}
          </button>
          
          <p style={{ color: "var(--text-muted)", fontSize: "0.75rem", marginTop: "1rem", textAlign: "center", opacity: 0.8 }}>
            Required gas limits will be discounted by any native dust in the sub-wallet.
          </p>
        </form>
      </div>

      {/* Pay Merchant */}
      <div className={styles.drawerSection} style={{ padding: 0, border: 'none', background: 'transparent', boxShadow: 'none' }}>
        <h3 style={{ display: "flex", alignItems: "center", gap: "0.75rem", marginBottom: "1.25rem", padding: '0 1rem' }}>
          <div style={{ width: '32px', height: '32px', borderRadius: '10px', background: 'rgba(16, 185, 129, 0.1)', display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
            <i className="fas fa-university" style={{ color: "#10b981", fontSize: '0.9rem' }}></i>
          </div>
          Move to Merchant Balance (Pay Merchant)
        </h3>
        <p style={{ fontSize: "0.85rem", color: "var(--text-muted)", marginBottom: "1.5rem", padding: '0 1rem', lineHeight: 1.5 }}>
          Instantly transfer funds from the customer's wallet to your merchant ledger.
        </p>

        <form onSubmit={onPayMerchant} className={styles.financialForm} style={{ borderColor: 'rgba(16, 185, 129, 0.2)' }}>
          <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: "1.5rem", marginBottom: "1.5rem" }}>
            <div className={styles.formGroup} style={{ marginBottom: 0 }}>
              <label>Select Asset</label>
              <div className={styles.inputGroup} style={{ position: 'relative' }}>
                <i className="fas fa-coins" style={{ position: 'absolute', left: '1.25rem', top: '50%', transform: 'translateY(-50%)', color: 'var(--text-muted)', fontSize: '0.9rem' }}></i>
                <select
                  className={styles.inputStyle}
                  value={payMerchantCryptoType}
                  onChange={(e) => setPayMerchantCryptoType(e.target.value)}
                  style={{ paddingLeft: '3rem' }}
                >
                  {supportedCurrencies.map((c, idx) => (
                    <option key={idx} value={c.crypto_type}>{c.crypto_type}</option>
                  ))}
                </select>
              </div>
            </div>
            <div className={styles.formGroup} style={{ marginBottom: 0 }}>
              <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '8px' }}>
                <label style={{ margin: 0 }}>Amount</label>
                {customerBalances?.find((b: any) => b.crypto_type === payMerchantCryptoType) && (
                   <span className={styles.usdSmall} style={{ color: '#10b981' }}>
                      Bal: {parseFloat(customerBalances.find((b: any) => b.crypto_type === payMerchantCryptoType).available_balance).toFixed(6)}
                   </span>
                )}
              </div>
              <div className={styles.inputGroup} style={{ position: 'relative' }}>
                <i className="fas fa-money-bill-wave" style={{ position: 'absolute', left: '1.25rem', top: '50%', transform: 'translateY(-50%)', color: 'var(--text-muted)', fontSize: '0.9rem' }}></i>
                <input
                  className={styles.inputStyle}
                  type="number"
                  step="any"
                  placeholder="0.00"
                  value={payMerchantAmount}
                  onChange={(e) => setPayMerchantAmount(e.target.value)}
                  required
                  style={{ paddingLeft: '3rem', paddingRight: '4rem' }}
                />
                <button
                  type="button"
                  className={styles.maxBtn}
                  onClick={() => {
                    const bal = customerBalances?.find((b: any) => b.crypto_type === payMerchantCryptoType);
                    if (bal) setPayMerchantAmount(bal.available_balance);
                  }}
                >MAX</button>
                {payMerchantAmount && (
                  <div style={{ position: 'absolute', right: '0', bottom: '-22px' }}>
                     {(() => {
                        const bal = customerBalances?.find((b: any) => b.crypto_type === payMerchantCryptoType);
                        if (!bal) return null;
                        const rate = parseFloat(bal.available_balance_usd) / parseFloat(bal.available_balance);
                        const usdVal = parseFloat(payMerchantAmount) * (rate || 0);
                        return <span className={styles.usdSmall}>≈ ${usdVal.toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 })} USD</span>
                     })()}
                  </div>
                )}
              </div>
            </div>
          </div>
          <button type="submit" className={styles.addBtn} style={{ width: "100%", background: "#10b981", height: '56px', boxShadow: '0 10px 20px rgba(16, 185, 129, 0.2)' }} disabled={payingMerchant}>
            {payingMerchant ? <i className="fas fa-spinner fa-spin"></i> : (
              <>
                <i className="fas fa-share-square"></i>
                Transfer to Merchant Balance
              </>
            )}
          </button>
        </form>
      </div>
    </div>
  );
};

export default ActionsTab;
