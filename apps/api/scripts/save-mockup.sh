#!/bin/bash
# Save a mockup to a viewable PNG file
# Usage: ./scripts/save-mockup.sh [output_file.png]

API_URL="${API_URL:-http://100.97.89.1:30880}"
API_KEY="${API_KEY:-rim_admintest1234567890abcdefghijklm}"
OUTPUT="${1:-mockup-output.png}"

echo "Generating mockup..."
echo "API: $API_URL"
echo "Output: $OUTPUT"

# Generate and save
curl -s -X POST \
  -H "X-API-Key: $API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "template_id": "white_male_front_9x16",
    "design_url": "https://httpbin.org/image/png",
    "placement": {
      "scale": 0.7,
      "offset_x": 0,
      "offset_y": 0
    }
  }' \
  "$API_URL/api/v1/mockups/generate" \
  | python3 -c "
import sys, json, base64
try:
    data = json.load(sys.stdin)
    if data.get('success'):
        b64 = data['mockup_url'].replace('data:image/png;base64,', '')
        sys.stdout.buffer.write(base64.b64decode(b64))
        print(f'Generation time: {data[\"metadata\"][\"generation_time_ms\"]}ms', file=sys.stderr)
        print(f'Dimensions: {data[\"metadata\"][\"dimensions\"][\"width\"]}x{data[\"metadata\"][\"dimensions\"][\"height\"]}', file=sys.stderr)
    else:
        print(f'Error: {data}', file=sys.stderr)
        sys.exit(1)
except Exception as e:
    print(f'Failed to parse response: {e}', file=sys.stderr)
    sys.exit(1)
" > "$OUTPUT"

if [ -f "$OUTPUT" ] && [ -s "$OUTPUT" ]; then
    echo ""
    echo "Saved: $OUTPUT"
    ls -lh "$OUTPUT"
    file "$OUTPUT"
else
    echo "Failed to generate mockup"
    exit 1
fi
