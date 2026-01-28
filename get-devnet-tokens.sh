#!/bin/bash

# Solana Devnet Token Helper
# Requests devnet SOL tokens for testing

echo " Solana Devnet Token Helper"
echo "============================="

# Check if solana CLI is installed
if ! command -v solana &> /dev/null; then
    echo " Solana CLI not found. Installing..."
    
    # Install Solana CLI
    sh -c "$(curl -sSfL https://release.solana.com/v1.18.4/install)"
    export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"
    
    if ! command -v solana &> /dev/null; then
        echo " Failed to install Solana CLI"
        echo " Manual installation:"
        echo "   curl -sSfL https://release.solana.com/v1.18.4/install | sh"
        echo "   export PATH=\"\$HOME/.local/share/solana/install/active_release/bin:\$PATH\""
        exit 1
    fi
fi

echo " Solana CLI found"

# Set to devnet
echo " Configuring Solana CLI for devnet..."
solana config set --url devnet

# Generate a new keypair for testing
echo " Generating test keypair..."
KEYPAIR_FILE="/tmp/solana_test_keypair.json"
solana-keygen new --no-bip39-passphrase --silent --outfile "$KEYPAIR_FILE"

# Get the public key
PUBLIC_KEY=$(solana-keygen pubkey "$KEYPAIR_FILE")
echo " Test wallet address: $PUBLIC_KEY"

# Request airdrop
echo " Requesting 2 SOL from devnet faucet..."
if solana airdrop 2 "$PUBLIC_KEY" --url devnet; then
    echo " Successfully received 2 SOL on devnet"
    
    # Check balance
    echo " Checking balance..."
    solana balance "$PUBLIC_KEY" --url devnet
    
    echo ""
    echo " Devnet tokens ready for testing!"
    echo " Wallet details:"
    echo "   Address: $PUBLIC_KEY"
    echo "   Network: Solana Devnet"
    echo "   Balance: 2 SOL"
    echo "   Keypair: $KEYPAIR_FILE"
    
else
    echo " Failed to get devnet tokens"
    echo " Alternative methods:"
    echo "   1. Visit: https://faucet.solana.com/"
    echo "   2. Use: solana airdrop 2 $PUBLIC_KEY --url devnet"
    echo "   3. Try again in a few minutes (rate limited)"
fi

echo ""
echo " Usage in tests:"
echo "   Customer wallet: $PUBLIC_KEY"
echo "   Network: devnet"
echo "   RPC URL: https://api.devnet.solana.com"
