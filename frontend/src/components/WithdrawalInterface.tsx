// Enhanced Withdrawal Interface Component
// Handles gas validation and withdrawal processing

import { useState, useEffect } from 'react';
import { merchantAPI, withdrawalAPI } from '@/services/apiService';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Badge } from '@/components/ui/badge';
import { ArrowDownUp, Fuel, AlertTriangle, CheckCircle, Clock } from 'lucide-react';

interface Balance {
  crypto_type: string;
  available_balance: string;
  balance_usd: string;
}

interface GasValidation {
  status: 'sufficient' | 'insufficient_native' | 'insufficient_gas';
  can_withdraw: boolean;
  message: string;
  gas_required?: string;
  gas_available?: string;
  gas_shortfall?: string;
  native_currency?: string;
  network?: string;
}

const CRYPTO_OPTIONS = [
  { value: 'SOL', label: 'SOL', network: 'Solana' },
  { value: 'USDT_SPL', label: 'USDT (Solana)', network: 'Solana' },
  { value: 'ETH', label: 'ETH', network: 'Ethereum' },
  { value: 'USDT_ERC20', label: 'USDT (Ethereum)', network: 'Ethereum' },
  { value: 'BNB', label: 'BNB', network: 'BSC' },
  { value: 'USDT_BEP20', label: 'USDT (BSC)', network: 'BSC' },
  { value: 'MATIC', label: 'MATIC', network: 'Polygon' },
  { value: 'USDT_POLYGON', label: 'USDT (Polygon)', network: 'Polygon' },
  { value: 'ARB', label: 'ARB', network: 'Arbitrum' },
  { value: 'USDT_ARBITRUM', label: 'USDT (Arbitrum)', network: 'Arbitrum' },
];

