#!/bin/bash
# R-Image-Magic API Test Suite
# Run: ./scripts/test-api.sh

set -e

API_URL="${API_URL:-http://100.97.89.1:30880}"
API_KEY="${API_KEY:-rim_admintest1234567890abcdefghijklm}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

PASSED=0
FAILED=0

# Test helper
test_endpoint() {
    local name="$1"
    local expected_status="$2"
    local method="$3"
    local endpoint="$4"
    local data="$5"

    if [ "$method" = "POST" ]; then
        response=$(curl -s -w "\n%{http_code}" -X POST \
            -H "X-API-Key: $API_KEY" \
            -H "Content-Type: application/json" \
            -d "$data" \
            "$API_URL$endpoint")
    else
        response=$(curl -s -w "\n%{http_code}" \
            -H "X-API-Key: $API_KEY" \
            "$API_URL$endpoint")
    fi

    status=$(echo "$response" | tail -n1)
    body=$(echo "$response" | sed '$d')

    if [ "$status" = "$expected_status" ]; then
        echo -e "${GREEN}✓${NC} $name (HTTP $status)"
        ((PASSED++))
        return 0
    else
        echo -e "${RED}✗${NC} $name (expected $expected_status, got $status)"
        echo "  Response: $(echo "$body" | head -c 200)"
        ((FAILED++))
        return 1
    fi
}

echo "========================================"
echo "  R-Image-Magic API Test Suite"
echo "========================================"
echo "API URL: $API_URL"
echo "API Key: ${API_KEY:0:20}..."
echo ""

# ===========================================
# 1. Health & Public Endpoints
# ===========================================
echo -e "\n${YELLOW}1. Health & Public Endpoints${NC}"
echo "----------------------------------------"

test_endpoint "Health check (no auth)" "200" "GET" "/health"

# ===========================================
# 2. Authentication Tests
# ===========================================
echo -e "\n${YELLOW}2. Authentication Tests${NC}"
echo "----------------------------------------"

# Test with valid key
test_endpoint "Valid API key" "200" "GET" "/api/v1/templates"

# Test with invalid key
API_KEY_BACKUP=$API_KEY
API_KEY="invalid_key_12345678901234567890"
test_endpoint "Invalid API key rejected" "401" "GET" "/api/v1/templates"
API_KEY=$API_KEY_BACKUP

# Test with no key
response=$(curl -s -w "\n%{http_code}" "$API_URL/api/v1/templates")
status=$(echo "$response" | tail -n1)
if [ "$status" = "401" ]; then
    echo -e "${GREEN}✓${NC} Missing API key rejected (HTTP $status)"
    ((PASSED++))
else
    echo -e "${RED}✗${NC} Missing API key should be rejected (got $status)"
    ((FAILED++))
fi

# ===========================================
# 3. Templates API
# ===========================================
echo -e "\n${YELLOW}3. Templates API${NC}"
echo "----------------------------------------"

test_endpoint "List templates" "200" "GET" "/api/v1/templates"

# Check template count
response=$(curl -s -H "X-API-Key: $API_KEY" "$API_URL/api/v1/templates")
count=$(echo "$response" | jq -r '.count // 0')
if [ "$count" -ge 3 ]; then
    echo -e "${GREEN}✓${NC} Has $count templates loaded"
    ((PASSED++))
else
    echo -e "${RED}✗${NC} Expected at least 3 templates, got $count"
    ((FAILED++))
fi

# ===========================================
# 4. Mockup Generation
# ===========================================
echo -e "\n${YELLOW}4. Mockup Generation${NC}"
echo "----------------------------------------"

# Test with valid request
DESIGN_URL="https://upload.wikimedia.org/wikipedia/commons/thumb/a/a7/Camponotus_flavomarginatus_ant.jpg/220px-Camponotus_flavomarginatus_ant.jpg"

response=$(curl -s -w "\n%{http_code}" -X POST \
    -H "X-API-Key: $API_KEY" \
    -H "Content-Type: application/json" \
    -d "{
        \"template_id\": \"white_male_front_9x16\",
        \"design_url\": \"$DESIGN_URL\",
        \"placement\": {
            \"scale\": 0.8,
            \"offset_x\": 0,
            \"offset_y\": 0
        }
    }" \
    "$API_URL/api/v1/mockups/generate")

status=$(echo "$response" | tail -n1)
body=$(echo "$response" | sed '$d')

