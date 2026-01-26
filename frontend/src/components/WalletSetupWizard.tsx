// Wallet Setup Wizard Component
// Guides merchants through 3-mode wallet configuration

import { useState, useEffect } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Shield, Key, Wallet, AlertTriangle, CheckCircle } from 'lucide-react';

interface WalletConfig {
  network: string;
  wallet_mode: 'address_only' | 'gateway_generated' | 'merchant_provided';
  address: string;
  has_private_key: boolean;
}

interface NetworkConfig {
  name: string;
  id: string;
  native_currency: string;
  description: string;
}

const NETWORKS: NetworkConfig[] = [
  { name: 'Ethereum', id: 'ethereum', native_currency: 'ETH', description: 'Ethereum mainnet' },
  { name: 'BSC', id: 'bsc', native_currency: 'BNB', description: 'Binance Smart Chain' },
  { name: 'Polygon', id: 'polygon', native_currency: 'MATIC', description: 'Polygon network' },
  { name: 'Arbitrum', id: 'arbitrum', native_currency: 'ARB', description: 'Arbitrum One' },
  { name: 'Solana', id: 'solana', native_currency: 'SOL', description: 'Solana mainnet' },
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
  const [privateKey, setPrivateKey] = useState('');
  const [password, setPassword] = useState('');
  const [confirmPassword, setConfirmPassword] = useState('');

  useEffect(() => {
    loadWalletConfigs();
  }, []);

  const loadWalletConfigs = async () => {
    try {
      const response = await fetch('/api/v1/wallets', {
        headers: { 'Authorization': `Bearer ${localStorage.getItem('api_key')}` }
      });
      const data = await response.json();
      setWallets(data.wallets || []);
    } catch (err) {
      setError('Failed to load wallet configurations');
    }
  };

  const handleAddressOnlySetup = async () => {
    if (!address) {
      setError('Please enter a wallet address');
      return;
    }

    setLoading(true);
    setError('');

    try {
      const response = await fetch('/api/v1/wallets/configure-address', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${localStorage.getItem('api_key')}`
        },
        body: JSON.stringify({
          network: selectedNetwork,
          address: address
        })
      });

      const data = await response.json();
      
      if (response.ok) {
        setSuccess(`Address-only wallet configured for ${selectedNetwork}`);
        setAddress('');
        loadWalletConfigs();
      } else {
        setError(data.error || 'Failed to configure wallet');
      }
    } catch (err) {
      setError('Network error occurred');
    } finally {
      setLoading(false);
    }
  };

  const handleGenerateWallet = async () => {
    if (!password || password !== confirmPassword) {
      setError('Please enter matching passwords');
      return;
    }

    setLoading(true);
    setError('');

    try {
      const response = await fetch('/api/v1/wallets/generate', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${localStorage.getItem('api_key')}`
        },
        body: JSON.stringify({
          network: selectedNetwork,
          encryption_password: password
        })
      });

      const data = await response.json();
      
      if (response.ok) {
        setSuccess(`Wallet generated for ${selectedNetwork}. Private key: ${data.wallet.private_key}`);
        setPassword('');
        setConfirmPassword('');
        loadWalletConfigs();
      } else {
        setError(data.error || 'Failed to generate wallet');
      }
    } catch (err) {
      setError('Network error occurred');
    } finally {
      setLoading(false);
    }
  };

  const handleImportWallet = async () => {
    if (!privateKey || !password || password !== confirmPassword) {
      setError('Please fill all fields with matching passwords');
      return;
    }

    setLoading(true);
    setError('');

    try {
      const response = await fetch('/api/v1/wallets/import', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${localStorage.getItem('api_key')}`
        },
        body: JSON.stringify({
          network: selectedNetwork,
          private_key: privateKey,
          encryption_password: password
        })
      });

      const data = await response.json();
      
      if (response.ok) {
        setSuccess(`Private key imported for ${selectedNetwork}`);
        setPrivateKey('');
        setPassword('');
        setConfirmPassword('');
        loadWalletConfigs();
      } else {
        setError(data.error || 'Failed to import wallet');
      }
    } catch (err) {
      setError('Network error occurred');
    } finally {
      setLoading(false);
    }
  };

  const getWalletForNetwork = (networkId: string) => {
    return wallets.find(w => w.network === networkId);
  };

  return (
    <div className="max-w-4xl mx-auto p-6 space-y-6">
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Wallet className="h-5 w-5" />
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
                      className={`p-3 border rounded-lg cursor-pointer transition-colors ${
                        selectedNetwork === network.id
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
                            <CheckCircle className="h-4 w-4" />
                            <span className="text-xs">
                              {wallet.wallet_mode === 'address_only' ? 'Address Only' :
                               wallet.wallet_mode === 'gateway_generated' ? 'Generated' : 'Imported'}
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
                <TabsList className="grid w-full grid-cols-3">
                  <TabsTrigger value="address_only" className="text-xs">
                    <Shield className="h-3 w-3 mr-1" />
                    Address Only
                  </TabsTrigger>
                  <TabsTrigger value="gateway_generated" className="text-xs">
                    <Key className="h-3 w-3 mr-1" />
                    Generate
                  </TabsTrigger>
                  <TabsTrigger value="merchant_provided" className="text-xs">
                    <Wallet className="h-3 w-3 mr-1" />
                    Import
                  </TabsTrigger>
                </TabsList>

                <TabsContent value="address_only" className="space-y-4">
                  <Alert>
                    <Shield className="h-4 w-4" />
                    <AlertDescription>
                      Maximum security - you control private keys externally. No withdrawal capability through FidduPay.
                    </AlertDescription>
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
                    <Key className="h-4 w-4" />
                    <AlertDescription>
                      FidduPay generates encrypted keys. Withdrawal capability enabled. You can export keys anytime.
                    </AlertDescription>
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

                <TabsContent value="merchant_provided" className="space-y-4">
                  <Alert>
                    <AlertTriangle className="h-4 w-4" />
                    <AlertDescription>
                      Import your existing private key. Withdrawal capability enabled. Key will be encrypted and stored securely.
                    </AlertDescription>
                  </Alert>
                  <div>
                    <Label htmlFor="private-key">Private Key</Label>
                    <Input
                      id="private-key"
                      type="password"
                      value={privateKey}
                      onChange={(e: React.ChangeEvent<HTMLInputElement>) => setPrivateKey(e.target.value)}
                      placeholder={selectedNetwork === 'solana' ? 'Base58 private key' : '0x... or hex private key'}
                    />
                  </div>
                  <div>
                    <Label htmlFor="import-password">Encryption Password</Label>
                    <Input
                      id="import-password"
                      type="password"
                      value={password}
                      onChange={(e: React.ChangeEvent<HTMLInputElement>) => setPassword(e.target.value)}
                      placeholder="Password to encrypt your key"
                    />
                  </div>
                  <div>
                    <Label htmlFor="import-confirm">Confirm Password</Label>
                    <Input
                      id="import-confirm"
                      type="password"
                      value={confirmPassword}
                      onChange={(e: React.ChangeEvent<HTMLInputElement>) => setConfirmPassword(e.target.value)}
                      placeholder="Confirm password"
                    />
                  </div>
                  <Button onClick={handleImportWallet} disabled={loading} className="w-full">
                    Import Private Key
                  </Button>
                </TabsContent>
              </Tabs>
            </div>
          </div>

          {error && (
            <Alert className="mt-4" variant="destructive">
              <AlertTriangle className="h-4 w-4" />
              <AlertDescription>{error}</AlertDescription>
            </Alert>
          )}

          {success && (
            <Alert className="mt-4">
              <CheckCircle className="h-4 w-4" />
              <AlertDescription>{success}</AlertDescription>
            </Alert>
          )}
        </CardContent>
      </Card>
    </div>
  );
}
