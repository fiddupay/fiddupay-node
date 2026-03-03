// Required for VSCode TS Server Indexing
import { useState } from 'react';
import { useParams, Link } from 'react-router-dom';
import { Shield, Clock, MessageSquare, AlertTriangle, ArrowLeft, CheckCircle2 } from 'lucide-react';

export default function TradingRoom() {
    const { tradeId } = useParams();
    const [status, setStatus] = useState<'PENDING' | 'PAID' | 'COMPLETED'>('PENDING');

    return (
        <div className="fade-in">
            <Link to="/orders" className="flex items-center gap-2 mb-6 nav-link" style={{ width: 'fit-content' }}>
                <ArrowLeft size={16} /> Back to Orders
            </Link>

            <div className="flex gap-6">
                {/* Left Column: Trade Details & Actions */}
                <div className="flex-col gap-6" style={{ flex: 2 }}>

                    {/* Status Alert Banner */}
                    <div className="panel flex items-center gap-4" style={{ backgroundColor: status === 'COMPLETED' ? 'rgba(35, 134, 54, 0.1)' : 'rgba(88, 166, 255, 0.1)', borderColor: status === 'COMPLETED' ? 'var(--accent-buy)' : 'var(--primary-color)' }}>
                        {status === 'COMPLETED' ? <CheckCircle2 size={32} color="var(--accent-buy)" /> : <Clock size={32} color="var(--primary-color)" />}
                        <div>
                            <h2 style={{ fontSize: '1.25rem' }}>
                                {status === 'PENDING' && 'Pending Payment Confirmation'}
                                {status === 'PAID' && 'Waiting for Seller to Release Crypto'}
                                {status === 'COMPLETED' && 'Order Completed Successfully'}
                            </h2>
                            <p style={{ color: 'var(--text-secondary)' }}>
                                {status === 'PENDING' && 'Please transfer the exact fiat amount to the seller within 14:59.'}
                                {status === 'PAID' && 'You have marked this order as paid. The seller is verifying the transfer.'}
                                {status === 'COMPLETED' && '1,000 USDT has been credited to your P2P Wallet.'}
                            </p>
                        </div>
                    </div>

                    {/* Order Details */}
                    <div className="panel">
                        <h3 className="mb-4" style={{ paddingBottom: '1rem', borderBottom: '1px solid var(--border-color)', marginBottom: '1rem' }}>Order #{tradeId || '12345'} Information</h3>

                        <div className="flex justify-between items-center" style={{ marginBottom: '1.5rem' }}>
                            <div className="flex-col gap-1">
                                <span style={{ color: 'var(--text-secondary)', fontSize: '0.875rem' }}>Amount to Pay</span>
                                <span style={{ fontSize: '1.5rem', fontWeight: 700, color: 'var(--accent-buy)' }}>₦1,650,000.00</span>
                            </div>
                            <div className="flex-col gap-1" style={{ textAlign: 'right' }}>
                                <span style={{ color: 'var(--text-secondary)', fontSize: '0.875rem' }}>You will receive</span>
                                <span style={{ fontSize: '1.5rem', fontWeight: 700 }}>1,000.00 <span style={{ color: 'var(--text-secondary)', fontSize: '1rem' }}>USDT</span></span>
                            </div>
                        </div>

                        <div style={{ backgroundColor: 'var(--bg-color)', padding: '1rem', borderRadius: '6px', marginBottom: '1.5rem' }}>
                            <div className="flex items-center gap-2" style={{ color: 'var(--accent-buy)', marginBottom: '0.5rem', fontWeight: 500, fontSize: '0.875rem' }}>
                                <Shield size={16} /> Crypto is securely locked in FidduPay Escrow
                            </div>
                            <p style={{ color: 'var(--text-secondary)', fontSize: '0.875rem' }}>
                                The 1,000 USDT has been deducted from the seller's account and is currently locked by the platform. It will be released automatically once you confirm payment and the seller verifies receipt.
                            </p>
                        </div>

                        {/* Payment Details */}
                        <h4 style={{ marginBottom: '1rem' }}>Seller's Payment Details</h4>
                        <div className="flex-col gap-4">
                            <div className="flex justify-between" style={{ padding: '0.75rem', backgroundColor: 'var(--bg-color)', borderRadius: '6px' }}>
                                <span style={{ color: 'var(--text-secondary)' }}>Full Name</span>
                                <span style={{ fontWeight: 600 }}>Chukwudi Okafor</span>
                            </div>
                            <div className="flex justify-between" style={{ padding: '0.75rem', backgroundColor: 'var(--bg-color)', borderRadius: '6px' }}>
                                <span style={{ color: 'var(--text-secondary)' }}>Bank Name</span>
                                <span style={{ fontWeight: 600 }}>Guaranty Trust Bank (GTB)</span>
                            </div>
                            <div className="flex justify-between" style={{ padding: '0.75rem', backgroundColor: 'var(--bg-color)', borderRadius: '6px' }}>
                                <span style={{ color: 'var(--text-secondary)' }}>Account Number</span>
                                <div className="flex items-center gap-2">
                                    <span style={{ fontWeight: 600, fontSize: '1.125rem' }}>0123456789</span>
                                    <button className="btn btn-outline" style={{ padding: '0.25rem 0.5rem', fontSize: '0.75rem' }}>Copy</button>
                                </div>
                            </div>
                        </div>

                        <div style={{ borderTop: '1px solid var(--border-color)', marginTop: '2rem', paddingTop: '1.5rem', display: 'flex', gap: '1rem', justifyContent: 'flex-end' }}>
                            <button className="btn" style={{ backgroundColor: 'transparent', color: 'var(--text-secondary)' }}>Cancel Order</button>

                            {status === 'PENDING' && (
                                <button className="btn btn-primary" style={{ backgroundColor: 'var(--primary-color)', color: '#fff', padding: '0.75rem 2rem' }} onClick={() => setStatus('PAID')}>
                                    Transferred, Notify Seller
                                </button>
                            )}

                            {status === 'PAID' && (
                                <button className="btn" style={{ backgroundColor: 'var(--bg-color)', color: 'var(--text-secondary)', padding: '0.75rem 2rem', cursor: 'not-allowed' }} disabled>
                                    Waiting for Release...
                                </button>
                            )}

                            {status === 'PAID' && (
                                <button className="btn btn-outline" style={{ color: 'var(--accent-sell)', borderColor: 'var(--accent-sell)' }} onClick={() => setStatus('COMPLETED')}>
                                    <AlertTriangle size={16} /> Appeal
                                </button>
                            )}
                        </div>
                    </div>
                </div>

                {/* Right Column: Chat */}
                <div className="panel flex-col" style={{ flex: 1, padding: 0, height: 'calc(100vh - 150px)', position: 'sticky', top: '80px' }}>
                    <div style={{ padding: '1rem', borderBottom: '1px solid var(--border-color)', display: 'flex', alignItems: 'center', gap: '0.5rem', fontWeight: 600 }}>
                        <MessageSquare size={18} /> Order Chat
                    </div>

                    <div style={{ flex: 1, overflowY: 'auto', padding: '1rem', display: 'flex', flexDirection: 'column', gap: '1rem' }}>
                        <div style={{ alignSelf: 'center', fontSize: '0.75rem', color: 'var(--text-secondary)', backgroundColor: 'var(--bg-color)', padding: '0.25rem 0.75rem', borderRadius: '9999px' }}>
                            Order created remotely. Escrow locked.
                        </div>

                        <div style={{ alignSelf: 'flex-start', maxWidth: '80%' }}>
                            <div style={{ backgroundColor: 'var(--bg-color)', padding: '0.75rem', borderRadius: '8px', borderTopLeftRadius: 0, color: 'var(--text-primary)', fontSize: '0.875rem' }}>
                                Hello! I'm online. Please proceed with the payment, drop proof here, and I'll release immediately.
                            </div>
                            <div style={{ fontSize: '0.7rem', color: 'var(--text-secondary)', marginTop: '0.25rem' }}>14:02 PM</div>
                        </div>

                        {status === 'PAID' && (
                            <div style={{ alignSelf: 'flex-end', maxWidth: '80%' }}>
                                <div style={{ backgroundColor: 'rgba(88, 166, 255, 0.1)', border: '1px solid var(--primary-color)', padding: '0.75rem', borderRadius: '8px', borderTopRightRadius: 0, color: 'var(--primary-color)', fontSize: '0.875rem' }}>
                                    I have made the payment. Please verify.
                                </div>
                                <div style={{ fontSize: '0.7rem', color: 'var(--text-secondary)', marginTop: '0.25rem', textAlign: 'right' }}>14:05 PM</div>
                            </div>
                        )}

                        {status === 'COMPLETED' && (
                            <div style={{ alignSelf: 'center', fontSize: '0.75rem', color: 'var(--accent-buy)', backgroundColor: 'rgba(35, 134, 54, 0.1)', padding: '0.25rem 0.75rem', borderRadius: '9999px', border: '1px solid var(--accent-buy)' }}>
                                Trade Completed. Escrow released.
                            </div>
                        )}
                    </div>

                    <div style={{ padding: '1rem', borderTop: '1px solid var(--border-color)' }}>
                        <div className="flex gap-2">
                            <input type="text" className="input" style={{ flex: 1 }} placeholder="Type a message..." disabled={status === 'COMPLETED'} />
                            <button className="btn btn-outline" disabled={status === 'COMPLETED'}>Send</button>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    );
}
