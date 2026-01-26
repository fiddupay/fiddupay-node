#!/bin/bash
# Advanced Stress Test with Authentication

echo " fiddupay Advanced Stress Test"
echo "==============================="

cd /home/vibes/crypto-payment-gateway

BASE_URL="http://localhost:8080"
STRESS_USERS=100
STRESS_DURATION=60  # seconds
API_KEY="${API_KEY:-}"

echo " Stress Test Configuration:"
echo "  Base URL: $BASE_URL"
echo "  Concurrent Users: $STRESS_USERS"
echo "  Test Duration: ${STRESS_DURATION}s"
echo ""

# Check server
if ! curl -s "$BASE_URL/health" > /dev/null 2>&1; then
    echo " Server not running on $BASE_URL"
    exit 1
fi
echo " Server is running"

# Create results directory
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
RESULTS_DIR="stress_test_results/$TIMESTAMP"
mkdir -p "$RESULTS_DIR"

# Function for stress testing with authentication
stress_test_endpoint() {
    local endpoint=$1
    local method=${2:-GET}
    local auth_header=${3:-""}
    local data=${4:-""}
    local test_name=$5
    
    echo " Stress Testing: $test_name"
    echo "   Duration: ${STRESS_DURATION}s with $STRESS_USERS concurrent users"
    
    # Create stress test script
    cat > "$RESULTS_DIR/stress_script.sh" << EOF
#!/bin/bash
USER_ID=\$1
END_TIME=\$((SECONDS + $STRESS_DURATION))
RESULTS_FILE="$RESULTS_DIR/stress_user_\${USER_ID}.txt"

REQUEST_COUNT=0
SUCCESS_COUNT=0
ERROR_COUNT=0

while [ \$SECONDS -lt \$END_TIME ]; do
    START_TIME=\$(date +%s%3N)
    
    if [ "$method" = "POST" ]; then
        if [ -n "$auth_header" ]; then
            RESPONSE=\$(curl -s -w "%{http_code},%{time_total}" \\
                -X POST \\
                -H "Content-Type: application/json" \\
                -H "$auth_header" \\
                -d '$data' \\
                "$BASE_URL$endpoint" 2>/dev/null)
        else
            RESPONSE=\$(curl -s -w "%{http_code},%{time_total}" \\
                -X POST \\
                -H "Content-Type: application/json" \\
                -d '$data' \\
                "$BASE_URL$endpoint" 2>/dev/null)
        fi
    else
        if [ -n "$auth_header" ]; then
            RESPONSE=\$(curl -s -w "%{http_code},%{time_total}" \\
                -H "$auth_header" \\
                "$BASE_URL$endpoint" 2>/dev/null)
        else
            RESPONSE=\$(curl -s -w "%{http_code},%{time_total}" \\
                "$BASE_URL$endpoint" 2>/dev/null)
        fi
    fi
    
    HTTP_CODE=\$(echo "\$RESPONSE" | tail -1 | cut -d',' -f1)
    RESPONSE_TIME=\$(echo "\$RESPONSE" | tail -1 | cut -d',' -f2)
    
    REQUEST_COUNT=\$((REQUEST_COUNT + 1))
    
    if [ "\$HTTP_CODE" = "200" ] || [ "\$HTTP_CODE" = "201" ] || [ "\$HTTP_CODE" = "404" ]; then
        SUCCESS_COUNT=\$((SUCCESS_COUNT + 1))
    else
        ERROR_COUNT=\$((ERROR_COUNT + 1))
    fi
    
    echo "\$REQUEST_COUNT,\$HTTP_CODE,\$RESPONSE_TIME" >> "\$RESULTS_FILE"
    
    # Small delay to prevent overwhelming
    sleep 0.01
done

echo "USER_\$USER_ID,\$REQUEST_COUNT,\$SUCCESS_COUNT,\$ERROR_COUNT" >> "$RESULTS_DIR/user_summary.txt"
EOF
    
    chmod +x "$RESULTS_DIR/stress_script.sh"
    
    # Initialize summary file
    echo "User,Total_Requests,Success,Errors" > "$RESULTS_DIR/user_summary.txt"
    
    # Start stress test
    START_TIME=$(date +%s)
    
    for i in $(seq 1 $STRESS_USERS); do
        "$RESULTS_DIR/stress_script.sh" $i &
    done
    
    # Monitor progress
    echo "    Running stress test..."
    for i in $(seq 1 $STRESS_DURATION); do
        sleep 1
        printf "   Progress: %d/%d seconds\r" $i $STRESS_DURATION
    done
    echo ""
    
    # Wait for completion
    wait
    
    END_TIME=$(date +%s)
    ACTUAL_DURATION=$((END_TIME - START_TIME))
    
    # Analyze results
    echo "   ⏱️  Actual Duration: ${ACTUAL_DURATION}s"
    
    # Combine results
    cat "$RESULTS_DIR"/stress_user_*.txt > "$RESULTS_DIR/all_requests.txt" 2>/dev/null
    
    TOTAL_REQUESTS=$(wc -l < "$RESULTS_DIR/all_requests.txt" 2>/dev/null || echo "0")
    SUCCESS_REQUESTS=$(awk -F',' '$2 == 200 || $2 == 201 || $2 == 404 {count++} END {print count+0}' "$RESULTS_DIR/all_requests.txt" 2>/dev/null)
    ERROR_REQUESTS=$((TOTAL_REQUESTS - SUCCESS_REQUESTS))
    
    if [ $TOTAL_REQUESTS -gt 0 ]; then
        SUCCESS_RATE=$(awk "BEGIN {printf \"%.2f\", ($SUCCESS_REQUESTS/$TOTAL_REQUESTS)*100}")
        RPS=$(awk "BEGIN {printf \"%.2f\", $TOTAL_REQUESTS/$ACTUAL_DURATION}")
        
        # Response time analysis
        awk -F',' '{print $3}' "$RESULTS_DIR/all_requests.txt" | sort -n > "$RESULTS_DIR/response_times.txt"
        
        if [ -s "$RESULTS_DIR/response_times.txt" ]; then
            MIN_TIME=$(head -1 "$RESULTS_DIR/response_times.txt")
            MAX_TIME=$(tail -1 "$RESULTS_DIR/response_times.txt")
            AVG_TIME=$(awk '{sum+=$1} END {printf "%.3f", sum/NR}' "$RESULTS_DIR/response_times.txt")
            
            P95_LINE=$(awk "BEGIN {printf \"%.0f\", $TOTAL_REQUESTS*0.95}")
            P95_TIME=$(sed -n "${P95_LINE}p" "$RESULTS_DIR/response_times.txt")
        else
            MIN_TIME="0"
            MAX_TIME="0"
            AVG_TIME="0"
            P95_TIME="0"
        fi
        
        echo "    Stress Test Results:"
        echo "     Total Requests: $TOTAL_REQUESTS"
        echo "     Successful: $SUCCESS_REQUESTS ($SUCCESS_RATE%)"
        echo "     Failed: $ERROR_REQUESTS"
        echo "     Requests/sec: $RPS"
        echo "     Response Times:"
        echo "       Min: ${MIN_TIME}s"
        echo "       Avg: ${AVG_TIME}s"
        echo "       P95: ${P95_TIME}s"
        echo "       Max: ${MAX_TIME}s"
        
        # Performance assessment
        if (( $(echo "$RPS >= 200" | bc -l) )); then
            echo "    EXCELLENT throughput under stress!"
        elif (( $(echo "$RPS >= 100" | bc -l) )); then
            echo "    Good throughput under stress"
        else
            echo "   ⚠️  Low throughput under stress"
        fi
        
        if (( $(echo "$SUCCESS_RATE >= 95" | bc -l) )); then
            echo "    Excellent reliability under stress"
        else
            echo "   ⚠️  Reliability issues under stress"
        fi
        
    else
        echo "    No requests completed"
        RPS="0"
        SUCCESS_RATE="0"
        AVG_TIME="0"
        P95_TIME="0"
    fi
    
    echo ""
    
    # Save to summary
    echo "$test_name,$SUCCESS_RATE,$AVG_TIME,$P95_TIME,$RPS" >> "$RESULTS_DIR/stress_summary.csv"
    
    # Cleanup
    rm -f "$RESULTS_DIR"/stress_user_*.txt
    rm -f "$RESULTS_DIR/stress_script.sh"
}

# Initialize summary
echo "Test Name,Success Rate %,Avg Response Time,P95 Response Time,RPS" > "$RESULTS_DIR/stress_summary.csv"

# Run stress tests
stress_test_endpoint "/health" "GET" "" "" "Health Check Stress"
stress_test_endpoint "/metrics" "GET" "" "" "Metrics Stress"

# Test with invalid data to check error handling
INVALID_DATA='{"invalid":"data"}'
stress_test_endpoint "/api/v1/merchants/register" "POST" "" "$INVALID_DATA" "Error Handling Stress"

echo " Stress Test Summary"
echo "====================="

printf "%-25s %-12s %-15s %-15s %-10s\n" "Test Name" "Success %" "Avg Time (s)" "P95 Time (s)" "RPS"
printf "%-25s %-12s %-15s %-15s %-10s\n" "-------------------------" "----------" "-------------" "-------------" "--------"

tail -n +2 "$RESULTS_DIR/stress_summary.csv" | while IFS=',' read -r name success avg p95 rps; do
    printf "%-25s %-12s %-15s %-15s %-10s\n" "$name" "$success" "$avg" "$p95" "$rps"
done

echo ""
echo " Stress test results saved to: $RESULTS_DIR/"
echo " Stress test complete!"