if [ "$status" = "200" ]; then
    echo -e "${GREEN}✓${NC} Mockup generation (HTTP $status)"
    ((PASSED++))

    # Check it's PNG
    if echo "$body" | jq -r '.mockup_url' | grep -q "data:image/png"; then
        echo -e "${GREEN}✓${NC} Output is PNG format"
        ((PASSED++))
    else
        echo -e "${RED}✗${NC} Output should be PNG format"
        ((FAILED++))
    fi

    # Check dimensions
    width=$(echo "$body" | jq -r '.metadata.dimensions.width')
    height=$(echo "$body" | jq -r '.metadata.dimensions.height')
    if [ "$width" = "1080" ] && [ "$height" = "1920" ]; then
        echo -e "${GREEN}✓${NC} Dimensions correct (${width}x${height})"
        ((PASSED++))
    else
        echo -e "${RED}✗${NC} Unexpected dimensions: ${width}x${height}"
        ((FAILED++))
    fi

    # Check generation time
    gen_time=$(echo "$body" | jq -r '.metadata.generation_time_ms')
    echo -e "  Generation time: ${gen_time}ms"
else
    echo -e "${RED}✗${NC} Mockup generation failed (HTTP $status)"
    echo "  Response: $(echo "$body" | head -c 200)"
    ((FAILED++))
fi

# Test invalid template
test_endpoint "Invalid template rejected" "404" "POST" "/api/v1/mockups/generate" \
    '{"template_id":"nonexistent","design_url":"https://example.com/img.png","placement":{"scale":0.8,"offset_x":0,"offset_y":0}}'

# Test missing placement
response=$(curl -s -w "\n%{http_code}" -X POST \
    -H "X-API-Key: $API_KEY" \
    -H "Content-Type: application/json" \
    -d '{"template_id":"white_male_front_9x16","design_url":"https://example.com/img.png"}' \
    "$API_URL/api/v1/mockups/generate")
status=$(echo "$response" | tail -n1)
if [ "$status" = "400" ]; then
    echo -e "${GREEN}✓${NC} Missing placement rejected (HTTP $status)"
    ((PASSED++))
else
    echo -e "${RED}✗${NC} Missing placement should be 400 (got $status)"
    ((FAILED++))
fi

# ===========================================
# 5. Usage & Billing API
# ===========================================
echo -e "\n${YELLOW}5. Usage & Billing API${NC}"
echo "----------------------------------------"

test_endpoint "Get usage stats" "200" "GET" "/api/v1/usage"
test_endpoint "Get current key info" "200" "GET" "/api/v1/keys/me"

# Check usage is being tracked
response=$(curl -s -H "X-API-Key: $API_KEY" "$API_URL/api/v1/usage")
total=$(echo "$response" | jq -r '.current_month.total_requests // 0')
if [ "$total" -gt 0 ]; then
    echo -e "${GREEN}✓${NC} Usage tracking working ($total requests this month)"
    ((PASSED++))
else
    echo -e "${YELLOW}!${NC} No usage recorded yet"
fi

# ===========================================
# 6. Rate Limiting
# ===========================================
echo -e "\n${YELLOW}6. Rate Limit Headers${NC}"
echo "----------------------------------------"

response=$(curl -s -D- -H "X-API-Key: $API_KEY" "$API_URL/api/v1/templates" -o /dev/null)

if echo "$response" | grep -qi "x-ratelimit-limit"; then
    limit=$(echo "$response" | grep -i "x-ratelimit-limit" | cut -d: -f2 | tr -d ' \r')
    remaining=$(echo "$response" | grep -i "x-ratelimit-remaining" | cut -d: -f2 | tr -d ' \r')
    echo -e "${GREEN}✓${NC} Rate limit headers present (limit: $limit, remaining: $remaining)"
    ((PASSED++))
else
    echo -e "${RED}✗${NC} Rate limit headers missing"
    ((FAILED++))
fi

# ===========================================
# Summary
# ===========================================
echo ""
echo "========================================"
echo "  Test Results"
echo "========================================"
echo -e "Passed: ${GREEN}$PASSED${NC}"
echo -e "Failed: ${RED}$FAILED${NC}"
echo ""

if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}All tests passed!${NC}"
    exit 0
else
    echo -e "${RED}Some tests failed!${NC}"
    exit 1
fi
