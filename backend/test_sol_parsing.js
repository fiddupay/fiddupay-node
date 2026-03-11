const axios = require('axios');

async function main() {
    const tx_hash = "4jCt18y8JC2UrxkQA6HYPDzKcj7deqNYKBupACSJ2Da8yJNc1vZusT8kYUKogDjmZrHPSMz5NNYAg8zbuwFgYY67";
    const rpc_url = "https://api.mainnet-beta.solana.com"; 
    
    console.log(`Fetching transaction: ${tx_hash}`);
    
    try {
        const response = await axios.post(rpc_url, {
            jsonrpc: "2.0",
            id: 1,
            method: "getTransaction",
            params: [
                tx_hash,
                {
                    encoding: "json",
                    maxSupportedTransactionVersion: 0,
                    commitment: "confirmed"
                }
            ]
        });
        
        const result = response.data.result;
        
        if (!result) {
            console.log("Transaction not found on this RPC node.");
            return;
        }
        
        const meta = result.meta;
        
        console.log("--- Pre Token Balances ---");
        if (meta.preTokenBalances && meta.preTokenBalances.length > 0) {
            meta.preTokenBalances.forEach((v, i) => {
                console.log(`[${i}] Index: ${v.accountIndex} | Mint: ${v.mint} | Owner: ${v.owner} | Amount: ${v.uiTokenAmount.uiAmount}`);
            });
        } else {
            console.log("None");
        }
        
        console.log("\n--- Post Token Balances ---");
        if (meta.postTokenBalances && meta.postTokenBalances.length > 0) {
            meta.postTokenBalances.forEach((v, i) => {
                console.log(`[${i}] Index: ${v.accountIndex} | Mint: ${v.mint} | Owner: ${v.owner} | Amount: ${v.uiTokenAmount.amount}`);
            });
        } else {
            console.log("None");
        }
        
        // Replicate parsing logic
        console.log("\n--- Replicating parsing logic ---");
        let best_owner = "Unknown";
        let best_amount = 0.0;
        
        if (meta.postTokenBalances && meta.postTokenBalances.length > 0) {
            for (const post_tb of meta.postTokenBalances) {
                const account_index = post_tb.accountIndex || 0;
                const post_raw = BigInt(post_tb.uiTokenAmount.amount || "0");
                const decimals = post_tb.uiTokenAmount.decimals || 6;
                
                // Find pre balance
                let pre_raw = BigInt(0);
                if (meta.preTokenBalances) {
                    const pre_tb = meta.preTokenBalances.find(p => p.accountIndex === account_index);
                    if (pre_tb) {
                        pre_raw = BigInt(pre_tb.uiTokenAmount.amount || "0");
                    }
                }
                
                if (post_raw > pre_raw) {
                    const increase = post_raw - pre_raw;
                    // convert to standard decimal amount based on decimals
                    const token_amount = Number(increase) / Math.pow(10, decimals);
                    
                    if (token_amount > best_amount) {
                        best_amount = token_amount;
                        if (post_tb.owner) {
                            best_owner = post_tb.owner;
                        }
                    }
                }
            }
        }
        
        console.log(`Recipient Owner: ${best_owner}`);
        console.log(`Amount Received: ${best_amount}`);
        
    } catch (error) {
        console.error("Error fetching or parsing:", error.message);
    }
}

main();
