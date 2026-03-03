// Required for VSCode TS Server Indexing
import { ArrowDownToLine, ArrowUpFromLine, ArrowRightLeft, Wallet, ShieldCheck } from 'lucide-react';

export default function WalletDashboard() {
    return (
        <div className="fade-in">
            <h1 style={{ fontSize: '1.75rem', marginBottom: '1.5rem' }}>P2P Funding Wallet</h1>

            <div className="grid" style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(300px, 1fr))', gap: '1.5rem', marginBottom: '2rem' }}>
                <div className="panel" style={{ background: 'linear-gradient(135deg, var(--panel-bg), #1a202c)' }}>
                    <div className="flex items-center gap-2 mb-2" style={{ color: 'var(--text-secondary)' }}>
                        <Wallet size={18} /> Estimated Balance
                    </div>
                    <div style={{ fontSize: '2.5rem', fontWeight: 700, marginBottom: '0.5rem' }}>
                        $4,520.00
                    </div>
                    <div className="flex gap-4">
                        <button className="btn btn-buy" style={{ flex: 1 }}><ArrowDownToLine size={16} /> Deposit</button>
                        <button className="btn btn-outline" style={{ flex: 1 }}><ArrowUpFromLine size={16} /> Withdraw</button>
                        <button className="btn btn-outline" style={{ flex: 1 }}><ArrowRightLeft size={16} /> Transfer</button>
                    </div>
                </div>

                <div className="panel flex-col justify-center">
                    <div className="flex items-center gap-3 mb-4">
                        <div style={{ padding: '0.75rem', backgroundColor: 'rgba(88, 166, 255, 0.1)', borderRadius: '50%', color: 'var(--primary-color)' }}>
                            <ShieldCheck size={24} />
                        </div>
                        <div>
                            <h3 style={{ fontSize: '1.125rem' }}>Custodial Wallet Standard</h3>
                            <p style={{ color: 'var(--text-secondary)', fontSize: '0.875rem' }}>Private keys are managed by FidduPay for strict escrow integrity.</p>
                        </div>
                    </div>
                    <div style={{ backgroundColor: 'var(--bg-color)', padding: '0.75rem', borderRadius: '6px', fontSize: '0.875rem' }}>
                        To trade on P2P, you must transfer funds from your Merchant Balance into this Funding Wallet.
                    </div>
                </div>
            </div>

            <div className="panel" style={{ padding: 0, overflow: 'hidden' }}>
                <table style={{ width: '100%', borderCollapse: 'collapse', textAlign: 'left' }}>
                    <thead style={{ backgroundColor: 'var(--bg-color)', borderBottom: '1px solid var(--border-color)' }}>
                        <tr>
                            <th style={{ padding: '1rem', color: 'var(--text-secondary)', fontWeight: 500, fontSize: '0.875rem' }}>Asset</th>
                            <th style={{ padding: '1rem', color: 'var(--text-secondary)', fontWeight: 500, fontSize: '0.875rem' }}>Total Balance</th>
                            <th style={{ padding: '1rem', color: 'var(--text-secondary)', fontWeight: 500, fontSize: '0.875rem' }}>Available</th>
                            <th style={{ padding: '1rem', color: 'var(--text-secondary)', fontWeight: 500, fontSize: '0.875rem' }}>In Order (Locked)</th>
                            <th style={{ padding: '1rem', color: 'var(--text-secondary)', fontWeight: 500, fontSize: '0.875rem', textAlign: 'right' }}>Actions</th>
                        </tr>
                    </thead>
                    <tbody>
                        <tr style={{ borderBottom: '1px solid var(--border-color)' }}>
                            <td style={{ padding: '1.25rem 1rem' }}>
                                <div className="flex items-center gap-3">
                                    <div style={{ width: '32px', height: '32px', borderRadius: '50%', backgroundColor: '#26a17b', display: 'flex', alignItems: 'center', justifyContent: 'center', color: '#fff', fontWeight: 'bold' }}>T</div>
                                    <span style={{ fontWeight: 600 }}>USDT</span>
                                </div>
                            </td>
                            <td style={{ padding: '1.25rem 1rem', fontWeight: 600 }}>1,500.00</td>
                            <td style={{ padding: '1.25rem 1rem' }}>1,000.00</td>
                            <td style={{ padding: '1.25rem 1rem', color: 'var(--text-secondary)' }}>500.00</td>
                            <td style={{ padding: '1.25rem 1rem', textAlign: 'right' }}>
                                <button className="btn btn-outline" style={{ fontSize: '0.75rem', padding: '0.25rem 0.5rem' }}>Trade</button>
                            </td>
                        </tr>
                        <tr>
                            <td style={{ padding: '1.25rem 1rem' }}>
                                <div className="flex items-center gap-3">
                                    <div style={{ width: '32px', height: '32px', borderRadius: '50%', backgroundColor: '#f7931a', display: 'flex', alignItems: 'center', justifyContent: 'center', color: '#fff', fontWeight: 'bold' }}>₿</div>
                                    <span style={{ fontWeight: 600 }}>BTC</span>
                                </div>
                            </td>
                            <td style={{ padding: '1.25rem 1rem', fontWeight: 600 }}>0.045</td>
                            <td style={{ padding: '1.25rem 1rem' }}>0.045</td>
                            <td style={{ padding: '1.25rem 1rem', color: 'var(--text-secondary)' }}>0.000</td>
                            <td style={{ padding: '1.25rem 1rem', textAlign: 'right' }}>
                                <button className="btn btn-outline" style={{ fontSize: '0.75rem', padding: '0.25rem 0.5rem' }}>Trade</button>
                            </td>
                        </tr>
                    </tbody>
                </table>
            </div>
        </div>
    );
}
