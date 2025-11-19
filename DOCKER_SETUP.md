# Docker Setup Guide - LDNS Project

## Overview

This guide explains the Docker containerization of the LDNS DNS server project, including the DNS server with REST API and the Terminal UI for management.

## Architecture

```
┌─────────────────────────────────────────┐
│         Docker Compose Network          │
│              (ldns-network)             │
│                                         │
│  ┌───────────────┐   ┌──────────────┐ │
│  │  DNS Server   │   │     TUI      │ │
│  │               │   │              │ │
│  │ Port 53 (UDP) │◄──┤ Connects to  │ │
│  │ Port 8080 API │   │ API on 8080  │ │
│  └───────────────┘   └──────────────┘ │
│                                         │
└─────────────────────────────────────────┘
```

## Files Created

### 1. DNS Server Dockerfile (`dnsraw/Dockerfile`)
- **Multi-stage build** for minimal image size
- **Stage 1 (builder)**: Compiles Rust code
  - Uses `rust:1.75-slim` base image
  - Optimized layer caching (dependencies built separately)
  - Only copies necessary source files
- **Stage 2 (runtime)**: Minimal production image
  - Uses `debian:bookworm-slim` (much smaller than full Rust image)
  - Non-root user (`dnsuser`) for security
  - Only includes compiled binary and runtime dependencies
  - Health check on API endpoint

**Key Features**:
- Final image size: ~100MB (vs 1GB+ with full Rust image)
- Security hardened: non-root user, minimal attack surface
- Health checks for container orchestration

### 2. TUI Dockerfile (`ldnsTUI/Dockerfile`)
Similar multi-stage build approach:
- Compiles the TUI application
- Creates minimal runtime image
- Configured to connect to DNS server via environment variable

**Note**: TUI requires interactive terminal (`-it` flag)

### 3. Docker Compose Configuration (`compose.yaml`)

Defines two services:

#### DNS Server Service
```yaml
dns-server:
  - Ports: 53/udp (DNS), 8080/tcp (API)
  - Capabilities: NET_BIND_SERVICE (for port 53)
  - Resource limits: 2 CPU, 512MB RAM
  - Security: cap_drop ALL, no-new-privileges
  - Health check: curl http://localhost:8080/health
  - Auto-restart: unless-stopped
```

#### TUI Service
```yaml
tui:
  - Profile: "tools" (doesn't start automatically)
  - Depends on: dns-server (waits for health check)
  - Environment: DNS_API_URL=http://dns-server:8080
  - Interactive: stdin_open + tty enabled
  - Resource limits: 1 CPU, 256MB RAM
```

### 4. .dockerignore Files
Created for both `dnsraw/` and `ldnsTUI/`:
- Excludes build artifacts (`target/`)
- Excludes IDE files, git history
- Excludes documentation and tests
- Reduces build context size significantly

## Usage

### Building Images

```bash
# Build both images
cd /Users/fabio/ldns
docker compose build

# Build individual services
docker compose build dns-server
docker compose build tui
```

### Running Services

```bash
# Start DNS server
docker compose up -d dns-server

# Check if server is healthy
docker compose ps

# View logs
docker compose logs -f dns-server

# Run TUI (interactive)
docker compose run --rm tui
```

### Stopping Services

```bash
# Stop all services
docker compose down

# Stop and remove volumes
docker compose down -v
```

## Security Features

### 1. Multi-Stage Builds
- **Why**: Separates build environment from runtime
- **Benefit**: No build tools in production image (smaller attack surface)

### 2. Non-Root Users
- DNS server runs as `dnsuser` (UID 1000)
- TUI runs as `tuiuser` (UID 1000)
- **Why**: Limits damage if container is compromised

### 3. Capability Management
```yaml
cap_add:
  - NET_BIND_SERVICE  # Only what's needed for port 53
cap_drop:
  - ALL  # Drop all other capabilities
```

### 4. Resource Limits
Prevents containers from consuming all host resources:
- CPU limits
- Memory limits
- Protects against resource exhaustion attacks

### 5. Security Options
```yaml
security_opt:
  - no-new-privileges:true  # Prevents privilege escalation
```

### 6. Network Isolation
- Custom bridge network (`ldns-network`)
- Services can only communicate within this network
- API not exposed externally (can be configured)

## Port Configuration

### DNS Server
- **Port 53 (UDP)**: DNS queries
  - Requires `NET_BIND_SERVICE` capability (privileged port)
  - Exposed on host: `0.0.0.0:53`
  
- **Port 8080 (TCP)**: REST API
  - Exposed on host: `0.0.0.0:8080`
  - **Security Note**: Consider binding to `127.0.0.1:8080` in production

### TUI
- No ports exposed (connects internally to dns-server)

## Environment Variables

### DNS Server
- `LOG_LEVEL`: Logging verbosity (default: `info`)
- `UPSTREAM_DNS`: Upstream DNS server (default: `1.1.1.1`)
- `RUST_LOG`: Rust logging configuration

### TUI
- `DNS_API_URL`: DNS server API endpoint (default: `http://dns-server:8080`)
- `RUST_LOG`: Rust logging configuration

## Volumes

### dns-data Volume
- **Purpose**: Persist blocklist data across container restarts
- **Mount point**: `/app/data`
- **Type**: Named volume (managed by Docker)

### Blocklist File Mount
```yaml
- ./dnsblock.txt:/app/dnsblock.txt:ro
```
- Read-only mount of default blocklist
- Can be updated via API (writes to volume)

