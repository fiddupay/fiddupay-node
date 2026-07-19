import React from 'react';
import { Outlet } from 'react-router-dom';
import Sidebar from './Sidebar';
import Header from './Header';

interface LayoutProps {
    onLogout: () => void;
    user: any;
}

const Layout: React.FC<LayoutProps> = ({ onLogout, user }) => {
    return (
        <div className="flex h-screen bg-[#0b0f19] font-sans">
            <Sidebar user={user} />
            <div className="flex flex-col flex-1 overflow-hidden">
                <Header onLogout={onLogout} />
                <main className="flex-1 overflow-y-auto p-8">
                    <Outlet />
                </main>
            </div>
        </div>
    );
};

export default Layout;
