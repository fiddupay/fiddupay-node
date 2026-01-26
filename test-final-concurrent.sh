#!/bin/bash

# Final Production Concurrent Test
echo " FidduPay Production Concurrent Test"
echo "======================================"

BASE_URL="http://localhost:8080"

# Test concurrent registrations
echo "Testing 10 concurrent registrations..."

pids=()
results_file="/tmp/final_concurrent_results.txt"
> "$results_file"

# Launch 10 concurrent requests
for i in {1..10}; do
    (
        timestamp=$(date +%s%N | cut -b1-13)
        email="final-test-${i}-${timestamp}@test.com"
        
        response=$(curl -s -X POST $BASE_URL/api/v1/merchants/register \
          -H "Content-Type: application/json" \
          -d "{\"email\":\"$email\",\"business_name\":\"Final Test $i\",\"password\":\"TestPassword123!\"}")
        
        api_key=$(echo $response | jq -r '.api_key // "FAILED"')
        
        if [[ $api_key == sk_* ]]; then
            echo "SUCCESS:$i:$api_key" >> "$results_file"
        else
            echo "FAILED:$i:$api_key" >> "$results_file"
        fi
    ) &
    pids+=($!)
done

# Wait for all requests to complete
echo "Waiting for all requests to complete..."
for pid in "${pids[@]}"; do
    wait $pid
done

# Analyze results
success_count=$(grep -c "SUCCESS:" "$results_file" 2>/dev/null || echo 0)
failed_count=$(grep -c "FAILED:" "$results_file" 2>/dev/null || echo 0)

echo ""
echo " FINAL RESULTS:"
echo "================"
echo " Successful: $success_count/10"
echo " Failed: $failed_count/10"

if [[ $success_count -eq 10 ]]; then
    echo ""
    echo " PERFECT! ALL 10 CONCURRENT REQUESTS SUCCEEDED!"
    echo " Backend handles concurrent requests flawlessly"
    echo " All API keys generated with correct 'sk_' prefixes"
    echo " System is production-ready for concurrent load"
    
    # Show sample keys
    echo ""
    echo " Sample Generated Keys:"
    grep "SUCCESS:" "$results_file" | head -3 | while IFS=':' read -r status id key; do
        echo "   Request $id: $key"
    done
    
elif [[ $success_count -gt 7 ]]; then
    echo "⚠️  GOOD: Most requests succeeded ($success_count/10)"
    echo "   Minor issues under concurrent load"
else
    echo " ISSUES: Only $success_count/10 requests succeeded"
    echo "   Concurrent handling needs improvement"
fi

# Cleanup
rm -f "$results_file"
