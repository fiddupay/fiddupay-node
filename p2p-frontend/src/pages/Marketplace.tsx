import { useState } from 'react';
import { Search, Filter } from 'lucide-react';

const MOCK_ADS = [
    { id: 1, user: 'CryptoKing99', trades: 450, completion: 98.5, price: 1650.00, fiat: 'NGN', crypto: 'USDT', limitMin: 10000, limitMax: 500000, methods: ['Bank Transfer', 'Opay'] },
    { id: 2, user: 'FastTrader_NG', trades: 1205, completion: 99.1, price: 1652.50, fiat: 'NGN', crypto: 'USDT', limitMin: 50000, limitMax: 2000000, methods: ['Bank Transfer'] },
    { id: 3, user: 'SecurePlug', trades: 89, completion: 95.0, price: 1655.00, fiat: 'NGN', crypto: 'USDT', limitMin: 5000, limitMax: 100000, methods: ['Chipper Cash', 'Bank Transfer'] },
];

export default function Marketplace() {
    const [action, setAction] = useState<'BUY' | 'SELL'>('BUY');
    const [crypto, setCrypto] = useState('USDT');
    const [fiat, setFiat] = useState('NGN');
    const [amount, setAmount] = useState('');

    return (
        <div className="flex-col gap-6 fade-in">
            {/* Header & Filters */}
            <div className="flex justify-between items-center mb-6">
                <div>
                    <h1 style={{ fontSize: '1.75rem', marginBottom: '0.5rem' }}>P2P Trading</h1>
                    <p style={{ color: 'var(--text-secondary)' }}>Buy and sell crypto directly with other users via bank transfer.</p>
                </div>
                <button className="btn btn-outline">
                    <Filter size={16} /> My Ads
                </button>
            </div>

            <div className="panel mb-6 flex items-center gap-4" style={{ padding: '1rem', flexWrap: 'wrap' }}>
                <div className="flex" style={{ backgroundColor: 'var(--bg-color)', borderRadius: '6px', padding: '4px' }}>
                    <button
                        className={`btn ${action === 'BUY' ? 'btn-buy' : ''}`}
                        style={{ color: action === 'BUY' ? '#fff' : 'var(--text-secondary)', backgroundColor: action === 'BUY' ? 'var(--accent-buy)' : 'transparent', boxShadow: 'none' }}
                        onClick={() => setAction('BUY')}
                    >
                        Buy
                    </button>
                    <button
                        className={`btn ${action === 'SELL' ? 'btn-sell' : ''}`}
                        style={{ color: action === 'SELL' ? '#fff' : 'var(--text-secondary)', backgroundColor: action === 'SELL' ? 'var(--accent-sell)' : 'transparent', boxShadow: 'none' }}
                        onClick={() => setAction('SELL')}
                    >
                        Sell
                    </button>
                </div>

                <div className="flex items-center gap-2" style={{ marginLeft: '1rem' }}>
                    <select className="input" value={crypto} onChange={(e) => setCrypto(e.target.value)} style={{ width: '100px', fontWeight: 'bold' }}>
                        <option value="USDT">USDT</option>
                        <option value="BTC">BTC</option>
                        <option value="ETH">ETH</option>
                    </select>
                    <span style={{ color: 'var(--text-secondary)' }}>with</span>
                    <select className="input" value={fiat} onChange={(e) => setFiat(e.target.value)} style={{ width: '100px', fontWeight: 'bold' }}>
                        <option value="NGN">NGN</option>
                        <option value="USD">USD</option>
                        <option value="GBP">GBP</option>
                    </select>
                </div>

                <div className="input-group" style={{ marginLeft: 'auto', flexDirection: 'row', alignItems: 'center' }}>
                    <div className="flex items-center" style={{ position: 'relative' }}>
                        <span style={{ position: 'absolute', left: '10px', color: 'var(--text-secondary)' }}>₦</span>
                        <input
                            type="number"
                            className="input"
                            placeholder="Enter amount..."
                            value={amount}
                            onChange={(e) => setAmount(e.target.value)}
                            style={{ paddingLeft: '25px', width: '200px' }}
                        />
                    </div>
                    <button className="btn btn-outline" style={{ height: '38px' }}><Search size={16} /> Search</button>
                </div>
            </div>

            {/* Ads Table */}
            <div className="panel" style={{ padding: 0, overflow: 'hidden' }}>
                <table style={{ width: '100%', borderCollapse: 'collapse', textAlign: 'left' }}>
                    <thead style={{ backgroundColor: 'var(--bg-color)', borderBottom: '1px solid var(--border-color)' }}>
                        <tr>
                            <th style={{ padding: '1rem', color: 'var(--text-secondary)', fontWeight: 500, fontSize: '0.875rem' }}>Advertiser (Completion rate)</th>
                            <th style={{ padding: '1rem', color: 'var(--text-secondary)', fontWeight: 500, fontSize: '0.875rem' }}>Price</th>
                            <th style={{ padding: '1rem', color: 'var(--text-secondary)', fontWeight: 500, fontSize: '0.875rem' }}>Available / Limits</th>
                            <th style={{ padding: '1rem', color: 'var(--text-secondary)', fontWeight: 500, fontSize: '0.875rem' }}>Payment</th>
                            <th style={{ padding: '1rem', color: 'var(--text-secondary)', fontWeight: 500, fontSize: '0.875rem', textAlign: 'right' }}>Trade <span style={{ color: 'var(--text-primary)' }}>0 Fee</span></th>
                        </tr>
                    </thead>
                    <tbody>
                        {MOCK_ADS.map((ad, i) => (
                            <tr key={ad.id} style={{ borderBottom: i === MOCK_ADS.length - 1 ? 'none' : '1px solid var(--border-color)', transition: 'background-color 0.2s', cursor: 'pointer' }} className="hover:bg-[var(--panel-bg-hover)]">
                                <td style={{ padding: '1.25rem 1rem' }}>
                                    <div className="flex items-center gap-3">
                                        <div style={{ width: '32px', height: '32px', borderRadius: '50%', backgroundColor: 'var(--primary-color)', color: '#fff', display: 'flex', alignItems: 'center', justifyContent: 'center', fontWeight: 'bold' }}>
                                            {ad.user.charAt(0)}
                                        </div>
                                        <div className="flex-col">
                                            <div className="flex items-center gap-2">
                                                <span style={{ fontWeight: 600, color: 'var(--primary-color)' }}>{ad.user}</span>
                                                <span className="badge badge-verification">Verified</span>
                                            </div>
                                            <div style={{ fontSize: '0.75rem', color: 'var(--text-secondary)', marginTop: '0.25rem' }}>
                                                {ad.trades} orders <span style={{ margin: '0 4px' }}>|</span> {ad.completion}% completion
                                            </div>
                                        </div>
                                    </div>
                                </td>
                                <td style={{ padding: '1.25rem 1rem' }}>
                                    <div style={{ fontSize: '1.25rem', fontWeight: 700, color: 'var(--text-primary)' }}>
                                        {ad.price.toLocaleString()} <span style={{ fontSize: '0.875rem', color: 'var(--text-secondary)' }}>{ad.fiat}</span>
                                    </div>
                                </td>
                                <td style={{ padding: '1.25rem 1rem' }}>
                                    <div className="flex-col gap-1" style={{ fontSize: '0.875rem' }}>
                                        <div className="flex justify-between" style={{ maxWidth: '200px' }}>
                                            <span style={{ color: 'var(--text-secondary)' }}>Available</span>
                                            <span style={{ fontWeight: 500 }}>{(ad.limitMax / ad.price).toFixed(2)} {ad.crypto}</span>
                                        </div>
                                        <div className="flex justify-between" style={{ maxWidth: '200px' }}>
                                            <span style={{ color: 'var(--text-secondary)' }}>Limit</span>
                                            <span style={{ fontWeight: 500 }}>₦{ad.limitMin.toLocaleString()} - ₦{ad.limitMax.toLocaleString()}</span>
                                        </div>
                                    </div>
                                </td>
                                <td style={{ padding: '1.25rem 1rem' }}>
                                    <div className="flex gap-2" style={{ flexWrap: 'wrap', maxWidth: '150px' }}>
                                        {ad.methods.map(m => (
                                            <span key={m} className="badge badge-payment" style={{ borderLeft: '3px solid var(--accent-buy)' }}>{m}</span>
                                        ))}
                                    </div>
                                </td>
                                <td style={{ padding: '1.25rem 1rem', textAlign: 'right' }}>
                                    <button className={`btn ${action === 'BUY' ? 'btn-buy' : 'btn-sell'}`} style={{ padding: '0.5rem 1.5rem' }}>
                                        {action} {ad.crypto}
                                    </button>
                                </td>
                            </tr>
                        ))}
                    </tbody>
                </table>
            </div>
        </div>
    );
}
