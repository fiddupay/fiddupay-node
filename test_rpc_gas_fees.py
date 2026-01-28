#!/usr/bin/env python3
"""
Test 2026 RPC endpoints for gas fee estimation
"""
import requests
import json

# Test RPC endpoints - Working 2026 endpoints
ENDPOINTS = {
    "ethereum": "https://eth.llamarpc.com",
    "ethereum_alt": "https://rpc.flashbots.net",
    "bsc": "https://bsc-dataseed.binance.org",
    "polygon": "https://polygon-rpc.com",
    "arbitrum": "https://arb1.arbitrum.io/rpc",
    "solana": "https://api.mainnet-beta.solana.com"
}

def test_ethereum_gas():
    """Test Ethereum eth_feeHistory"""
    payload = {
        "jsonrpc": "2.0",
        "method": "eth_feeHistory",
        "params": ["0x4", "latest", [10.0, 25.0, 50.0]],
        "id": 1
    }
    
    for name, url in [("ethereum", ENDPOINTS["ethereum"]), ("ethereum_alt", ENDPOINTS["ethereum_alt"])]:
        try:
            response = requests.post(url, json=payload, timeout=10)
            data = response.json()
            
            if "result" in data:
                result = data["result"]
                base_fee = result.get("baseFeePerGas", [])[-1] if result.get("baseFeePerGas") else "0x0"
                reward = result.get("reward", [[]])[-1][1] if result.get("reward") else "0x0"
                
                base_fee_gwei = int(base_fee, 16) / 1e9
                priority_fee_gwei = int(reward, 16) / 1e9
                
                print(f" {name.upper()}: Base fee: {base_fee_gwei:.2f} gwei, Priority: {priority_fee_gwei:.2f} gwei")
            else:
                print(f" {name.upper()}: {data.get('error', 'No result')}")
        except Exception as e:
            print(f" {name.upper()}: {str(e)}")

def test_bsc_gas():
    """Test BSC eth_gasPrice"""
    payload = {
        "jsonrpc": "2.0",
        "method": "eth_gasPrice",
        "params": [],
        "id": 1
    }
    
    try:
        response = requests.post(ENDPOINTS["bsc"], json=payload, timeout=10)
        data = response.json()
        
        if "result" in data:
            gas_price_gwei = int(data["result"], 16) / 1e9
            print(f" BSC: Gas price: {gas_price_gwei:.2f} gwei")
        else:
            print(f" BSC: {data.get('error', 'No result')}")
    except Exception as e:
        print(f" BSC: {str(e)}")

def test_polygon_gas():
    """Test Polygon eth_feeHistory"""
    payload = {
        "jsonrpc": "2.0",
        "method": "eth_feeHistory",
        "params": ["0x4", "latest", [25.0]],
        "id": 1
    }
    
    try:
        response = requests.post(ENDPOINTS["polygon"], json=payload, timeout=10)
        data = response.json()
        
        if "result" in data:
            result = data["result"]
            base_fee = result.get("baseFeePerGas", [])[-1] if result.get("baseFeePerGas") else "0x0"
            reward = result.get("reward", [[]])[-1][0] if result.get("reward") else "0x0"
            
            base_fee_gwei = int(base_fee, 16) / 1e9
            priority_fee_gwei = int(reward, 16) / 1e9
            
            print(f" POLYGON: Base fee: {base_fee_gwei:.2f} gwei, Priority: {priority_fee_gwei:.2f} gwei")
        else:
            print(f" POLYGON: {data.get('error', 'No result')}")
    except Exception as e:
        print(f" POLYGON: {str(e)}")

def test_arbitrum_gas():
    """Test Arbitrum eth_gasPrice"""
    payload = {
        "jsonrpc": "2.0",
        "method": "eth_gasPrice",
        "params": [],
        "id": 1
    }
    
    try:
        response = requests.post(ENDPOINTS["arbitrum"], json=payload, timeout=10)
        data = response.json()
        
        if "result" in data:
            gas_price_gwei = int(data["result"], 16) / 1e9
            print(f" ARBITRUM: Gas price: {gas_price_gwei:.2f} gwei")
        else:
            print(f" ARBITRUM: {data.get('error', 'No result')}")
    except Exception as e:
        print(f" ARBITRUM: {str(e)}")

def test_solana_gas():
    """Test Solana getRecentPrioritizationFees"""
    payload = {
        "jsonrpc": "2.0",
        "method": "getRecentPrioritizationFees",
        "params": [[]],
        "id": 1
    }
    
    try:
        response = requests.post(ENDPOINTS["solana"], json=payload, timeout=10)
        data = response.json()
        
        if "result" in data:
            fees = [item.get("prioritizationFee", 0) for item in data["result"]]
            fees.sort()
            median_fee = fees[len(fees)//2] if fees else 0
            base_fee = 5000  # lamports
            total_fee = base_fee + median_fee
            
            print(f" SOLANA: Base: {base_fee} lamports, Priority: {median_fee} lamports, Total: {total_fee} lamports")
        else:
            print(f" SOLANA: {data.get('error', 'No result')}")
    except Exception as e:
        print(f" SOLANA: {str(e)}")

if __name__ == "__main__":
    print(" Testing 2026 RPC endpoints for gas fee estimation...\n")
    
    test_ethereum_gas()
    test_bsc_gas()
    test_polygon_gas()
    test_arbitrum_gas()
    test_solana_gas()
    
    print("\n Test completed!")
