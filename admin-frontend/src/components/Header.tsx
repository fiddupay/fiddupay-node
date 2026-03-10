import React from 'react';

const Header: React.FC = () => {
    return (
        <header className="h-16 flex items-center justify-between px-8 bg-surface border-b border-gray-200">
            <div className="flex items-center gap-4">
                <h2 className="text-sm font-medium text-gray-500">Super Admin Portal</h2>
            </div>
            <div>
                <button className="text-sm text-primary-600 font-medium hover:text-primary-700">
                    Logout
                </button>
            </div>
        </header>
    );
};

export default Header;
