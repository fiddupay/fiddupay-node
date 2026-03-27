import React from 'react';
import styles from "./styles/CustomerDrawers.module.css";

interface CreateCustomerProps {
  isOpen: boolean;
  onClose: () => void;
  newCustomer: {
    external_id: string;
    email: string;
    first_name: string;
    last_name: string;
  };
  setNewCustomer: (val: any) => void;
  submitting: boolean;
  onSubmit: (e: React.FormEvent) => void;
}

const CreateCustomerDrawer: React.FC<CreateCustomerProps> = ({
  isOpen,
  onClose,
  newCustomer,
  setNewCustomer,
  submitting,
  onSubmit
}) => {
  if (!isOpen) return null;

  return (
    <div className={styles.overlay} onClick={onClose}>
      <div className={styles.drawer} onClick={(e) => e.stopPropagation()}>
        <div className={styles.drawerHeader}>
          <h2>
            <i className="fas fa-user-plus" style={{ color: "#2563eb" }}></i>{" "}
            New Customer
          </h2>
          <button className={styles.closeBtn} onClick={onClose}>
            <i className="fas fa-times"></i>
          </button>
        </div>
        <div className={styles.drawerBody}>
          <form onSubmit={onSubmit}>
            <div className={styles.formGroup}>
              <label>External Reference ID*</label>
              <div className={styles.inputGroup}>
                <i className="fas fa-id-card"></i>
                <input
                  className={styles.inputStyle}
                  required
                  placeholder="e.g. system_user_99"
                  value={newCustomer.external_id}
                  onChange={(e) =>
                    setNewCustomer({
                      ...newCustomer,
                      external_id: e.target.value,
                    })
                  }
                />
              </div>
              <p style={{ fontSize: "0.75rem", color: "#64748b", marginTop: "0.5rem" }}>
                Must be unique. Wallets will be auto-provisioned upon registration.
              </p>
            </div>
            <div className={styles.formGroup}>
              <label>Email Address</label>
              <div className={styles.inputGroup}>
                <i className="fas fa-envelope"></i>
                <input
                  className={styles.inputStyle}
                  type="email"
                  placeholder="customer@domain.com"
                  value={newCustomer.email}
                  onChange={(e) =>
                    setNewCustomer({
                      ...newCustomer,
                      email: e.target.value,
                    })
                  }
                />
              </div>
            </div>
            <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: "1.5rem" }}>
              <div className={styles.formGroup}>
                <label>First Name</label>
                <div className={styles.inputGroup}>
                  <i className="fas fa-user-circle"></i>
                  <input
                    className={styles.inputStyle}
                    placeholder="John"
                    value={newCustomer.first_name}
                    onChange={(e) =>
                      setNewCustomer({
                        ...newCustomer,
                        first_name: e.target.value,
                      })
                    }
                  />
                </div>
              </div>
              <div className={styles.formGroup}>
                <label>Last Name</label>
                <div className={styles.inputGroup}>
                  <i className="fas fa-user-circle"></i>
                  <input
                    className={styles.inputStyle}
                    placeholder="Doe"
                    value={newCustomer.last_name}
                    onChange={(e) =>
                      setNewCustomer({
                        ...newCustomer,
                        last_name: e.target.value,
                      })
                    }
                  />
                </div>
              </div>
            </div>
            <button
              className={styles.addBtn}
              style={{ width: "100%", marginTop: "2rem" }}
              disabled={submitting}
            >
              {submitting ? (
                <i className="fas fa-circle-notch fa-spin"></i>
              ) : (
                "Complete Registration"
              )}
            </button>
          </form>
        </div>
      </div>
    </div>
  );
};

export default CreateCustomerDrawer;
