// Wallet Setup Wizard Component
// Guides merchants through 3-mode wallet configuration

import { useState, useEffect } from 'react';
import { walletAPI } from '@/services/apiService';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';

interface WalletConfig {
  network: string;
  wallet_mode: 'address_only' | 'gateway_generated';
  address: string;
}

interface NetworkConfig {
  name: string;
  id: string;
  crypto_type: string; // Backend crypto_type identifier
  native_currency: string;
  description: string;
}

const NETWORKS: NetworkConfig[] = [
  { name: 'Ethereum', id: 'ethereum', crypto_type: 'ETH', native_currency: 'ETH', description: 'Ethereum mainnet' },
  { name: 'BSC', id: 'bsc', crypto_type: 'BNB', native_currency: 'BNB', description: 'Binance Smart Chain' },
  { name: 'Polygon', id: 'polygon', crypto_type: 'MATIC', native_currency: 'MATIC', description: 'Polygon network' },
  { name: 'Arbitrum', id: 'arbitrum', crypto_type: 'ARB', native_currency: 'ARB', description: 'Arbitrum One' },
  { name: 'Solana', id: 'solana', crypto_type: 'SOL', native_currency: 'SOL', description: 'Solana mainnet' },
  { name: 'Bitcoin', id: 'bitcoin', crypto_type: 'BTC', native_currency: 'BTC', description: 'Bitcoin mainnet' },
];