export default function WithdrawalInterface() {
  const [balances, setBalances] = useState<Balance[]>([]);
  const [selectedCrypto, setSelectedCrypto] = useState<string>('');
  const [amount, setAmount] = useState<string>('');
  const [toAddress, setToAddress] = useState<string>('');
  const [password, setPassword] = useState<string>('');
  const [gasValidation, setGasValidation] = useState<GasValidation | null>(null);
  const [loading, setLoading] = useState(false);
  const [processing, setProcessing] = useState(false);
  const [error, setError] = useState<string>('');
  const [success, setSuccess] = useState<string>('');

  useEffect(() => {
    loadBalances();
  }, []);

  useEffect(() => {
    if (selectedCrypto && amount && parseFloat(amount) > 0) {
      validateGas();
    } else {
      setGasValidation(null);
    }
  }, [selectedCrypto, amount]);

  const loadBalances = async () => {
    try {
      const response = await merchantAPI.getBalance()
      setBalances(response.data.balances || [])
    } catch (err) {
      setError('Failed to load balances');
    }
  };

  const validateGas = async () => {
    try {
      const response = await withdrawalAPI.validateGas(selectedCrypto, parseFloat(amount))
      setGasValidation(response.data)
    } catch (err) {
      console.error('Gas validation failed:', err);
    }
  };

  const handleWithdrawal = async () => {
    if (!selectedCrypto || !amount || !toAddress) {
      setError('Please fill all required fields');
      return;
    }

    if (!gasValidation?.can_withdraw) {
      setError('Cannot withdraw - insufficient gas or balance');
      return;
    }

    setLoading(true);
    setError('');

    try {
      // Create withdrawal
      const createData = await withdrawalAPI.create({
        crypto_type: selectedCrypto,
        amount: parseFloat(amount),
        to_address: toAddress,
        description: 'Manual withdrawal'
      })
      
      // Process withdrawal
      setProcessing(true)
      const processData = await withdrawalAPI.process(createData.data.id, password)
      
      setSuccess(`Withdrawal processed successfully! Transaction: ${processData.data.withdrawal.transaction_hash}`)
      setAmount('')
      setToAddress('')
      setPassword('')
      loadBalances()
    } catch (err) {
      setError('Network error occurred')
    } finally {
      setLoading(false)
      setProcessing(false)
    }
  };

  const getBalance = (cryptoType: string) => {
    return balances.find(b => b.crypto_type === cryptoType);
  };

  const selectedCryptoInfo = CRYPTO_OPTIONS.find(c => c.value === selectedCrypto);
  const balance = selectedCrypto ? getBalance(selectedCrypto) : null;

  return (
    <div className="max-w-2xl mx-auto p-6">
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <ArrowDownUp className="h-5 w-5" />
            Withdraw Funds
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-6">
          {/* Cryptocurrency Selection */}
          <div>
            <Label htmlFor="crypto">Cryptocurrency</Label>
            <Select value={selectedCrypto} onValueChange={setSelectedCrypto}>
              <SelectTrigger>
                <SelectValue placeholder="Select cryptocurrency" />
              </SelectTrigger>
              <SelectContent>
                {CRYPTO_OPTIONS.map((crypto) => {
                  const bal = getBalance(crypto.value);
                  return (
                    <SelectItem key={crypto.value} value={crypto.value}>
                      <div className="flex items-center justify-between w-full">
                        <span>{crypto.label}</span>
                        <span className="text-sm text-gray-500 ml-2">
                          {bal ? `${parseFloat(bal.available_balance).toFixed(4)}` : '0.0000'}
                        </span>
                      </div>
                    </SelectItem>
                  );
                })}
              </SelectContent>
            </Select>
            {balance && (
              <div className="mt-1 text-sm text-gray-600">
                Available: {parseFloat(balance.available_balance).toFixed(6)} {selectedCryptoInfo?.label} 
                (${parseFloat(balance.balance_usd).toFixed(2)})
              </div>
            )}
          </div>

          {/* Amount */}
          <div>
            <Label htmlFor="amount">Amount</Label>
            <Input
              id="amount"
              type="number"
              step="0.000001"
              value={amount}
              onChange={(e: React.ChangeEvent<HTMLInputElement>) => setAmount(e.target.value)}
              placeholder="0.000000"
            />
            {balance && amount && (
              <div className="mt-1 text-sm text-gray-600">
                ≈ ${(parseFloat(amount) * parseFloat(balance.balance_usd) / parseFloat(balance.available_balance)).toFixed(2)} USD
              </div>
            )}
          </div>

          {/* To Address */}
          <div>
            <Label htmlFor="to-address">Recipient Address</Label>
            <Input
              id="to-address"
              value={toAddress}
              onChange={(e: React.ChangeEvent<HTMLInputElement>) => setToAddress(e.target.value)}
              placeholder={selectedCryptoInfo?.network === 'Solana' ? 'Solana address' : '0x...'}
            />
          </div>

          {/* Gas Validation */}
          {gasValidation && (
            <Alert className={gasValidation.can_withdraw ? '' : 'border-orange-200 bg-orange-50'}>
              <Fuel className="h-4 w-4" />
              <AlertDescription>
                <div className="flex items-center justify-between">
                  <span>{gasValidation.message}</span>
                  <Badge variant={gasValidation.can_withdraw ? 'default' : 'secondary'}>
                    {gasValidation.status === 'sufficient' ? 'Ready' : 'Gas Needed'}
                  </Badge>
                </div>
                {gasValidation.status === 'insufficient_gas' && (
                  <div className="mt-2 text-sm">
                    Need {gasValidation.gas_shortfall} {gasValidation.native_currency} for gas fees on {gasValidation.network}
                  </div>
                )}
              </AlertDescription>
            </Alert>
          )}

          {/* Password for Processing */}
          {gasValidation?.can_withdraw && (
            <div>
              <Label htmlFor="password">Wallet Password</Label>
              <Input
                id="password"
                type="password"
                value={password}
                onChange={(e: React.ChangeEvent<HTMLInputElement>) => setPassword(e.target.value)}
                placeholder="Enter your wallet encryption password"
              />
            </div>
          )}

          {/* Submit Button */}
          <Button
            onClick={handleWithdrawal}
            disabled={loading || processing || !gasValidation?.can_withdraw || !password}
            className="w-full"
          >
            {processing ? (
              <>
                <Clock className="h-4 w-4 mr-2 animate-spin" />
                Processing Withdrawal...
              </>
            ) : loading ? (
              'Creating Withdrawal...'
            ) : (
              'Withdraw Funds'
            )}
          </Button>

          {error && (
            <Alert variant="destructive">
              <AlertTriangle className="h-4 w-4" />
              <AlertDescription>{error}</AlertDescription>
            </Alert>
          )}

          {success && (
            <Alert>
              <CheckCircle className="h-4 w-4" />
              <AlertDescription>{success}</AlertDescription>
            </Alert>
          )}

          {/* Gas Fee Information */}
          <div className="text-xs text-gray-500 space-y-1">
            <div className="font-medium">Gas Fee Information:</div>
            <div>• Native currencies (SOL, ETH, BNB, MATIC, ARB): Gas auto-deducted from withdrawal</div>
            <div>• USDT tokens: Requires separate gas deposit in network's native currency</div>
            <div>• Gas fees vary based on network congestion</div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
