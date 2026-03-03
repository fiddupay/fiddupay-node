import { Link, useLocation } from 'react-router-dom';
import { useAppStore } from '../store';

export const Navbar = () => {
    const location = useLocation();
    const { user, isAuthenticated, logout } = useAppStore();

    return (
        <header className="header" style={{ position: 'sticky', top: 0, zIndex: 50, backdropFilter: 'blur(12px)', borderBottom: '1px solid var(--border-color)', backgroundColor: 'rgba(10, 10, 10, 0.8)' }}>
            <div className="container flex justify-between items-center w-full">
                <div className="flex items-center gap-6">
                    <Link to="/" className="header-logo" style={{ fontSize: '1.25rem', fontWeight: 700, color: 'var(--primary-color)' }}>
                        FidduPay P2P
                    </Link>
                    <nav className="nav-links" style={{ display: 'flex', gap: '1.5rem' }}>
                        <Link to="/" className={`nav-link ${location.pathname === '/' ? 'active' : ''}`} style={{ color: location.pathname === '/' ? 'var(--primary-color)' : 'var(--text-secondary)', textDecoration: 'none' }}>Marketplace</Link>
                        <Link to="/orders" className={`nav-link ${location.pathname.startsWith('/orders') ? 'active' : ''}`} style={{ color: location.pathname.startsWith('/orders') ? 'var(--primary-color)' : 'var(--text-secondary)', textDecoration: 'none' }}>Orders</Link>
                        <Link to="/wallet" className={`nav-link ${location.pathname === '/wallet' ? 'active' : ''}`} style={{ color: location.pathname === '/wallet' ? 'var(--primary-color)' : 'var(--text-secondary)', textDecoration: 'none' }}>Wallet</Link>
                    </nav>
                </div>
                <div className="flex gap-4 items-center">
                    {isAuthenticated ? (
                        <>
                            <span className="text-sm" style={{ color: 'var(--text-secondary)', fontSize: '0.875rem' }}>Welcome, {user?.nickname}</span>
                            <button onClick={logout} className="btn btn-outline" style={{ padding: '0.25rem 0.75rem', fontSize: '0.875rem' }}>Log Out</button>
                        </>
                    ) : (
                        <button className="btn btn-buy" style={{ padding: '0.25rem 1rem', fontSize: '0.875rem' }}>Login</button>
                    )}
                </div>
            </div>
        </header>
    );
};
