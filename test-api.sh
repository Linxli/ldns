#!/bin/bash

# API Testing Script for DNS Blocklist
# This script demonstrates how to test the API endpoints

API_URL="http://localhost:8080"

echo "======================================"
echo "DNS Blocklist API Test"
echo "======================================"
echo ""

# Test 1: Valid URL
echo "Test 1: Updating blocklist with a valid URL"
echo "--------------------------------------------"
curl -X PUT "${API_URL}/blocklist" \
  -H "Content-Type: application/json" \
  -d '{"url": "https://gitlab.com/hagezi/mirror/-/raw/main/dns-blocklists/adblock/ultimate.txt"}' \
  -w "\nHTTP Status: %{http_code}\n" \
  -s | jq .

echo ""
echo ""

# Test 2: Invalid URL format
echo "Test 2: Testing with an invalid URL (should return 400)"
echo "--------------------------------------------------------"
curl -X PUT "${API_URL}/blocklist" \
  -H "Content-Type: application/json" \
  -d '{"url": "not-a-valid-url"}' \
  -w "\nHTTP Status: %{http_code}\n" \
  -s | jq .

echo ""
echo ""

# Test 3: Non-existent URL
echo "Test 3: Testing with a non-existent URL (should return error)"
echo "--------------------------------------------------------------"
curl -X PUT "${API_URL}/blocklist" \
  -H "Content-Type: application/json" \
  -d '{"url": "https://example.com/doesnt-exist.txt"}' \
  -w "\nHTTP Status: %{http_code}\n" \
  -s | jq .

echo ""
echo "======================================"
echo "Tests complete!"
echo "======================================"
