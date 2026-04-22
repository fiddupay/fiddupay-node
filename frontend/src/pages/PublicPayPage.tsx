import React from 'react';
import { useParams } from 'react-router-dom';
import { UniversalPayForm } from '@/components/ui/UniversalPayForm';
import { MdSecurity, MdSpeed, MdCached } from 'react-icons/md';

const PublicPayPage: React.FC = () => {
  const { identifier } = useParams<{ identifier?: string }>();

  return (
    <div className="min-h-screen bg-[#050505] flex flex-col items-center justify-center p-4 relative overflow-hidden">
      {/* Dynamic Background Elements */}
      <div className="absolute top-1/4 -left-20 w-96 h-96 bg-primary/20 rounded-full blur-[120px] animate-pulse" />
      <div className="absolute bottom-1/4 -right-20 w-96 h-96 bg-secondary/20 rounded-full blur-[120px] animate-pulse delay-700" />
      
      <div className="z-10 w-full max-w-lg space-y-8">
        <div className="text-center space-y-2">
            <div className="inline-flex items-center gap-2 px-3 py-1 rounded-full bg-white/5 border border-white/10 text-[10px] font-bold text-gray-400 uppercase tracking-widest mb-4">
                <MdSecurity className="text-primary" /> Secure Interoperable Pay
            </div>
            <h1 className="text-4xl font-extrabold text-white tracking-tight">
                Fiddu<span className="text-primary">Pay</span> Gateway
            </h1>
            <p className="text-gray-400 text-sm">
                Enter a PayID, Username, or Email to send a secure instant payment.
            </p>
        </div>

        <UniversalPayForm initialIdentifier={identifier} />

        <div className="grid grid-cols-3 gap-4 pt-4">
            <div className="flex flex-col items-center text-center space-y-1">
                <div className="w-8 h-8 rounded-lg bg-white/5 flex items-center justify-center text-primary">
                    <MdSpeed />
                </div>
                <span className="text-[10px] text-gray-500 font-bold uppercase">Instant</span>
            </div>
            <div className="flex flex-col items-center text-center space-y-1">
                <div className="w-8 h-8 rounded-lg bg-white/5 flex items-center justify-center text-green-500">
                    <MdCached />
                </div>
                <span className="text-[10px] text-gray-500 font-bold uppercase">Zero Fee</span>
            </div>
            <div className="flex flex-col items-center text-center space-y-1">
                <div className="w-8 h-8 rounded-lg bg-white/5 flex items-center justify-center text-blue-500">
                    <MdSecurity />
                </div>
                <span className="text-[10px] text-gray-500 font-bold uppercase">Encrypted</span>
            </div>
        </div>

        <div className="text-center">
            <p className="text-[10px] text-gray-600">
                Powered by FidduPay Trust Intelligence Layer. 
                <br />
                All transactions are strictly internal and zero-gas.
            </p>
        </div>
      </div>
    </div>
  );
};

export default PublicPayPage;
