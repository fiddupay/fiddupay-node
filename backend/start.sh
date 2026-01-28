#!/bin/bash

# Load environment variables from .env file
set -a
source /home/vibes/crypto-payment-gateway/backend/.env
set +a

# Start the backend
exec /home/vibes/crypto-payment-gateway/backend/target/release/fiddupay
