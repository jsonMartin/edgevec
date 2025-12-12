#!/bin/bash
set -e

# Navigate to edgevec root
cd "$(dirname "$0")/.."

echo "🚀 [1/2] Building WASM target..."
wasm-pack build --target web

echo "✅ WASM Build Complete."
echo ""
echo "🚀 [2/2] Instructions to Run Demo:"
echo "1. Start Python Server in 'edgevec/' root:"
echo "   python3 -m http.server 8080"
echo ""
echo "2. Open Browser:"
echo "   http://localhost:8080/examples/browser/"
echo ""
echo "ℹ️  Note: We serve from root so '../../pkg' imports resolve correctly."

