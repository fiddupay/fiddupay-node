import React from 'react';

interface HeaderProps {
    onLogout: () => void;
}

const Header: React.FC<HeaderProps> = ({ onLogout }) => {
    return (
        <header className="h-16 flex items-center justify-between px-8 bg-[#151c2c] border-b border-white/10">
            <div className="flex items-center gap-4">
                <h2 className="text-sm font-semibold text-slate-400">Super Admin Portal</h2>
            </div>
            <div>
                <button 
                    onClick={onLogout}
                    className="text-sm text-primary-400 font-bold hover:text-rose-400 transition-colors"
                >
                    Logout
                </button>
            </div>
        </header>
    );
};

export default Header;
