#!/bin/bash
# R-Image-Magic Load Test
# Tests concurrent mockup generation capacity

API_URL="${API_URL:-http://100.97.89.1:30880}"
API_KEY="${API_KEY:-rim_admintest1234567890abcdefghijklm}"
CONCURRENT="${1:-5}"
TOTAL="${2:-10}"

echo "========================================"
echo "  R-Image-Magic Load Test"
echo "========================================"
echo "API URL: $API_URL"
echo "Concurrent: $CONCURRENT"
echo "Total requests: $TOTAL"
echo ""

# Check if hey is installed, if not use curl-based approach
if command -v hey &> /dev/null; then
    echo "Using 'hey' for load testing..."
    hey -n $TOTAL -c $CONCURRENT \
        -H "X-API-Key: $API_KEY" \
        -H "Content-Type: application/json" \
        -m POST \
        -d '{"template_id":"white_male_front_9x16","design_url":"https://httpbin.org/image/png","placement":{"scale":0.8,"offset_x":0,"offset_y":0}}' \
        "$API_URL/api/v1/mockups/generate"
else
    echo "Using parallel curl for load testing..."
    echo "(Install 'hey' for better metrics: go install github.com/rakyll/hey@latest)"
    echo ""

    START=$(date +%s.%N)

    # Create temp file for results
    RESULTS=$(mktemp)

    # Run requests in parallel
    for i in $(seq 1 $TOTAL); do
        (
            start=$(date +%s.%N)
            response=$(curl -s -o /dev/null -w "%{http_code},%{time_total}" \
                -X POST \
                -H "X-API-Key: $API_KEY" \
                -H "Content-Type: application/json" \
                -d '{"template_id":"white_male_front_9x16","design_url":"https://httpbin.org/image/png","placement":{"scale":0.8,"offset_x":0,"offset_y":0}}' \
                "$API_URL/api/v1/mockups/generate")
            echo "$response" >> "$RESULTS"
        ) &

        # Limit concurrency
        if (( i % CONCURRENT == 0 )); then
            wait
        fi
    done
    wait

    END=$(date +%s.%N)
    DURATION=$(echo "$END - $START" | bc)

    # Analyze results
    SUCCESS=$(grep "^200" "$RESULTS" | wc -l)
    FAILED=$(grep -v "^200" "$RESULTS" | wc -l)

    # Calculate average time
    AVG_TIME=$(awk -F, '{sum+=$2; count++} END {print sum/count}' "$RESULTS")
    MIN_TIME=$(awk -F, 'NR==1||$2<min{min=$2} END{print min}' "$RESULTS")
    MAX_TIME=$(awk -F, 'NR==1||$2>max{max=$2} END{print max}' "$RESULTS")

    RPS=$(echo "scale=2; $TOTAL / $DURATION" | bc)

    echo "----------------------------------------"
    echo "Results:"
    echo "  Total time: ${DURATION}s"
    echo "  Requests/sec: $RPS"
    echo "  Success: $SUCCESS"
    echo "  Failed: $FAILED"
    echo ""
    echo "Latency:"
    echo "  Min: ${MIN_TIME}s"
    echo "  Avg: ${AVG_TIME}s"
    echo "  Max: ${MAX_TIME}s"
    echo "----------------------------------------"

    rm -f "$RESULTS"
fi