export default function WalletSetupWizard() {
  const [wallets, setWallets] = useState<WalletConfig[]>([]);
  const [selectedNetwork, setSelectedNetwork] = useState<string>('ethereum');
  const [selectedMode, setSelectedMode] = useState<string>('address_only');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string>('');
  const [success, setSuccess] = useState<string>('');

  // Form states
  const [address, setAddress] = useState('');
  const [password, setPassword] = useState('');
  const [confirmPassword, setConfirmPassword] = useState('');

  useEffect(() => {
    loadWalletConfigs();
  }, []);

  const loadWalletConfigs = async () => {
    try {
      const response = await walletAPI.getAll()
      setWallets(response.data.data.wallets || [])
    } catch (err) {
      setError('Failed to load wallet configurations');
    }
  };

  const getSelectedCryptoType = () => {
    const network = NETWORKS.find(n => n.id === selectedNetwork);
    return network?.crypto_type || selectedNetwork.toUpperCase();
  };

  const handleAddressOnlySetup = async () => {
    if (!address) {
      setError('Please enter a wallet address');
      return;
    }

    setLoading(true);
    setError('');

    try {
      await walletAPI.setup({
        crypto_type: getSelectedCryptoType(),
        mode: 'address',
        address: address
      })

      setSuccess(`Address-only wallet configured for ${selectedNetwork}`)
      setAddress('')
      loadWalletConfigs()
    } catch (err) {
      setError('Network error occurred')
    } finally {
      setLoading(false)
    }
  }

  const handleGenerateWallet = async () => {
    if (!password || password !== confirmPassword) {
      setError('Please enter matching passwords');
      return;
    }

    setLoading(true);
    setError('');

    try {
      const data = await walletAPI.setup({
        crypto_type: getSelectedCryptoType(),
        mode: 'generate'
      })

      setSuccess(`Wallet generated for ${selectedNetwork}. Address: ${data.data?.wallet?.address || 'created'}`)
      setPassword('')
      setConfirmPassword('')
      loadWalletConfigs()
    } catch (err) {
      setError('Network error occurred')
    } finally {
      setLoading(false)
    }
  }


  const getWalletForNetwork = (networkId: string) => {
    return wallets.find(w => w.network === networkId);
  };

  return (
    <div className="max-w-4xl mx-auto p-6 space-y-6">
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <i className="fas fa-wallet text-xl"></i>
            Wallet Setup Wizard
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
            {/* Network Selection */}
            <div>
              <Label className="text-base font-medium">Select Network</Label>
              <div className="mt-2 space-y-2">
                {NETWORKS.map((network) => {
                  const wallet = getWalletForNetwork(network.id);
                  return (
                    <div
                      key={network.id}
                      className={`p-3 border rounded-lg cursor-pointer transition-colors ${selectedNetwork === network.id
                        ? 'border-blue-500 bg-blue-50'
                        : 'border-gray-200 hover:border-gray-300'
                        }`}
                      onClick={() => setSelectedNetwork(network.id)}
                    >
                      <div className="flex items-center justify-between">
                        <div>
                          <div className="font-medium">{network.name}</div>
                          <div className="text-sm text-gray-500">{network.description}</div>
                        </div>
                        {wallet && (
                          <div className="flex items-center gap-1 text-green-600">
                            <i className="fas fa-check-circle text-sm"></i>
                            <span className="text-xs">
                              {wallet.wallet_mode === 'address_only' ? 'Address Only' : 'Generated'}
                            </span>
                          </div>
                        )}
                      </div>
                    </div>
                  );
                })}
              </div>
            </div>

            {/* Wallet Mode Configuration */}
            <div>
              <Label className="text-base font-medium">Wallet Configuration</Label>
              <Tabs value={selectedMode} onValueChange={setSelectedMode} className="mt-2">
                <TabsList className="grid w-full grid-cols-2">
                  <TabsTrigger value="address_only" className="text-xs">
                    <i className="fas fa-shield-alt mr-1"></i>
                    Address Only
                  </TabsTrigger>
                  <TabsTrigger value="gateway_generated" className="text-xs">
                    <i className="fas fa-key mr-1"></i>
                    Generate
                  </TabsTrigger>
                </TabsList>

                <TabsContent value="address_only" className="space-y-4">
                  <Alert>
                    <div className="flex items-center gap-3">
                      <i className="fas fa-shield-alt text-lg"></i>
                      <AlertDescription>
                        Maximum security - you control private keys externally. No withdrawal capability through FidduPay.
                      </AlertDescription>
                    </div>
                  </Alert>
                  <div>
                    <Label htmlFor="address">Wallet Address</Label>
                    <Input
                      id="address"
                      value={address}
                      onChange={(e: React.ChangeEvent<HTMLInputElement>) => setAddress(e.target.value)}
                      placeholder={selectedNetwork === 'solana' ? 'Solana address' : '0x...'}
                    />
                  </div>
                  <Button onClick={handleAddressOnlySetup} disabled={loading} className="w-full">
                    Configure Address-Only Wallet
                  </Button>
                </TabsContent>

                <TabsContent value="gateway_generated" className="space-y-4">
                  <Alert>
                    <div className="flex items-center gap-3">
                      <i className="fas fa-key text-lg"></i>
                      <AlertDescription>
                        FidduPay generates encrypted keys. Withdrawal capability enabled.
                      </AlertDescription>
                    </div>
                  </Alert>
                  <div>
                    <Label htmlFor="gen-password">Encryption Password</Label>
                    <Input
                      id="gen-password"
                      type="password"
                      value={password}
                      onChange={(e: React.ChangeEvent<HTMLInputElement>) => setPassword(e.target.value)}
                      placeholder="Strong password for key encryption"
                    />
                  </div>
                  <div>
                    <Label htmlFor="gen-confirm">Confirm Password</Label>
                    <Input
                      id="gen-confirm"
                      type="password"
                      value={confirmPassword}
                      onChange={(e: React.ChangeEvent<HTMLInputElement>) => setConfirmPassword(e.target.value)}
                      placeholder="Confirm password"
                    />
                  </div>
                  <Button onClick={handleGenerateWallet} disabled={loading} className="w-full">
                    Generate New Wallet
                  </Button>
                </TabsContent>

              </Tabs>
            </div>
          </div>

          {error && (
            <Alert className="mt-4" variant="destructive">
              <div className="flex items-center gap-3">
                <i className="fas fa-exclamation-circle text-lg"></i>
                <AlertDescription>{error}</AlertDescription>
              </div>
            </Alert>
          )}

          {success && (
            <Alert className="mt-4">
              <div className="flex items-center gap-3">
                <i className="fas fa-check-circle text-lg"></i>
                <AlertDescription>{success}</AlertDescription>
              </div>
            </Alert>
          )}
        </CardContent>
      </Card>
    </div>
  );
}

