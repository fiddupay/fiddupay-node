#!/bin/bash
# fiddupay High-Performance Endpoint Load Test

echo " fiddupay High-Performance Endpoint Load Test"
echo "=============================================="

cd /home/vibes/crypto-payment-gateway

# Configuration
BASE_URL="http://localhost:8080"
CONCURRENT_USERS=50
REQUESTS_PER_USER=20
TOTAL_REQUESTS=$((CONCURRENT_USERS * REQUESTS_PER_USER))

echo " Test Configuration:"
echo "  Base URL: $BASE_URL"
echo "  Concurrent Users: $CONCURRENT_USERS"
echo "  Requests per User: $REQUESTS_PER_USER"
echo "  Total Requests: $TOTAL_REQUESTS"
echo ""

# Check if server is running
echo " Checking server status..."
if ! curl -s "$BASE_URL/health" > /dev/null 2>&1; then
    echo " Server not running on $BASE_URL"
    echo "   Please start the server with: cargo run"
    exit 1
fi
echo " Server is running"
echo ""

# Create test data directory
mkdir -p load_test_results
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
RESULTS_DIR="load_test_results/$TIMESTAMP"
mkdir -p "$RESULTS_DIR"

# Function to run concurrent requests
run_concurrent_test() {
    local endpoint=$1
    local method=${2:-GET}
    local data=${3:-""}
    local test_name=$4
    local expected_status=${5:-200}
    
    echo " Testing: $test_name"
    echo "   Endpoint: $method $endpoint"
    echo "   Concurrent Users: $CONCURRENT_USERS"
    
    # Create temporary script for concurrent execution
    cat > "$RESULTS_DIR/test_script.sh" << EOF
#!/bin/bash
USER_ID=\$1
RESULTS_FILE="$RESULTS_DIR/user_\${USER_ID}_results.txt"

for i in \$(seq 1 $REQUESTS_PER_USER); do
    START_TIME=\$(date +%s%3N)
    
    if [ "$method" = "POST" ]; then
        RESPONSE=\$(curl -s -w "%{http_code},%{time_total},%{time_connect},%{time_starttransfer}" \\
            -X POST \\
            -H "Content-Type: application/json" \\
            -d '$data' \\
            "$BASE_URL$endpoint" 2>/dev/null)
    else
        RESPONSE=\$(curl -s -w "%{http_code},%{time_total},%{time_connect},%{time_starttransfer}" \\
            "$BASE_URL$endpoint" 2>/dev/null)
    fi
    
    END_TIME=\$(date +%s%3N)
    DURATION=\$((END_TIME - START_TIME))
    
    echo "\$RESPONSE,\$DURATION" >> "\$RESULTS_FILE"
done
EOF
    
    chmod +x "$RESULTS_DIR/test_script.sh"
    
    # Run concurrent users
    START_TIME=$(date +%s)
    
    for i in $(seq 1 $CONCURRENT_USERS); do
        "$RESULTS_DIR/test_script.sh" $i &
    done
    
    # Wait for all background jobs to complete
    wait
    
    END_TIME=$(date +%s)
    TOTAL_DURATION=$((END_TIME - START_TIME))
    
    # Analyze results
    echo "   ⏱  Total Test Duration: ${TOTAL_DURATION}s"
    
    # Combine all results
    cat "$RESULTS_DIR"/user_*_results.txt > "$RESULTS_DIR/combined_results.txt"
    
    # Calculate statistics
    TOTAL_REQUESTS_MADE=$(wc -l < "$RESULTS_DIR/combined_results.txt")
    SUCCESS_COUNT=$(awk -F',' -v status="$expected_status" '$1 == status {count++} END {print count+0}' "$RESULTS_DIR/combined_results.txt")
    ERROR_COUNT=$((TOTAL_REQUESTS_MADE - SUCCESS_COUNT))
    SUCCESS_RATE=$(awk "BEGIN {printf \"%.2f\", ($SUCCESS_COUNT/$TOTAL_REQUESTS_MADE)*100}")
    
    # Response time statistics
    awk -F',' '{print $2}' "$RESULTS_DIR/combined_results.txt" | sort -n > "$RESULTS_DIR/response_times.txt"
    
    MIN_TIME=$(head -1 "$RESULTS_DIR/response_times.txt")
    MAX_TIME=$(tail -1 "$RESULTS_DIR/response_times.txt")
    
    # Calculate percentiles
    P50_LINE=$(awk "BEGIN {printf \"%.0f\", $TOTAL_REQUESTS_MADE*0.5}")
    P95_LINE=$(awk "BEGIN {printf \"%.0f\", $TOTAL_REQUESTS_MADE*0.95}")
    P99_LINE=$(awk "BEGIN {printf \"%.0f\", $TOTAL_REQUESTS_MADE*0.99}")
    
    P50_TIME=$(sed -n "${P50_LINE}p" "$RESULTS_DIR/response_times.txt")
    P95_TIME=$(sed -n "${P95_LINE}p" "$RESULTS_DIR/response_times.txt")
    P99_TIME=$(sed -n "${P99_LINE}p" "$RESULTS_DIR/response_times.txt")
    
    AVG_TIME=$(awk '{sum+=$1} END {printf "%.3f", sum/NR}' "$RESULTS_DIR/response_times.txt")
    
    RPS=$(awk "BEGIN {printf \"%.2f\", $SUCCESS_COUNT/$TOTAL_DURATION}")
    
    echo "    Results:"
    echo "     Total Requests: $TOTAL_REQUESTS_MADE"
    echo "     Successful: $SUCCESS_COUNT ($SUCCESS_RATE%)"
    echo "     Failed: $ERROR_COUNT"
    echo "     Requests/sec: $RPS"
    echo "     Response Times (seconds):"
    echo "       Min: $MIN_TIME"
    echo "       Avg: $AVG_TIME"
    echo "       P50: $P50_TIME"
    echo "       P95: $P95_TIME"
    echo "       P99: $P99_TIME"
    echo "       Max: $MAX_TIME"
    
    # Performance assessment
    if (( $(echo "$SUCCESS_RATE >= 99.0" | bc -l) )); then
        echo "    Excellent reliability ($SUCCESS_RATE% success rate)"
    elif (( $(echo "$SUCCESS_RATE >= 95.0" | bc -l) )); then
        echo "    Good reliability ($SUCCESS_RATE% success rate)"
    else
        echo "     Poor reliability ($SUCCESS_RATE% success rate)"
    fi
    
    if (( $(echo "$P95_TIME < 1.0" | bc -l) )); then
        echo "    Excellent performance (P95 < 1s)"
    elif (( $(echo "$P95_TIME < 2.0" | bc -l) )); then
        echo "    Good performance (P95 < 2s)"
    else
        echo "     Slow performance (P95 >= 2s)"
    fi
    
    if (( $(echo "$RPS >= 100" | bc -l) )); then
        echo "    High throughput ($RPS RPS)"
    elif (( $(echo "$RPS >= 50" | bc -l) )); then
        echo "    Good throughput ($RPS RPS)"
    else
        echo "     Low throughput ($RPS RPS)"
    fi
    
    echo ""
    
    # Clean up user result files
    rm -f "$RESULTS_DIR"/user_*_results.txt
    rm -f "$RESULTS_DIR/test_script.sh"
    
    # Save summary
    echo "$test_name,$SUCCESS_RATE,$AVG_TIME,$P95_TIME,$RPS" >> "$RESULTS_DIR/summary.csv"
}

