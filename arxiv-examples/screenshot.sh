#!/bin/bash
# Take screenshots of HTML files using headless Chrome
# Usage: ./screenshot.sh <html_file> <output_png> [viewport_width]
set -e

HTML_FILE="$1"
OUTPUT_PNG="$2"
WIDTH="${3:-1200}"

if [ -z "$HTML_FILE" ] || [ -z "$OUTPUT_PNG" ]; then
  echo "Usage: $0 <html_file> <output_png> [viewport_width]"
  exit 1
fi

# Convert to absolute path for Chrome
ABS_HTML=$(realpath "$HTML_FILE")

google-chrome --headless --disable-gpu --no-sandbox \
  --window-size=${WIDTH},800 \
  --screenshot="$OUTPUT_PNG" \
  --virtual-time-budget=5000 \
  "file://$ABS_HTML" 2>/dev/null

echo "Screenshot saved: $OUTPUT_PNG"
