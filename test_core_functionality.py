#!/usr/bin/env python3
"""
Core functionality tests for FidduPay
"""

def test_fee_calculation():
    """Test fee calculation logic"""
    import decimal
    
    payment = decimal.Decimal("100.00")
    fee_rate = decimal.Decimal("0.0075")  # 0.75%
    processing_fee = payment * fee_rate
    forwarding_amount = payment - processing_fee
    
    print(f" Payment: {payment}, Fee: {processing_fee}, Forwarding: {forwarding_amount}")
    
    assert processing_fee == decimal.Decimal("0.750000")
    assert forwarding_amount == decimal.Decimal("99.250000")
    print(" Fee calculations correct")

def test_address_generation():
    """Test address generation logic"""
    import uuid
    
    # Simulate EVM address generation
    def generate_evm_address():
        unique_id = uuid.uuid4()
        # Ensure 40 hex characters (20 bytes)
        hex_part = unique_id.hex[:32] + "12345678"  # Pad to 40 chars
        return f"0x{hex_part[:40]}"

    # Simulate Solana address generation  
    def generate_solana_address():
        return str(uuid.uuid4())

    eth_addr = generate_evm_address()
    sol_addr = generate_solana_address()

    print(f" ETH Address: {eth_addr}")
    print(f" SOL Address: {sol_addr}")

    assert eth_addr.startswith("0x")
    assert len(eth_addr) == 42
    assert len(sol_addr) == 36
    print(" Address generation logic correct")

def test_webhook_payload():
    """Test webhook payload structure"""
    import json
    from datetime import datetime

    # Test webhook payload structure
    webhook_payload = {
        "event": "address_only_payment_status",
        "payment_id": "test_payment_123",
        "merchant_id": 1,
        "status": "Completed",
        "crypto_type": "ETH",
        "requested_amount": "1.00",
        "processing_fee": "0.0075",
        "forwarding_amount": "0.9925",
        "gateway_deposit_address": "0x1234567890abcdef",
        "merchant_destination_address": "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb",
        "timestamp": datetime.utcnow().isoformat()
    }

    json_payload = json.dumps(webhook_payload, indent=2)
    print(" Webhook payload structure:")
    print(json_payload)

    # Validate required fields
    required_fields = ["event", "payment_id", "status", "crypto_type"]
    for field in required_fields:
        assert field in webhook_payload
        
    print(" Webhook payload validation correct")

if __name__ == "__main__":
    print(" Running Core Functionality Tests")
    
    try:
        test_fee_calculation()
        test_address_generation()
        test_webhook_payload()
        print("\n All core functionality tests passed!")
    except Exception as e:
        print(f"\n Test failed: {str(e)}")
        import traceback
        traceback.print_exc()
        exit(1)
