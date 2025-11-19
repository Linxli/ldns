#!/bin/bash

# Demo script to run the DNS server and TUI together

echo "======================================"
echo "DNS Server + TUI Demo"
echo "======================================"
echo ""

# Check if DNS server is already running
if lsof -Pi :8080 -sTCP:LISTEN -t >/dev/null 2>&1 ; then
    echo "✓ DNS API server is already running on port 8080"
else
    echo "✗ DNS API server is not running!"
    echo ""
    echo "Please start the DNS server first:"
    echo "  cd dnsraw && cargo run"
    echo ""
    echo "Then in another terminal, run the TUI:"
    echo "  cd ldnsTUI && cargo run"
    exit 1
fi

echo ""
echo "Starting the TUI..."
echo ""
echo "Controls:"
echo "  [B] - Update Blocklist URL"
echo "  [Q] - Quit"
echo ""
echo "Press any key to continue..."
read -n 1

cd ldnsTUI && cargo run
