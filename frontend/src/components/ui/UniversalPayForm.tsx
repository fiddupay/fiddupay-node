import React, { useState, useEffect } from 'react';
import { 
  MdSearch, 
  MdShield, 
  MdErrorOutline, 
  MdArrowForward, 
  MdRefresh, 
  MdAttachMoney,
  MdCheckCircle
} from 'react-icons/md';
import { publicAPI, merchantAPI } from '@/services/apiService';
import { useToast } from '@/contexts/ToastContext';
import { useBalanceStore } from '@/stores/balanceStore';
import styles from '@/styles/components/UniversalPayForm.module.css';

interface ResolvedMerchant {
  business_id?: number;
  merchant_id: number;
  business_name: string;
  pay_id: string;
  username: string | null;
  kyc_tier: number;
  compliance_status: string;
  trust_score: number;
  social_handle_count: number;
  logo_url: string | null;
}

interface UniversalPayFormProps {
  initialIdentifier?: string;
}

export const UniversalPayForm: React.FC<UniversalPayFormProps> = ({ initialIdentifier }) => {
  const { showToast } = useToast();
  const { balance: walletBalance, fetchBalance } = useBalanceStore();
  const [identifier, setIdentifier] = useState(initialIdentifier || '');
  const [merchant, setMerchant] = useState<ResolvedMerchant | null>(null);
  const [searching, setSearching] = useState(false);
  const [amount, setAmount] = useState('');
  const [selectedCrypto, setSelectedCrypto] = useState('');
  const [pin, setPin] = useState('');
  const [processing, setProcessing] = useState(false);

  useEffect(() => {
    fetchBalance();
  }, []);

  useEffect(() => {
    const timer = setTimeout(() => {
      if (identifier.length >= 3) {
        handleResolve();
      } else {
        setMerchant(null);
      }
    }, 500);
    return () => clearTimeout(timer);
  }, [identifier]);

  const handleResolve = async () => {
    setSearching(true);
    try {
      const response = await publicAPI.resolveMerchant(identifier);
      setMerchant(response.data);
    } catch (error) {
      setMerchant(null);
    } finally {
      setSearching(false);
    }
  };

  const handlePay = async () => {
    if (!merchant || !amount || !selectedCrypto || pin.length !== 4) {
        showToast('Please fill all fields correctly', 'warning');
        return;
    }
    
    setProcessing(true);
    try {
      await merchantAPI.executeTransfer({
        recipient_identifier: identifier,
        crypto_type: selectedCrypto,
        amount: parseFloat(amount),
        pin: pin
      });
      
      showToast(`Payment of $${amount} ${selectedCrypto} sent successfully!`, 'success');
      setAmount('');
      setPin('');
      setMerchant(null);
      setIdentifier('');
      fetchBalance(true);
    } catch (error: any) {
      showToast(error.response?.data?.error || 'Payment failed', 'error');
    } finally {
      setProcessing(false);
    }
  };

  const getTierLabel = (tier: number) => {
    if (tier >= 2) return 'Gold Verified';
    if (tier === 1) return 'Silver Verified';
    return 'Basic';
  };

  const selectedWallet = walletBalance?.balances?.find(b => b.crypto_type === selectedCrypto);

  return (
    <div className={styles.formWrapper}>
      <div className={styles.topBar} />
      
      <div className={styles.header}>
        <h3>
            <MdAttachMoney className="text-primary" />
            Universal Interoperable Pay
        </h3>
        <p>Pay any FidduPay merchant or user with 0 fees using their Email, Username, or PayID.</p>
      </div>

      <div className={styles.section}>
        {/* Recipient Search */}
        <div className={styles.inputGroup}>
          <label className={styles.label}>Recipient Identifier</label>
          <div className="relative">
            <MdSearch className={styles.inputIcon} size={20} />
            <input
              type="text"
              placeholder="Email, @username, or FID-X"
              className={`${styles.inputField} ${styles.inputWithIcon}`}
              value={identifier}
              onChange={(e) => setIdentifier(e.target.value)}
            />
            {searching && <MdRefresh className="absolute right-3 top-1/2 -translate-y-1/2 text-primary animate-spin" size={18} />}
          </div>
        </div>

        {merchant && (
          <div className={styles.merchantCard}>
            <div className={styles.merchantInfo}>
              <div className={styles.merchantMain}>
                <div className={styles.avatar}>
                  {merchant.business_name.charAt(0).toUpperCase()}
                </div>
                <div>
                  <h4 className={styles.businessName}>{merchant.business_name}</h4>
                  <p className={styles.payId}>@{merchant.username || merchant.pay_id}</p>
                </div>
              </div>
              <div style={{ textAlign: 'right' }}>
                <div style={{ 
                    fontSize: '10px', 
                    fontWeight: '800', 
                    padding: '4px 8px', 
                    borderRadius: '6px',
                    background: merchant.kyc_tier >= 2 ? '#f59e0b' : (merchant.kyc_tier === 1 ? '#94a3b8' : '#334155'),
                    color: '#fff',
                    display: 'inline-block'
                }}>
                  {getTierLabel(merchant.kyc_tier)}
                </div>
                <div style={{ fontSize: '10px', color: 'rgba(255,255,255,0.4)', marginTop: '4px' }}>
                    Trust Score: {merchant.trust_score}%
                </div>
              </div>
            </div>

            <div className={styles.badgeGroup}>
                 <div className={styles.badge}>
                    <MdCheckCircle className={styles.verifiedBadge} />
                    <span>Identity Verified</span>
                 </div>
                 {merchant.social_handle_count > 0 && (
                    <div className={styles.badge}>
                        <MdShield className="text-blue-500" />
                        <span>Socials Linked</span>
                    </div>
                 )}
            </div>
          </div>
        )}

        {!merchant && identifier.length >= 3 && !searching && (
           <div className={styles.errorBox}>
              <MdErrorOutline className="text-red-500" size={20} />
              <p className={styles.errorText}>Recipient not found. Double check the identifier.</p>
           </div>
        )}

        {/* Transfer Details */}
        <div style={{ display: 'flex', flexDirection: 'column', gap: '20px', opacity: merchant ? 1 : 0.4, pointerEvents: merchant ? 'auto' : 'none', transition: 'all 0.3s' }}>
          <div className={styles.grid}>
            <div className={styles.inputGroup}>
                <label className={styles.label}>Select Asset</label>
                <div style={{ position: 'relative' }}>
                    <select 
                        className={styles.selectField}
                        value={selectedCrypto}
                        onChange={(e) => setSelectedCrypto(e.target.value)}
                    >
                        <option value="" style={{ background: '#171717' }}>Select Currency</option>
                        {walletBalance?.balances?.map(b => (
                            <option key={b.crypto_type} value={b.crypto_type} style={{ background: '#171717' }}>
                                {b.crypto_type}
                            </option>
                        ))}
                    </select>
                    <div style={{ position: 'absolute', right: '16px', top: '50%', transform: 'translateY(-50%)', pointerEvents: 'none', color: 'rgba(255,255,255,0.3)' }}>
                        ▼
                    </div>
                </div>
                {selectedWallet && (
                    <p style={{ fontSize: '10px', color: 'var(--primary)', marginTop: '6px' }}>Available: {parseFloat(selectedWallet.available_balance).toFixed(6)}</p>
                )}
            </div>

            <div className={styles.inputGroup}>
                <label className={styles.label}>Amount</label>
                <input
                    type="number"
                    placeholder="0.00"
                    className={styles.inputField}
                    value={amount}
                    onChange={(e) => setAmount(e.target.value)}
                />
            </div>
          </div>

          <div className={styles.inputGroup}>
            <label className={styles.label}>Transaction PIN</label>
            <input
                type="password"
                placeholder="••••"
                maxLength={4}
                className={`${styles.inputField} ${styles.pinField}`}
                value={pin}
                onChange={(e) => setPin(e.target.value.replace(/\D/g, ''))}
            />
          </div>

          <button
            onClick={handlePay}
            disabled={!merchant || !amount || !selectedCrypto || pin.length !== 4 || processing}
            className={styles.submitBtn}
          >
            {processing ? (
                <MdRefresh size={24} className="animate-spin" />
            ) : (
                <>Send Secure Payment <MdArrowForward /></>
            )}
          </button>
          <p className={styles.footerNote}>Inter-merchant payments are processed instantly with 0 platform fees.</p>
        </div>
      </div>
    </div>
  );
};

export default UniversalPayForm;
