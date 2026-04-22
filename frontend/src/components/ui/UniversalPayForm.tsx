import React, { useState, useEffect } from 'react';
import { 
  MdSearch, 
  MdShield, 
  MdErrorOutline, 
  MdArrowForward, 
  MdRefresh, 
  MdAttachMoney 
} from 'react-icons/md';
import { publicAPI, merchantAPI } from '@/services/apiService';
import { useToast } from '@/contexts/ToastContext';
import { useBalanceStore } from '@/stores/balanceStore';

interface ResolvedMerchant {
  business_id?: number; // In case we need the internal ID
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

  // Initial load of balances
  useEffect(() => {
    fetchBalance();
  }, []);

  // Debounced search
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
      fetchBalance(true); // Refresh balances
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
    <div className="w-full max-w-lg mx-auto bg-white/5 backdrop-blur-2xl border border-white/10 rounded-3xl shadow-3xl overflow-hidden p-6 space-y-6 relative">
      <div className="bg-gradient-to-r from-primary to-secondary h-1 w-full absolute top-0 left-0" />
      
      <div className="space-y-1">
        <h3 className="text-xl font-bold flex items-center gap-2 text-white">
            <MdAttachMoney className="text-primary" />
            Universal Interoperable Pay
        </h3>
        <p className="text-xs text-gray-400">Pay any FidduPay merchant or user with 0 fees using their Email, Username, or PayID.</p>
      </div>

      <div className="space-y-6">
        {/* Recipient Search */}
        <div className="space-y-2">
          <label className="text-xs font-bold text-gray-500 uppercase">Recipient Identifier</label>
          <div className="relative">
            <MdSearch className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-500" />
            <input
              type="text"
              placeholder="Email, @username, or FID-XXXX-XXXX"
              className="w-full bg-white/5 border border-white/10 rounded-xl pl-10 pr-4 py-3 text-white focus:border-primary outline-none transition-all"
              value={identifier}
              onChange={(e) => setIdentifier(e.target.value)}
            />
            {searching && <MdRefresh className="absolute right-3 top-1/2 -translate-y-1/2 w-4 h-4 text-primary animate-spin" />}
          </div>
        </div>

        {merchant && (
          <div className="p-4 rounded-xl bg-primary/5 border border-primary/20 space-y-4 animate-in fade-in zoom-in duration-300">
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-3">
                <div className="w-12 h-12 rounded-full bg-gradient-to-br from-primary/20 to-secondary/20 flex items-center justify-center text-xl font-bold text-primary border border-primary/30">
                  {merchant.business_name.charAt(0).toUpperCase()}
                </div>
                <div>
                  <h4 className="text-white font-bold leading-tight">{merchant.business_name}</h4>
                  <p className="text-[10px] text-gray-400 font-mono">@{merchant.username || merchant.pay_id}</p>
                </div>
              </div>
              <div className="text-right">
                <div className={`px-2 py-1 rounded text-[10px] font-bold ${merchant.kyc_tier >= 2 ? 'bg-amber-500 text-white' : (merchant.kyc_tier === 1 ? 'bg-slate-400 text-white' : 'bg-gray-700 text-gray-300')}`}>
                  {getTierLabel(merchant.kyc_tier)}
                </div>
                <div className="text-[10px] text-gray-500 mt-1">Trust Score: {merchant.trust_score}%</div>
              </div>
            </div>

            <div className="flex gap-2">
                 <div className="flex-1 px-3 py-2 rounded-lg bg-black/20 border border-white/5 flex items-center gap-2">
                    <MdShield className="w-3 h-3 text-green-500" />
                    <span className="text-[10px] text-gray-300">Identity Verified</span>
                 </div>
                 {merchant.social_handle_count > 0 && (
                    <div className="flex-1 px-3 py-2 rounded-lg bg-black/20 border border-white/5 flex items-center gap-2">
                        <MdShield className="w-3 h-3 text-blue-500" />
                        <span className="text-[10px] text-gray-300">Socials Linked</span>
                    </div>
                 )}
            </div>
          </div>
        )}

        {!merchant && identifier.length >= 3 && !searching && (
           <div className="p-4 rounded-xl bg-red-500/5 border border-red-500/20 flex items-center gap-3">
              <MdErrorOutline className="text-red-500 w-5 h-5 shrink-0" />
              <p className="text-xs text-red-300">Recipient not found. Double check the identifier.</p>
           </div>
        )}

        {/* Transfer Details */}
        <div className={`space-y-4 transition-all duration-500 ${merchant ? 'opacity-100' : 'opacity-30 pointer-events-none'}`}>
          <div className="grid grid-cols-2 gap-4">
            <div className="space-y-2">
                <label className="text-xs font-bold text-gray-500 uppercase">Select Asset</label>
                <select 
                    className="w-full bg-white/5 border border-white/10 rounded-xl px-4 py-3 text-white focus:border-primary outline-none appearance-none"
                    value={selectedCrypto}
                    onChange={(e) => setSelectedCrypto(e.target.value)}
                >
                    <option value="" className="bg-slate-900">Select Currency</option>
                    {walletBalance?.balances?.map(b => (
                        <option key={b.crypto_type} value={b.crypto_type} className="bg-slate-900">
                            {b.crypto_type} (Avail: {parseFloat(b.available_balance).toFixed(4)})
                        </option>
                    ))}
                </select>
                {selectedWallet && (
                    <p className="text-[10px] text-primary mt-1">Available: {parseFloat(selectedWallet.available_balance).toFixed(6)} {selectedCrypto}</p>
                )}
            </div>

            <div className="space-y-2">
                <label className="text-xs font-bold text-gray-500 uppercase">Amount (Units)</label>
                <input
                    type="number"
                    placeholder="0.00"
                    className="w-full bg-white/5 border border-white/10 rounded-xl px-4 py-3 text-white focus:border-primary outline-none"
                    value={amount}
                    onChange={(e) => setAmount(e.target.value)}
                />
            </div>
          </div>

          <div className="space-y-2">
            <label className="text-xs font-bold text-gray-500 uppercase">Transaction PIN</label>
            <input
                type="password"
                placeholder="4-digit PIN"
                maxLength={4}
                className="w-full bg-white/5 border border-white/10 rounded-xl px-4 py-3 text-center tracking-[1em] text-white focus:border-primary outline-none"
                value={pin}
                onChange={(e) => setPin(e.target.value.replace(/\D/g, ''))}
            />
          </div>

          <button
            onClick={handlePay}
            disabled={!merchant || !amount || !selectedCrypto || pin.length !== 4 || processing}
            className="w-full group bg-gradient-to-r from-primary to-primary-hover text-white py-4 rounded-xl font-bold flex items-center justify-center gap-2 shadow-lg shadow-primary/20 hover:scale-[1.02] active:scale-[0.98] transition-all disabled:opacity-50 disabled:hover:scale-100"
          >
            {processing ? (
                <MdRefresh className="w-5 h-5 animate-spin" />
            ) : (
                <>Send Secure Payment <MdArrowForward className="group-hover:translate-x-1 transition-transform" /></>
            )}
          </button>
          <p className="text-center text-[10px] text-gray-500 italic">Inter-merchant payments are processed instantly with 0 platform fees.</p>
        </div>
      </div>
    </div>
  );
};

export default UniversalPayForm;
