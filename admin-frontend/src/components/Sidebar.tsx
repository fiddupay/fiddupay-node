import React from 'react';
import { NavLink } from 'react-router-dom';
import { 
    LayoutDashboard, 
    Settings, 
    Users, 
    CreditCard, 
    ArrowUpRight, 
    Wallet, 
    Server,
    ShieldAlert
} from 'lucide-react';
import clsx from 'clsx';

interface SidebarProps {
    user: any;
}

const Sidebar: React.FC<SidebarProps> = ({ user }) => {
    const navItems = [
        { to: '/dashboard', label: 'Dashboard', icon: LayoutDashboard },
        { to: '/merchants', label: 'Merchants', icon: Users },
        { to: '/payments', label: 'Payments', icon: CreditCard },
        { to: '/withdrawals', label: 'Withdrawals', icon: ArrowUpRight },
        { to: '/wallets', label: 'Wallets', icon: Wallet },
        { to: '/system', label: 'System', icon: Server },
        { to: '/security', label: 'Security', icon: ShieldAlert },
        { to: '/settings', label: 'Settings', icon: Settings },
    ];

    return (
        <aside className="w-64 bg-[#0d1321] text-white flex flex-col h-full border-r border-white/5">
            <div className="h-16 flex items-center px-6 border-b border-white/10">
                <h1 className="text-lg font-bold tracking-wider bg-gradient-to-r from-blue-400 to-indigo-300 bg-clip-text text-transparent">FIDDUPAY ADMIN</h1>
            </div>

            <nav className="flex-1 py-6 flex flex-col gap-1 px-4">
                {navItems.map((item) => (
                    <NavLink
                        key={item.to}
                        to={item.to}
                        className={({ isActive }) => clsx(
                            'flex items-center gap-3 px-4 py-3 rounded-xl transition-all font-semibold text-sm',
                            isActive
                                ? 'bg-primary-600 text-white shadow-glow border border-primary-500/30'
                                : 'text-slate-400 hover:bg-white/5 hover:text-white'
                        )}
                    >
                        <item.icon size={18} />
                        {item.label}
                    </NavLink>
                ))}
            </nav>

            <div className="p-4 border-t border-white/10 bg-[#070b14]">
                <div className="flex items-center gap-3">
                    <div className="w-9 h-9 rounded-xl bg-primary-600/20 border border-primary-500/30 flex items-center justify-center text-sm font-bold text-primary-400">
                        {user?.username?.charAt(0).toUpperCase() || 'A'}
                    </div>
                    <div className="flex flex-col">
                        <span className="text-sm font-bold text-slate-200">{user?.username || 'Admin'}</span>
                        <span className="text-xs text-slate-500 font-semibold">{user?.role || 'Super Admin'}</span>
                    </div>
                </div>
            </div>
        </aside>
    );
};

export default Sidebar;
