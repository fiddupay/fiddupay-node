#!/bin/bash

# Concurrent Request Test
echo " FidduPay Concurrent Request Test"
echo "==================================="

BASE_URL="http://localhost:8080"

# Wait for backend to be ready
sleep 3

# Function to test registration
test_registration() {
    local id=$1
    local timestamp=$(date +%s%N | cut -b1-13)  # milliseconds
    local email="concurrent-test-${id}-${timestamp}@test.com"
    
    echo "Testing registration $id..."
    local response=$(curl -s -w "HTTP_CODE:%{http_code}" -X POST $BASE_URL/api/v1/merchant/register \
      -H "Content-Type: application/json" \
      -d "{\"email\":\"$email\",\"business_name\":\"Concurrent Test $id\",\"password\":\"TestPassword123!\"}")
    
    local http_code=$(echo "$response" | grep -o "HTTP_CODE:[0-9]*" | cut -d: -f2)
    local body=$(echo "$response" | sed 's/HTTP_CODE:[0-9]*$//')
    
    if [[ $http_code == "201" ]]; then
        local api_key=$(echo $body | jq -r '.api_key // empty')
        if [[ $api_key == sk_* ]]; then
            echo " Registration $id: SUCCESS - $api_key"
            return 0
        else
            echo " Registration $id: Invalid key - $api_key"
            return 1
        fi
    else
        echo " Registration $id: HTTP $http_code - $body"
        return 1
    fi
}

# Test 1: Sequential requests
echo "1. Testing sequential requests..."
success_count=0
for i in {1..5}; do
    if test_registration $i; then
        ((success_count++))
    fi
    sleep 0.5  # Small delay between requests
done
echo "Sequential: $success_count/5 successful"

# Test 2: Concurrent requests
echo ""
echo "2. Testing concurrent requests..."
pids=()
results_file="/tmp/concurrent_results.txt"
> "$results_file"  # Clear results file

# Launch 5 concurrent requests
for i in {1..5}; do
    (
        if test_registration "concurrent-$i"; then
            echo "SUCCESS" >> "$results_file"
        else
            echo "FAILED" >> "$results_file"
        fi
    ) &
    pids+=($!)
done

# Wait for all background processes
for pid in "${pids[@]}"; do
    wait $pid
done

# Count results
concurrent_success=$(grep -c "SUCCESS" "$results_file" 2>/dev/null || echo 0)
echo "Concurrent: $concurrent_success/5 successful"

# Test 3: Rapid fire test
echo ""
echo "3. Testing rapid fire requests..."
rapid_success=0
for i in {1..10}; do
    if test_registration "rapid-$i"; then
        ((rapid_success++))
    fi
    # No delay - rapid fire
done
echo "Rapid fire: $rapid_success/10 successful"

# Summary
echo ""
echo " CONCURRENT TEST SUMMARY"
echo "========================="
echo "Sequential requests: $success_count/5"
echo "Concurrent requests: $concurrent_success/5"
echo "Rapid fire requests: $rapid_success/10"

total_success=$((success_count + concurrent_success + rapid_success))
total_tests=20

if [[ $total_success -eq $total_tests ]]; then
    echo " ALL TESTS PASSED: Backend handles concurrent requests perfectly!"
elif [[ $total_success -gt $((total_tests / 2)) ]]; then
    echo "  PARTIAL SUCCESS: $total_success/$total_tests requests successful"
else
    echo " FAILED: Only $total_success/$total_tests requests successful"
    exit 1
fi

# Cleanup
rm -f "$results_file"
