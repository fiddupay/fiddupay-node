import React from 'react';
import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom';
import Layout from './components/Layout';
import Dashboard from './pages/Dashboard';
import Settings from './pages/Settings';
import MerchantsPage from './pages/MerchantsPage';
import PaymentsPage from './pages/PaymentsPage';
import WithdrawalsPage from './pages/WithdrawalsPage';
import WalletsPage from './pages/WalletsPage';
import SystemPage from './pages/SystemPage';

const App: React.FC = () => {
    return (
        <BrowserRouter>
            <Routes>
                <Route path="/" element={<Layout />}>
                    <Route index element={<Navigate to="/dashboard" replace />} />
                    <Route path="dashboard" element={<Dashboard />} />
                    <Route path="merchants" element={<MerchantsPage />} />
                    <Route path="payments" element={<PaymentsPage />} />
                    <Route path="withdrawals" element={<WithdrawalsPage />} />
                    <Route path="wallets" element={<WalletsPage />} />
                    <Route path="system" element={<SystemPage />} />
                    <Route path="settings" element={<Settings />} />
                </Route>
            </Routes>
        </BrowserRouter>
    );
};

export default App;