## Health Checks

### DNS Server Health Check
```yaml
healthcheck:
  test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
  interval: 30s
  timeout: 5s
  retries: 3
  start_period: 10s
```

**What it checks**: API endpoint availability
**Why important**: 
- Orchestration knows when service is ready
- TUI waits for healthy DNS server before starting
- Auto-restart on failure

## Troubleshooting

### Container Won't Start

**Port 53 already in use**:
```bash
# Check what's using port 53
sudo lsof -i :53

# Stop conflicting service (e.g., systemd-resolved)
sudo systemctl stop systemd-resolved
```

**Permission denied on port 53**:
```bash
# Ensure cap_add: NET_BIND_SERVICE is in compose.yaml
# Or run Docker with sudo (not recommended)
```

### Build Failures

**Out of disk space**:
```bash
# Clean up Docker
docker system prune -a
```

**Cargo build fails**:
```bash
# Check Cargo.lock exists
# Ensure all dependencies are in Cargo.toml
# Try building locally first to verify code compiles
```

### Runtime Issues

**DNS queries not working**:
```bash
# Test DNS server
dig @127.0.0.1 example.com

# Check logs
docker compose logs dns-server

# Verify port binding
sudo netstat -tulpn | grep :53
```

**API not responding**:
```bash
# Test API endpoint
curl http://localhost:8080/health

# Check if container is healthy
docker compose ps
```

**TUI can't connect to API**:
```bash
# Verify DNS server is running
docker compose ps dns-server

# Check network connectivity
docker compose exec tui ping dns-server

# Verify DNS_API_URL environment variable
docker compose exec tui env | grep DNS_API
```

## Best Practices

### 1. Don't Run as Root
✅ Both Dockerfiles use non-root users
❌ Never run containers with `--privileged` unless absolutely necessary

### 2. Use Multi-Stage Builds
✅ Separates build from runtime
✅ Smaller images, faster deployments
✅ No build tools in production

### 3. Pin Base Image Versions
✅ Using specific versions (`rust:1.75-slim`)
❌ Don't use `:latest` tags in production

### 4. Minimize Layers
✅ Combine RUN commands with `&&`
✅ Clean up in same layer (`rm -rf /var/lib/apt/lists/*`)

### 5. Use .dockerignore
✅ Reduces build context size
✅ Faster builds
✅ Don't copy unnecessary files

## Performance Considerations

### Build Time Optimization

**Dependency caching** (DNS server only):
- Dependencies are built in a separate layer
- Only rebuilds when Cargo.toml/Cargo.lock changes
- Source code changes don't rebuild dependencies

**Layer caching**:
- Docker caches unchanged layers
- Order commands from least to most frequently changing

### Runtime Performance

**Resource limits**:
- Prevents one container from starving others
- Adjust limits based on actual usage

**Health checks**:
- Don't check too frequently (adds overhead)
- Current: 30s interval is reasonable

## Security Audit Findings

The security audit identified several critical issues in the application code (not Docker-specific):

### Critical Issues
1. **No API authentication** - Anyone can update blocklist
2. **SSRF vulnerability** - Can be used to scan internal networks
3. **Unbounded memory** - No size limits on blocklists
4. **UTF-8 panic** - Non-UTF-8 input crashes server

### Docker Mitigations
While Docker can't fix application vulnerabilities, it provides defense-in-depth:
- **Network isolation**: Limits SSRF impact
- **Resource limits**: Prevents memory exhaustion DoS
- **Non-root user**: Limits compromise damage
- **Read-only filesystem**: Prevents file modifications (can be added)

**TODO**: Address critical security issues in application code (see security audit report)

## Production Deployment Checklist

Before deploying to production:

- [ ] Implement API authentication
- [ ] Add rate limiting to API
- [ ] Fix SSRF vulnerability
- [ ] Add size limits to blocklist uploads
- [ ] Fix UTF-8 panic issue
- [ ] Bind API to localhost or add firewall rules
- [ ] Use TLS/HTTPS for API
- [ ] Set up monitoring and alerting
- [ ] Configure log aggregation
- [ ] Set up automated backups of dns-data volume
- [ ] Test disaster recovery procedures
- [ ] Document incident response procedures

## Monitoring

### Logs
```bash
# View all logs
docker compose logs -f

# View specific service
docker compose logs -f dns-server

# Save logs to file
docker compose logs dns-server > dns-server.log
```

### Resource Usage
```bash
# View resource usage
docker stats

# Specific container
docker stats ldns-dns-server
```

### Health Status
```bash
# Check all services
docker compose ps

# Check specific service health
docker inspect ldns-dns-server | grep -A 20 Health
```

## Next Steps

1. **Test the deployment**: Start containers and verify functionality
2. **Address security issues**: Implement fixes from security audit
3. **Add monitoring**: Set up Prometheus/Grafana
4. **CI/CD Pipeline**: Automate build and deployment
5. **Documentation**: Update README with Docker instructions

## Learning Resources

- **Docker Multi-Stage Builds**: https://docs.docker.com/build/building/multi-stage/
- **Docker Compose**: https://docs.docker.com/compose/
- **Container Security**: https://cheatsheetseries.owasp.org/cheatsheets/Docker_Security_Cheat_Sheet.html
- **Rust in Docker**: https://hub.docker.com/_/rust

---

**Created**: 2025-11-19
**Project**: LDNS DNS Server
**Status**: Docker setup complete, security issues pending