# Initialize summary file
echo "Test Name,Success Rate %,Avg Response Time,P95 Response Time,RPS" > "$RESULTS_DIR/summary.csv"

# Test 1: Health Check Endpoint
run_concurrent_test "/health" "GET" "" "Health Check" 200

# Test 2: Metrics Endpoint  
run_concurrent_test "/metrics" "GET" "" "Metrics" 200

# Test 3: API Documentation
run_concurrent_test "/api/v1" "GET" "" "API Root" 200

# Test 4: Merchant Registration (if endpoint exists)
REGISTER_DATA='{"business_name":"Load Test Business","email":"loadtest@example.com","password":"testpass123"}'
run_concurrent_test "/api/v1/merchant/register" "POST" "$REGISTER_DATA" "Merchant Registration" 201

# Test 5: Invalid Endpoint (404 handling)
run_concurrent_test "/api/v1/nonexistent" "GET" "" "404 Handling" 404

# Test 6: CORS Preflight
run_concurrent_test "/api/v1/merchants" "OPTIONS" "" "CORS Preflight" 200

echo " Load Test Summary"
echo "==================="

# Overall summary
echo " Test Results Summary:"
echo ""
printf "%-25s %-12s %-15s %-15s %-10s\n" "Test Name" "Success %" "Avg Time (s)" "P95 Time (s)" "RPS"
printf "%-25s %-12s %-15s %-15s %-10s\n" "-------------------------" "----------" "-------------" "-------------" "--------"

