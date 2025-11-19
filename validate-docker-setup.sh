#!/bin/bash

# Docker Setup Validation Script for LDNS Project
# This script validates the Docker configuration and tests the setup

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    if [ $1 -eq 0 ]; then
        echo -e "${GREEN}✓${NC} $2"
    else
        echo -e "${RED}✗${NC} $2"
    fi
}

print_info() {
    echo -e "${YELLOW}ℹ${NC} $1"
}

print_header() {
    echo ""
    echo "================================"
    echo "$1"
    echo "================================"
}

# Check if Docker is running
print_header "Checking Docker Installation"

if ! command -v docker &> /dev/null; then
    print_status 1 "Docker is not installed"
    exit 1
fi
print_status 0 "Docker is installed: $(docker --version)"

if ! docker info &> /dev/null; then
    print_status 1 "Docker daemon is not running"
    print_info "Please start Docker Desktop or run: sudo systemctl start docker"
    exit 1
fi
print_status 0 "Docker daemon is running"

if ! command -v docker compose &> /dev/null; then
    print_status 1 "Docker Compose is not installed"
    exit 1
fi
print_status 0 "Docker Compose is installed: $(docker compose version)"

# Validate file structure
print_header "Validating File Structure"

FILES=(
    "compose.yaml"
    "dnsraw/Dockerfile"
    "dnsraw/.dockerignore"
    "dnsraw/Cargo.toml"
    "ldnsTUI/Dockerfile"
    "ldnsTUI/.dockerignore"
    "ldnsTUI/Cargo.toml"
    "dnsblock.txt"
)

for file in "${FILES[@]}"; do
    if [ -f "$file" ]; then
        print_status 0 "Found: $file"
    else
        print_status 1 "Missing: $file"
    fi
done

# Validate Dockerfile syntax
print_header "Validating Dockerfile Syntax"

if docker build --check -f dnsraw/Dockerfile dnsraw/ &> /dev/null; then
    print_status 0 "dnsraw/Dockerfile syntax is valid"
else
    print_status 1 "dnsraw/Dockerfile has syntax errors"
fi

if docker build --check -f ldnsTUI/Dockerfile ldnsTUI/ &> /dev/null; then
    print_status 0 "ldnsTUI/Dockerfile syntax is valid"
else
    print_status 1 "ldnsTUI/Dockerfile has syntax errors"
fi

# Validate compose.yaml
print_header "Validating Docker Compose Configuration"

if docker compose config > /dev/null 2>&1; then
    print_status 0 "compose.yaml syntax is valid"
else
    print_status 1 "compose.yaml has syntax errors"
    docker compose config
    exit 1
fi

# Check for required services
if docker compose config --services | grep -q "dns-server"; then
    print_status 0 "Service 'dns-server' is defined"
else
    print_status 1 "Service 'dns-server' is not defined"
fi

if docker compose config --services | grep -q "tui"; then
    print_status 0 "Service 'tui' is defined"
else
    print_status 1 "Service 'tui' is not defined"
fi

# Check port availability
print_header "Checking Port Availability"

check_port() {
    local port=$1
    local protocol=$2
    if lsof -i :$port &> /dev/null; then
        print_status 1 "Port $port ($protocol) is already in use"
        lsof -i :$port
        return 1
    else
        print_status 0 "Port $port ($protocol) is available"
        return 0
    fi
}

check_port 53 "DNS"
check_port 8080 "API"

# Validate build context
print_header "Validating Build Context"

if [ -d "dnsraw/src" ]; then
    print_status 0 "dnsraw source directory exists"
else
    print_status 1 "dnsraw source directory is missing"
fi

if [ -d "ldnsTUI/src" ]; then
    print_status 0 "ldnsTUI source directory exists"
else
    print_status 1 "ldnsTUI source directory is missing"
fi

# Check for common issues
print_header "Checking for Common Issues"

# Check if .dockerignore includes target/
if grep -q "target/" dnsraw/.dockerignore; then
    print_status 0 "dnsraw/.dockerignore excludes target/ directory"
else
    print_status 1 "dnsraw/.dockerignore should exclude target/ directory"
fi

if grep -q "target/" ldnsTUI/.dockerignore; then
    print_status 0 "ldnsTUI/.dockerignore excludes target/ directory"
else
    print_status 1 "ldnsTUI/.dockerignore should exclude target/ directory"
fi

# Check Docker disk space
print_header "Docker System Information"

DISK_USAGE=$(docker system df --format "{{.Type}}\t{{.Size}}" 2>/dev/null || echo "Unable to check")
echo "$DISK_USAGE"

# Provide build instructions
print_header "Next Steps"

echo ""
echo "To build the images, run:"
echo "  docker compose build"
echo ""
echo "To build without cache (clean build):"
echo "  docker compose build --no-cache"
echo ""
echo "To start the DNS server:"
echo "  docker compose up -d dns-server"
echo ""
echo "To view logs:"
echo "  docker compose logs -f dns-server"
echo ""
echo "To run the TUI:"
echo "  docker compose run --rm tui"
echo ""
echo "For more information, see DOCKER_SETUP.md"
echo ""

print_status 0 "Validation complete! Your Docker setup is ready to build."
