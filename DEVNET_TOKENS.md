#  Solana Devnet Token Guide

## Quick Setup

### 1. Install Solana CLI
```bash
curl -sSfL https://release.solana.com/v1.18.4/install | sh
export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"
```

### 2. Configure for Devnet
```bash
solana config set --url devnet
```

### 3. Generate Test Wallet
```bash
solana-keygen new --no-bip39-passphrase --outfile ~/devnet-wallet.json
```

### 4. Get Wallet Address
```bash
solana-keygen pubkey ~/devnet-wallet.json
```

### 5. Request Devnet SOL
```bash
# Method 1: CLI (2 SOL per request)
solana airdrop 2 <YOUR_WALLET_ADDRESS> --url devnet

# Method 2: Web Faucet
# Visit: https://faucet.solana.com/
# Enter your wallet address and request tokens
```

### 6. Check Balance
```bash
solana balance <YOUR_WALLET_ADDRESS> --url devnet
```

## Example Usage

```bash
# Generate wallet
WALLET=$(solana-keygen pubkey ~/devnet-wallet.json)
echo "Wallet: $WALLET"

# Request tokens
solana airdrop 2 $WALLET --url devnet

# Check balance
solana balance $WALLET --url devnet
```

## For Testing FidduPay

1. **Customer Wallet**: Use the generated wallet address as customer
2. **Merchant Wallet**: Configure in FidduPay dashboard  
3. **Network**: All transactions use Solana Devnet
4. **Tokens**: Free devnet SOL for testing

## Rate Limits

- **CLI Airdrop**: 2 SOL per request, ~5 requests per hour
- **Web Faucet**: 1-2 SOL per request, rate limited by IP
- **Alternative**: Ask community on Discord for devnet tokens