tail -n +2 "$RESULTS_DIR/summary.csv" | while IFS=',' read -r name success avg p95 rps; do
    printf "%-25s %-12s %-15s %-15s %-10s\n" "$name" "$success" "$avg" "$p95" "$rps"
done

echo ""

# Calculate overall performance score
OVERALL_SUCCESS=$(awk -F',' 'NR>1 {sum+=$2; count++} END {printf "%.1f", sum/count}' "$RESULTS_DIR/summary.csv")
OVERALL_P95=$(awk -F',' 'NR>1 {sum+=$4; count++} END {printf "%.3f", sum/count}' "$RESULTS_DIR/summary.csv")
OVERALL_RPS=$(awk -F',' 'NR>1 {sum+=$5; count++} END {printf "%.1f", sum/count}' "$RESULTS_DIR/summary.csv")

echo " Overall Performance Score:"
echo "  Average Success Rate: $OVERALL_SUCCESS%"
echo "  Average P95 Response Time: ${OVERALL_P95}s"
echo "  Average Throughput: $OVERALL_RPS RPS"

# Performance grade
SCORE=0

if (( $(echo "$OVERALL_SUCCESS >= 99.0" | bc -l) )); then
    SCORE=$((SCORE + 3))
elif (( $(echo "$OVERALL_SUCCESS >= 95.0" | bc -l) )); then
    SCORE=$((SCORE + 2))
else
    SCORE=$((SCORE + 1))
fi

if (( $(echo "$OVERALL_P95 < 1.0" | bc -l) )); then
    SCORE=$((SCORE + 3))
elif (( $(echo "$OVERALL_P95 < 2.0" | bc -l) )); then
    SCORE=$((SCORE + 2))
else
    SCORE=$((SCORE + 1))
fi

if (( $(echo "$OVERALL_RPS >= 100" | bc -l) )); then
    SCORE=$((SCORE + 3))
elif (( $(echo "$OVERALL_RPS >= 50" | bc -l) )); then
    SCORE=$((SCORE + 2))
else
    SCORE=$((SCORE + 1))
fi

echo ""
echo " Performance Grade: $SCORE/9"

if [ $SCORE -ge 8 ]; then
    echo " OUTSTANDING! Your endpoints are highly optimized for concurrent load!"
elif [ $SCORE -ge 6 ]; then
    echo " EXCELLENT! Your endpoints handle concurrent load very well!"
elif [ $SCORE -ge 4 ]; then
    echo " GOOD! Your endpoints handle concurrent load adequately!"
else
    echo "  NEEDS IMPROVEMENT! Consider optimizing for better concurrent performance!"
fi

echo ""
echo " Detailed results saved to: $RESULTS_DIR/"
echo " Load test complete!"

# Recommendations
echo ""
echo " Performance Recommendations:"
if (( $(echo "$OVERALL_P95 >= 2.0" | bc -l) )); then
    echo "  - Consider adding response caching for frequently accessed endpoints"
    echo "  - Review database query performance and indexing"
fi

if (( $(echo "$OVERALL_RPS < 50" | bc -l) )); then
    echo "  - Consider increasing connection pool size"
    echo "  - Review async/await patterns for better concurrency"
fi

if (( $(echo "$OVERALL_SUCCESS < 95.0" | bc -l) )); then
    echo "  - Review error handling and timeout configurations"
    echo "  - Consider implementing circuit breaker patterns"
fi

echo "  - Monitor resource usage (CPU, memory, connections) during peak load"
echo "  - Consider implementing rate limiting for production deployment"
