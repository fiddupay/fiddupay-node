import React, { useState } from 'react';
import { Lock, ShieldAlert, Loader2 } from 'lucide-react';
import { adminAPI } from '../lib/api';

interface LoginPageProps {
    onLoginSuccess: (token: string, user: any) => void;
}

const LoginPage: React.FC<LoginPageProps> = ({ onLoginSuccess }) => {
    const [username, setUsername] = useState('');
    const [password, setPassword] = useState('');
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        setLoading(true);
        setError(null);
        try {
            const res = await adminAPI.login({ username, password });
            if (res.data && res.data.success) {
                onLoginSuccess(res.data.session_token, res.data.user);
            } else {
                setError('Failed to authenticate');
            }
        } catch (err: any) {
            setError(err.response?.data?.error || 'Invalid credentials');
        } finally {
            setLoading(false);
        }
    };

    return (
        <div className="min-h-screen bg-[#0b0f19] flex items-center justify-center p-4 font-sans">
            <div className="w-full max-w-md bg-[#151c2c] rounded-3xl border border-white/5 shadow-2xl p-8 space-y-6">
                <div className="text-center space-y-2">
                    <div className="w-16 h-16 bg-primary-600/10 border border-primary-500/20 rounded-2xl flex items-center justify-center mx-auto text-primary-400">
                        <Lock size={32} />
                    </div>
                    <h1 className="text-2xl font-bold text-slate-100 tracking-wider">FIDDUPAY ADMIN</h1>
                    <p className="text-slate-400 text-sm">Secure Administrative Portal Gateway</p>
                </div>

                {error && (
                    <div className="p-3 bg-rose-500/10 border border-rose-500/20 text-rose-400 rounded-xl text-xs flex items-center gap-2 font-semibold">
                        <ShieldAlert size={16} />
                        {error}
                    </div>
                )}

                <form onSubmit={handleSubmit} className="space-y-4">
                    <div className="space-y-1.5">
                        <label className="text-xs font-bold text-slate-400 uppercase">Username</label>
                        <input
                            type="text"
                            required
                            value={username}
                            onChange={(e) => setUsername(e.target.value)}
                            placeholder="Enter username"
                            className="block w-full px-4 py-3 bg-[#0b0f19] border border-white/5 rounded-xl text-sm focus:outline-none focus:border-primary-500 transition-all text-slate-200"
                        />
                    </div>

                    <div className="space-y-1.5">
                        <label className="text-xs font-bold text-slate-400 uppercase">Password</label>
                        <input
                            type="password"
                            required
                            value={password}
                            onChange={(e) => setPassword(e.target.value)}
                            placeholder="••••••••"
                            className="block w-full px-4 py-3 bg-[#0b0f19] border border-white/5 rounded-xl text-sm focus:outline-none focus:border-primary-500 transition-all text-slate-200"
                        />
                    </div>

                    <button
                        type="submit"
                        disabled={loading}
                        className="w-full py-3 bg-primary-600 hover:bg-primary-500 text-white rounded-xl text-sm font-bold transition-all shadow-glow active:scale-95 flex items-center justify-center gap-2 disabled:opacity-50"
                    >
                        {loading ? (
                            <>
                                <Loader2 size={18} className="animate-spin" />
                                Authenticating...
                            </>
                        ) : (
                            'Sign In to Dashboard'
                        )}
                    </button>
                </form>
            </div>
        </div>
    );
};

export default LoginPage;
