import React from 'react';

const Header: React.FC = () => {
    return (
        <header className="h-16 flex items-center justify-between px-8 bg-[#151c2c] border-b border-white/10">
            <div className="flex items-center gap-4">
                <h2 className="text-sm font-medium text-slate-400">Super Admin Portal</h2>
            </div>
            <div>
                <button className="text-sm text-primary-500 font-semibold hover:text-primary-100 transition-colors">
                    Logout
                </button>
            </div>
        </header>
    );
};

export default Header;
