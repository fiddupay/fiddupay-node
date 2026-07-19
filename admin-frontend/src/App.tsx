import React, { useState, useEffect } from 'react';
import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom';
import Layout from './components/Layout';
import Dashboard from './pages/Dashboard';
import Settings from './pages/Settings';
import MerchantsPage from './pages/MerchantsPage';
import PaymentsPage from './pages/PaymentsPage';
import WithdrawalsPage from './pages/WithdrawalsPage';
import WalletsPage from './pages/WalletsPage';
import SystemPage from './pages/SystemPage';
import SecurityPage from './pages/SecurityPage';
import LoginPage from './pages/LoginPage';

const App: React.FC = () => {
    const [token, setToken] = useState<string | null>(localStorage.getItem('admin_session_token'));
    const [user, setUser] = useState<any>(null);

    useEffect(() => {
        const storedUser = localStorage.getItem('admin_session_user');
        if (storedUser) {
            try {
                setUser(JSON.parse(storedUser));
            } catch (e) {
                console.error(e);
            }
        }
    }, []);

    const handleLoginSuccess = (newToken: string, newUser: any) => {
        localStorage.setItem('admin_session_token', newToken);
        localStorage.setItem('admin_session_user', JSON.stringify(newUser));
        setToken(newToken);
        setUser(newUser);
    };

    const handleLogout = () => {
        localStorage.removeItem('admin_session_token');
        localStorage.removeItem('admin_session_user');
        setToken(null);
        setUser(null);
    };

    if (!token) {
        return <LoginPage onLoginSuccess={handleLoginSuccess} />;
    }

    return (
        <BrowserRouter>
            <Routes>
                <Route path="/" element={<Layout onLogout={handleLogout} user={user} />}>
                    <Route index element={<Navigate to="/dashboard" replace />} />
                    <Route path="dashboard" element={<Dashboard />} />
                    <Route path="merchants" element={<MerchantsPage />} />
                    <Route path="payments" element={<PaymentsPage />} />
                    <Route path="withdrawals" element={<WithdrawalsPage />} />
                    <Route path="wallets" element={<WalletsPage />} />
                    <Route path="system" element={<SystemPage />} />
                    <Route path="security" element={<SecurityPage />} />
                    <Route path="settings" element={<Settings />} />
                    <Route path="*" element={<Navigate to="/dashboard" replace />} />
                </Route>
            </Routes>
        </BrowserRouter>
    );
};

export default App;
