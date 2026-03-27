import React from 'react';
import styles from "../styles/CustomerDrawers.module.css";
import { FaWallet } from "react-icons/fa";
import { Wallet } from "../types";

interface OverviewTabProps {
  supportedCurrencies: any[];
  customerBalances: any;
  customerWallets: Wallet[];
  expandedAsset: string | null;
  setExpandedAsset: (val: string | null) => void;
  onProvisionWallets: () => void;
  provisioning: boolean;
  onCopy: (text: string) => void;
}

const OverviewTab: React.FC<OverviewTabProps> = ({
  supportedCurrencies,
  customerBalances,
  customerWallets,
  expandedAsset,
  setExpandedAsset,
  onProvisionWallets,
  provisioning,
  onCopy
}) => {
  return (
    <>
      <div className={styles.drawerSection}>
        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "1.5rem" }}>
          <h3 style={{ margin: 0 }}>
            <i className="fas fa-wallet" style={{ color: "#2563eb" }}></i>{" "}
            Portfolio
          </h3>
          {customerWallets.length === 0 && (
            <button
              onClick={onProvisionWallets}
              className={styles.provisionBtn}
              style={{ width: "auto", padding: "0.5rem 1rem" }}
              disabled={provisioning}
            >
              {provisioning ? <i className="fas fa-circle-notch fa-spin"></i> : <i className="fas fa-magic"></i>}
              Provision All
            </button>
          )}
        </div>

        <div className={styles.portfolioList}>
          {supportedCurrencies.length > 0 ? (
            supportedCurrencies.map((asset: any) => {
              const balance = customerBalances?.find((b: any) => b.crypto_type === asset.crypto_type);
              const wallet = customerWallets.find((w: any) => w.crypto_type === asset.crypto_type);
              const isExpanded = expandedAsset === asset.crypto_type;

              return (
                <div
                  key={asset.crypto_type}
                  className={`${styles.portfolioItem} ${isExpanded ? styles.expanded : ""}`}
                  onClick={() => setExpandedAsset(isExpanded ? null : asset.crypto_type)}
                >
                  <div className={styles.portfolioMain}>
                    <div className={styles.assetInfoGroup}>
                      <div className={styles.assetIconSmall}>
                        {asset.icon_url ? (
                          <img src={asset.icon_url} alt={asset.crypto_type} />
                        ) : (
                          <FaWallet />
                        )}
                      </div>
                      <div className={styles.assetMetaGroup}>
                        <span className={styles.assetCode}>{asset.crypto_type}</span>
                        <span className={styles.assetNetworkName}>{asset.network}</span>
                      </div>
                    </div>

                    <div className={styles.assetBalanceGroup}>
                      <span className={styles.balanceValue}>
                        {parseFloat(balance?.available_balance || "0").toFixed(6)}
                      </span>
                      <span className={styles.usdValue}>
                        ≈ ${parseFloat(balance?.available_balance_usd || "0").toLocaleString(undefined, {
                          minimumFractionDigits: 2,
                          maximumFractionDigits: 2,
                        })}
                      </span>
                    </div>
                  </div>

                  {isExpanded && (
                    <div className={styles.expandedDetails} onClick={(e) => e.stopPropagation()}>
                      {wallet ? (
                        <div className={styles.addressContainer}>
                          <label>Deposit Address</label>
                          <div className={styles.addressRow}>
                            <code className={styles.addressCode}>{wallet.address}</code>
                            <button
                              className={styles.miniCopyBtn}
                              onClick={() => {
                                onCopy(wallet.address);
                              }}
                            >
                              <i className="fas fa-copy"></i>
                            </button>
                          </div>
                        </div>
                      ) : (
                        <div style={{ textAlign: 'center', padding: '1rem' }}>
                          <p style={{ fontSize: '0.8rem', color: '#64748b', marginBottom: '0.75rem' }}>No wallet address generated yet.</p>
                        </div>
                      )}
                    </div>
                  )}
                </div>
              );
            })
          ) : (
            <div style={{ padding: "2rem", textAlign: "center", color: "#64748b" }}>
              <i className="fas fa-coins fa-2x" style={{ marginBottom: "1rem" }}></i>
              <p>No currencies supported by your merchant account.</p>
            </div>
          )}
        </div>
      </div>
    </>
  );
};

export default OverviewTab;
