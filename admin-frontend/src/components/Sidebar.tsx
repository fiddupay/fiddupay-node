import React from 'react';
import { NavLink } from 'react-router-dom';
import { LayoutDashboard, Settings } from 'lucide-react';
import clsx from 'clsx';

const Sidebar: React.FC = () => {
    const navItems = [
        { to: '/dashboard', label: 'Dashboard', icon: LayoutDashboard },
        { to: '/settings', label: 'Settings', icon: Settings },
    ];

    return (
        <aside className="w-64 bg-slate-900 text-white flex flex-col h-full">
            <div className="h-16 flex items-center px-6 border-b border-slate-800">
                <h1 className="text-xl font-bold tracking-tight text-white">FidduPay Admin</h1>
            </div>

            <nav className="flex-1 py-4 flex flex-col gap-1 px-3">
                {navItems.map((item) => (
                    <NavLink
                        key={item.to}
                        to={item.to}
                        className={({ isActive }) => clsx(
                            'flex items-center gap-3 px-3 py-2.5 rounded-md transition-colors font-medium text-sm',
                            isActive
                                ? 'bg-primary-600 text-white'
                                : 'text-slate-300 hover:bg-slate-800 hover:text-white'
                        )}
                    >
                        <item.icon size={18} />
                        {item.label}
                    </NavLink>
                ))}
            </nav>

            <div className="p-4 border-t border-slate-800">
                <div className="flex items-center gap-3">
                    <div className="w-8 h-8 rounded-full bg-slate-700 flex items-center justify-center text-sm font-medium">
                        SA
                    </div>
                    <div className="flex flex-col">
                        <span className="text-sm font-medium">Super Admin</span>
                        <span className="text-xs text-slate-400">admin@fiddupay.com</span>
                    </div>
                </div>
            </div>
        </aside>
    );
};

export default Sidebar;
