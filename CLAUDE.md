# CLAUDE.md - AI Assistant Guide for LDNS Project

> **Last Updated:** 2025-11-23
> **Project:** LDNS - High-Performance DNS Server with Blocklist Support
> **Language:** Rust (Edition 2024 for dnsraw, 2021 for ldnsTUI)
> **Status:** Production-ready with containerized deployment

---

## Table of Contents

1. [Project Overview](#project-overview)
2. [Codebase Structure](#codebase-structure)
3. [Development Environment](#development-environment)
4. [Architecture & Key Decisions](#architecture--key-decisions)
5. [Coding Conventions](#coding-conventions)
6. [Testing Strategy](#testing-strategy)
7. [Build & Deployment](#build--deployment)
8. [Git Workflow](#git-workflow)
9. [AI Assistant Guidelines](#ai-assistant-guidelines)

---

## Project Overview

### What is LDNS?

LDNS is a high-performance DNS server written in Rust that provides:

- **Real-time Domain Blocking**: Ad/tracker blocking using customizable blocklists
- **RESTful API**: Runtime configuration and blocklist management
- **Terminal UI**: Interactive TUI for server management
- **Zero Downtime Updates**: Blocklist reloads without service interruption
- **Docker Support**: Fully containerized deployment with Docker Compose

### Technology Stack

| Component | Technology | Version | Purpose |
|-----------|-----------|---------|---------|
| DNS Server | `hickory-proto` | 0.25.2 | DNS protocol implementation |
| DNS Resolution | `hickory-resolver` | 0.25.2 | Upstream DNS queries |
| HTTP API | `actix-web` | 4.11.0 | REST API framework |
| TUI Framework | `ratatui` | 0.29 | Terminal user interface |
| Terminal Control | `crossterm` | 0.28 | Cross-platform terminal manipulation |
| Async Runtime | `tokio` | 1.48.0 | Async/await executor |
| HTTP Client | `reqwest` | 0.11 | Download blocklists |
| Serialization | `serde` / `serde_json` | 1.0 | JSON handling |
| Testing | `mockito` | 1.6 | HTTP mocking for tests |

### Project Goals

1. **Performance**: O(1) domain lookups using HashSet (1000x faster than linear search)
2. **Safety**: Memory-safe Rust with no data races
3. **Reliability**: Comprehensive test coverage with CI/CD
4. **Usability**: Both API and TUI interfaces for management
5. **Deployability**: Production-ready Docker containers

---

## Codebase Structure

### Directory Layout

```
ldns/
├── dnsraw/                         # Main DNS server application
│   ├── src/
│   │   ├── main.rs                # Entry point - starts UDP and HTTP servers
│   │   ├── lib.rs                 # Public library interface
│   │   ├── api.rs                 # REST API endpoints (PUT /blocklist)
│   │   ├── blocklookup.rs         # Core blocklist logic (HashSet-based)
│   │   ├── resolver.rs            # Upstream DNS resolver (forwards queries)
│   │   └── udplistener.rs         # UDP DNS listener (port 53)
│   ├── tests/
│   │   ├── api_tests.rs           # API integration tests
│   │   └── tests.rs               # DNS functionality tests
│   ├── Cargo.toml                 # Dependencies and build config
│   ├── Cargo.lock                 # Locked dependency versions
│   ├── Dockerfile                 # Multi-stage Docker build
│   └── Makefile.toml              # Build automation (cargo-make)
│
├── ldnsTUI/                        # Terminal UI application
│   ├── src/
│   │   └── main.rs                # TUI event loop and rendering
│   ├── Cargo.toml
│   └── Dockerfile
│
├── .github/
│   └── workflows/
│       └── test-rust.yml          # CI/CD pipeline (build, test, lint, coverage)
│
├── compose.yaml                   # Docker Compose orchestration
├── Dockerfile                     # Root-level Docker build (dnsraw)
├── dnsblock.txt                   # Default blocklist (233k+ domains)
│
├── DNS_SERVER_GUIDE.md            # Comprehensive technical guide
├── DOCKER_SETUP.md                # Docker deployment guide
├── QUICK_REFERENCE.md             # Quick command reference
├── BUILD_ISSUES_AND_SOLUTIONS.md  # Troubleshooting guide
├── PRESET_FEATURE_GUIDE.md        # Feature documentation
│
└── CLAUDE.md                      # This file (AI assistant guide)
```

### Module Responsibilities

#### dnsraw/src/main.rs
- **Purpose**: Application entry point
- **Responsibilities**:
  - Initialize blocklist from file
  - Spawn UDP DNS listener on port 53
  - Spawn HTTP API server on port 8080
  - Use tokio::join! for concurrent servers

#### dnsraw/src/blocklookup.rs
- **Purpose**: Core blocklist management
- **Key Components**:
  - `BLOCKLIST`: Global `Arc<RwLock<HashSet<String>>>` for thread-safe access
  - `load_file(Option<Vec<u8>>)`: Parse and load blocklist (None = read from disk)
  - `is_blocked(&str)`: O(1) domain lookup
- **Performance**: HashSet provides constant-time lookups vs O(n) string search

#### dnsraw/src/api.rs
- **Purpose**: REST API for blocklist management
- **Endpoints**:
  - `PUT /blocklist`: Update blocklist URL, download, and reload
- **Error Handling**: Layered validation (URL format → network → parsing)
- **Response Types**: JSON with detailed error messages

#### dnsraw/src/udplistener.rs
- **Purpose**: DNS protocol handler
- **Responsibilities**:
  - Listen on UDP port 53
  - Parse DNS queries
  - Check blocklist (return NXDomain if blocked)
  - Forward to upstream resolver if not blocked
  - Return response to client

#### dnsraw/src/resolver.rs
- **Purpose**: Upstream DNS resolution
- **Uses**: `hickory_resolver` for standard DNS lookups
- **Upstream**: Configurable (default: 1.1.1.1 Cloudflare DNS)

#### ldnsTUI/src/main.rs
- **Purpose**: Terminal UI for management
- **Architecture**: Event loop (render → input → update → repeat)
- **Screens**: Home, EditBlocklist
- **Input Mode**: Modal (like vim) - normal vs insert mode
- **API Communication**: Calls REST API endpoints via reqwest

---

## Development Environment

### Prerequisites

```bash
# Rust toolchain
rustc --version  # Should be 1.88.0 or newer
cargo --version

# Required for dnsraw
rustfmt  # Code formatting
clippy   # Linting

# Optional but recommended
docker   # For containerized deployment
```

### Local Development Setup

```bash
# Clone repository
git clone <repo-url>
cd ldns

# Build dnsraw (DNS server)
cd dnsraw
cargo build
cargo test

# Run dnsraw locally (requires sudo for port 53)
sudo cargo run

# In another terminal, build and run TUI
cd ../ldnsTUI
cargo build
cargo run
```

### Environment Variables

#### DNS Server (dnsraw)
```bash
LOG_LEVEL=info          # Logging level (debug, info, warn, error)
UPSTREAM_DNS=1.1.1.1    # Upstream DNS server
RUST_LOG=info           # Rust logging configuration
```

#### TUI (ldnsTUI)
```bash
DNS_API_URL=http://localhost:8080  # API endpoint (use service name in Docker)
RUST_LOG=info                       # Logging level
```

### Docker Development

```bash
# Start DNS server (foreground)
docker compose up --build

# Start DNS server (background)
docker compose up -d

# Run TUI (requires DNS server running)
docker compose run --rm tui

# View logs
docker compose logs -f dns-server

# Stop services
docker compose down
```

---

## Architecture & Key Decisions

### 1. Data Structure: HashSet vs String

**Decision**: Use `HashSet<String>` for blocklist storage

**Rationale**:
- **Performance**: O(1) lookup vs O(n) string scanning
- **Benchmark**: 5μs vs 5ms per lookup (1000x faster)
- **Memory**: No string copying per query (687,500x less memory per query)
- **Scalability**: Performance independent of blocklist size (233k+ domains)

**Trade-off**: ~20-30% more memory for hash table overhead (acceptable)

**Implementation**:
```rust
lazy_static! {
    static ref BLOCKLIST: Arc<RwLock<HashSet<String>>> =
        Arc::new(RwLock::new(HashSet::new()));
}

// Usage
let blocklist = BLOCKLIST.read().await;
if blocklist.contains(&domain) { /* blocked */ }
```

### 2. Concurrency Pattern: Arc<RwLock<T>>

**Decision**: Use `Arc<RwLock<HashSet<String>>>` for shared state

**Components**:
- **Arc** (Atomic Reference Counted): Thread-safe shared ownership
- **RwLock** (Read-Write Lock): Many readers OR one writer (not both)

**Rationale**:
- **DNS queries**: Multiple concurrent reads (shared lock)
- **API updates**: Rare exclusive writes (exclusive lock)
- **Zero downtime**: Reads continue during brief write lock

**Alternative Considered**: `Mutex<T>` - Rejected (only allows one reader at a time)

**Critical**: Hold locks for minimal time
```rust
// ✓ GOOD: Prepare data outside lock
let new_data = expensive_operation();
let mut lock = BLOCKLIST.write().await;
*lock = new_data;  // Brief lock

// ✗ BAD: Long-running work while holding lock
let mut lock = BLOCKLIST.write().await;
expensive_operation();  // Blocks all DNS queries!
*lock = new_data;
```

### 3. API Parameter Design: Option<Vec<u8>>

**Decision**: Use `Option<Vec<u8>>` for `load_file()`

**Problem**: Distinguish "no data" from "empty data"

**Evolution**:
```rust
// v1 (BROKEN): Empty vec treated as "read from disk"
fn load_file(bytes: Vec<u8>)

// v2 (CORRECT): Type system enforces distinction
fn load_file(bytes: Option<Vec<u8>>)
```

**Semantics**:
- `None`: Read blocklist from disk file
- `Some(vec![])`: Use empty blocklist (clear all blocks)
- `Some(data)`: Use provided data

**Lesson**: Use Rust's type system to make ambiguous cases explicit

### 4. HTTP Method Semantics

**Decision**: Use `PUT /blocklist` (not POST)

**Rationale**:
- **Idempotent**: Multiple identical requests = same result
- **Replacing**: We replace entire blocklist URL, not adding to collection
- **RESTful**: PUT = update/replace resource

**Response Design**:
```json
{
  "message": "Blocklist updated successfully",
  "url": "https://example.com/list.txt",
  "domains_loaded": 233382
}
```
- Includes `domains_loaded` for immediate validation feedback
- Descriptive error messages for debugging

---

## Coding Conventions

### Rust Style Guidelines

Follow official Rust conventions enforced by CI:

```bash
# Format code (CI requirement)
cargo fmt

# Lint with Clippy (CI fails on warnings)
cargo clippy -- -D warnings

# Check both in one command
cargo fmt && cargo clippy -- -D warnings
```

### Code Organization

#### Module Structure
```rust
// ===== IMPORTS =====
use std::collections::HashSet;
use tokio::sync::RwLock;

// ===== TYPES =====
#[derive(Deserialize, Serialize)]
struct Request { ... }

// ===== CONSTANTS =====
const DNS_PORT: u16 = 53;

// ===== GLOBAL STATE =====
lazy_static! { ... }

// ===== PUBLIC API =====
pub async fn public_function() { ... }

// ===== PRIVATE HELPERS =====
async fn helper() { ... }
```

#### Naming Conventions
```rust
// Types: PascalCase
struct BlocklistUpdateRequest { ... }

// Functions: snake_case
async fn update_blocklist() { ... }

// Constants: SCREAMING_SNAKE_CASE
const MAX_RETRIES: u32 = 3;

// Module files: snake_case
blocklookup.rs
api.rs
```

### Documentation

**All public items MUST have doc comments**:
```rust
/// Updates the blocklist from a remote URL
///
/// # Arguments
/// * `url` - The URL to download the blocklist from
///
/// # Returns
/// Number of domains loaded into the blocklist
///
/// # Errors
/// Returns error if URL is invalid or download fails
///
/// # Example
/// ```
/// let count = update_blocklist("https://example.com/list.txt").await?;
/// println!("Loaded {} domains", count);
/// ```
pub async fn update_blocklist(url: &str) -> Result<usize, Error>
```

### Error Handling

**Use Result<T, E> for all fallible operations**:
```rust
// ✓ GOOD: Explicit error handling
match risky_operation() {
    Ok(value) => use_value(value),
    Err(e) => handle_error(e),
}

// ✓ GOOD: Propagate with ?
let value = risky_operation()?;

// ✗ BAD: unwrap() in production code
let value = risky_operation().unwrap();  // Crashes on error!

// ✓ OK: unwrap() in tests
#[test]
fn test_something() {
    let value = operation().unwrap();  // Test failure is ok
}
```

**Error message quality**:
```rust
// ✗ BAD: Generic error
"Error occurred"

// ✓ GOOD: Specific and actionable
"Failed to download blocklist from 'https://example.com': connection timeout after 30s"
```

### Async/Await

**Use async/await for I/O operations**:
```rust
// Network I/O, file I/O, etc.
async fn download_blocklist(url: &str) -> Result<Vec<u8>> {
    let response = reqwest::get(url).await?;
    let bytes = response.bytes().await?;
    Ok(bytes.to_vec())
}

// CPU-bound work: No async needed
fn parse_blocklist(data: &str) -> HashSet<String> {
    data.lines()
        .filter(|line| line.starts_with("||"))
        .map(|line| line.trim_matches('|').trim_end_matches('^').to_string())
        .collect()
}
```

### Performance Guidelines

1. **Avoid allocations in hot paths** (DNS query handling)
   ```rust
   // ✗ BAD: Allocates string on every DNS query
   let name = qname.to_string();

   // ✓ GOOD: Borrow when possible
   let name: &str = qname.as_ref();
   ```

2. **Choose appropriate data structures**
   - Fast lookup: `HashSet<T>`, `HashMap<K, V>`
   - Ordered: `BTreeMap<K, V>`, `BTreeSet<T>`
   - Small collections: `Vec<T>` (cache-friendly)
   - Queue: `VecDeque<T>`

3. **Minimize lock hold time**
   ```rust
   // Prepare data outside lock
   let parsed = parse_blocklist(data);

   // Hold lock only for swap
   let mut blocklist = BLOCKLIST.write().await;
   *blocklist = parsed;  // Minimal time
   ```

---

## Testing Strategy

### Test Organization

```
dnsraw/
├── src/
│   └── *.rs              # Implementation
└── tests/
    ├── tests.rs          # Unit/integration tests for DNS functionality
    └── api_tests.rs      # API integration tests
```

### Running Tests

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test file
cargo test --test api_tests

# Run single test
cargo test test_update_blocklist_success

# Run with coverage
cargo tarpaulin --skip-clean --out Xml
```

### Test Categories

#### 1. Unit Tests (in src files)
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_domain() {
        let result = parse_domain("||example.com^");
        assert_eq!(result, "example.com");
    }
}
```

#### 2. Integration Tests (in tests/ directory)
```rust
#[tokio::test]
async fn test_blocklist_loading() {
    let data = "||ads.com^\n||tracker.net^";
    let count = load_file(Some(data.as_bytes().to_vec())).await;
    assert_eq!(count, 2);
}
```

#### 3. API Tests with Mocking
```rust
#[tokio::test]
async fn test_update_blocklist_success() {
    // Create mock HTTP server
    let mut server = mockito::Server::new_async().await;

    server.mock("GET", "/blocklist.txt")
        .with_status(200)
        .with_body("||ads.com^")
        .create_async()
        .await;

    // Test API endpoint
    let response = update_blocklist(server.url()).await;
    assert!(response.is_ok());
}
```

### Coverage Goals

- **Minimum**: 80% line coverage
- **Target**: 100% (currently achieved)
- **CI Requirement**: Tests must pass on all PRs

### Test-Driven Development (TDD)

**Cycle**: Red → Green → Refactor

1. **Write test** (fails - no implementation)
2. **Implement** (minimal code to pass)
3. **Refactor** (improve quality)
4. **Repeat**

**Example**: Empty blocklist test found Option<Vec<u8>> bug

---

## Build & Deployment

### Local Build

```bash
# Debug build (fast compilation, slower runtime)
cargo build

# Release build (optimized)
cargo build --release

# Run release binary
./target/release/dnsraw
```

### Release Profile Optimizations

From `dnsraw/Cargo.toml`:
```toml
[profile.release]
strip = true           # Remove debug symbols (smaller binary)
lto = true            # Link-time optimization (faster runtime)
opt-level = "z"       # Optimize for size
codegen-units = 1     # Better optimization (slower compile)
```

### Docker Build

#### Multi-Stage Build (dnsraw)
```dockerfile
# Stage 1: Build
FROM rust:1.88.0-alpine AS build
# ... build with cargo ...

# Stage 2: Runtime
FROM alpine:3.18 AS final
# ... minimal runtime image ...
```

**Benefits**:
- **Small image**: Only runtime dependencies (no build tools)
- **Security**: Non-root user, minimal attack surface
- **Performance**: Optimized release binary

#### Docker Compose Orchestration

```bash
# Start DNS server only
docker compose up dns-server

# Start with build
docker compose up --build

# Run TUI interactively
docker compose run --rm tui

# View logs
docker compose logs -f

# Stop all services
docker compose down

# Remove volumes
docker compose down -v
```

### Container Security

From `compose.yaml`:
```yaml
security_opt:
  - no-new-privileges:true  # Prevent privilege escalation
cap_add:
  - NET_BIND_SERVICE        # Allow binding to port 53
cap_drop:
  - ALL                     # Drop all other capabilities
```

### Port Configuration

| Port | Protocol | Service | Exposed |
|------|----------|---------|---------|
| 53 | UDP | DNS queries | Yes (0.0.0.0:53) |
| 8080 | TCP | REST API | Yes (0.0.0.0:8080) |

**Note**: Port 53 requires root or `NET_BIND_SERVICE` capability

---

## Git Workflow

### Branch Strategy

- **main**: Production-ready code
- **claude/**: AI-generated feature branches (auto-managed)
- Feature branches: Short-lived, merged via PR

### Commit Message Format

```
<type>: <subject>

<body>

<footer>
```

**Types**:
- `feat`: New feature
- `fix`: Bug fix
- `refactor`: Code restructuring
- `test`: Add/modify tests
- `docs`: Documentation
- `chore`: Build/tooling changes

**Examples**:
```
feat: add HashSet-based blocklist for O(1) lookups

Replaced linear string search with HashSet for 1000x performance
improvement. Blocklist queries now complete in ~5μs vs 5ms.

Closes #42

---

fix: handle empty blocklist in load_file

Changed parameter from Vec<u8> to Option<Vec<u8>> to distinguish
between "no data" and "empty data".

Fixes #58
```

### Pull Request Process

1. **Create branch** from main
   ```bash
   git checkout -b feature/my-feature
   ```

2. **Make changes** with tests
   ```bash
   # Edit code
   cargo test
   cargo fmt
   cargo clippy -- -D warnings
   ```

3. **Commit** with clear messages
   ```bash
   git add .
   git commit -m "feat: add new feature"
   ```

4. **Push** to remote
   ```bash
   git push -u origin feature/my-feature
   ```

5. **Create PR** via GitHub
   - CI must pass (build, test, lint, format)
   - Request review if needed
   - Merge when approved

### CI/CD Pipeline

From `.github/workflows/test-rust.yml`:

**Triggers**: Push/PR to main branch

**Steps**:
1. **Build**: `cargo build --verbose`
2. **Test**: `cargo test --verbose`
3. **Lint**: `cargo clippy -- -D warnings` (fails on warnings)
4. **Format**: `cargo fmt -- --check` (fails if not formatted)
5. **Coverage**: `cargo tarpaulin` (uploads coverage report)

**Requirements**:
- All steps must pass for PR merge
- Code must be formatted with `rustfmt`
- No clippy warnings allowed

---

## AI Assistant Guidelines

### When Analyzing This Codebase

1. **Understand the architecture first**
   - Read this CLAUDE.md thoroughly
   - Review DNS_SERVER_GUIDE.md for deep technical details
   - Check QUICK_REFERENCE.md for common commands

2. **Respect the performance decisions**
   - HashSet is intentional (not Vec or BTreeSet)
   - Arc<RwLock<T>> is the correct pattern (not Mutex)
   - Don't suggest "simpler" alternatives that sacrifice performance

3. **Follow Rust idioms**
   - Use `?` for error propagation
   - Prefer `match` over `if let` for exhaustive handling
   - Use iterator chains instead of loops where appropriate

4. **Maintain test coverage**
   - Add tests for ALL new functionality
   - Update existing tests when changing behavior
   - Use mockito for external HTTP calls

### When Making Changes

#### Code Modifications

```bash
# ALWAYS before committing
cargo fmt                     # Format code
cargo clippy -- -D warnings   # Lint (must pass)
cargo test                    # All tests must pass
```

#### Adding Features

1. **Plan first**: Understand existing patterns
2. **Write tests**: TDD approach (test first)
3. **Implement**: Follow coding conventions
4. **Document**: Add doc comments for public items
5. **Verify**: Run full test suite + CI checks

#### Refactoring

1. **Run tests first**: Ensure they pass
2. **Make incremental changes**: One refactor at a time
3. **Run tests after each change**: Catch regressions early
4. **Update docs**: Keep comments/docs in sync

#### Performance Changes

1. **Benchmark first**: Measure current performance
2. **Make change**: Implement optimization
3. **Benchmark again**: Verify improvement
4. **Document**: Explain why change was made

### Common Tasks

#### Add New API Endpoint

```rust
// 1. Define request/response types in api.rs
#[derive(Deserialize, Serialize)]
struct NewRequest { ... }

// 2. Implement handler
#[get("/new-endpoint")]
async fn new_endpoint() -> impl Responder {
    HttpResponse::Ok().json(...)
}

// 3. Register in server()
HttpServer::new(|| {
    App::new()
        .service(update_blocklist)
        .service(new_endpoint)  // Add here
})

// 4. Add test in tests/api_tests.rs
#[tokio::test]
async fn test_new_endpoint() { ... }
```

#### Modify Blocklist Logic

```rust
// blocklookup.rs

// ⚠️ CRITICAL: Minimize lock hold time
pub async fn modify_blocklist() {
    // Do expensive work OUTSIDE lock
    let new_data = expensive_operation();

    // Hold lock ONLY for swap
    let mut blocklist = BLOCKLIST.write().await;
    *blocklist = new_data;
    // Lock released here
}
```

#### Add TUI Screen

```rust
// 1. Add to Screen enum
enum Screen {
    Home,
    EditBlocklist,
    NewScreen,  // Add here
}

// 2. Add draw function
fn draw_new_screen(f: &mut Frame, app: &App, area: Rect) {
    // Use ratatui widgets
}

// 3. Add to match in draw_ui()
match app.screen {
    Screen::NewScreen => draw_new_screen(f, app, area),
    // ...
}

// 4. Add key handling in handle_key()
```

### What NOT to Do

1. **Don't use `unwrap()` in production code**
   - Exception: Tests are OK
   - Use `?` or `match` instead

2. **Don't hold locks during I/O**
   ```rust
   // ✗ BAD
   let mut data = LOCK.write().await;
   download_file().await;  // Network I/O while holding lock!
   *data = new_value;
   ```

3. **Don't skip tests**
   - Every feature needs tests
   - Every bug fix needs a regression test

4. **Don't bypass CI checks**
   - Format, lint, test must all pass
   - No "will fix later" commits

5. **Don't change dependencies without discussion**
   - Dependency updates affect security and build time
   - Document why a dependency is needed

### Debugging Tips

#### DNS Server Issues

```bash
# Check port 53 is available
lsof -i :53

# Run with logging
RUST_LOG=debug cargo run

# Test DNS query
dig @127.0.0.1 example.com

# Test with blocked domain
dig @127.0.0.1 ads.com  # Should return NXDomain
```

#### API Issues

```bash
# Test API directly
curl -X PUT http://localhost:8080/blocklist \
  -H "Content-Type: application/json" \
  -d '{"url":"https://example.com/list.txt"}'

# Check API is listening
lsof -i :8080

# View API logs
cargo run 2>&1 | grep api
```

#### TUI Issues

```bash
# Check API URL
echo $DNS_API_URL

# Test API connection
curl $DNS_API_URL/blocklist

# Reset terminal if corrupted
reset
```

#### Test Failures

```bash
# Run single failing test with output
cargo test test_name -- --nocapture

# Check for port conflicts
lsof -i :8080  # Tests use port 8080

# Clean and rebuild
cargo clean
cargo test
```

### Documentation to Reference

| Document | When to Read |
|----------|-------------|
| CLAUDE.md | Always start here (this file) |
| DNS_SERVER_GUIDE.md | Deep technical understanding needed |
| QUICK_REFERENCE.md | Quick command lookup |
| DOCKER_SETUP.md | Deployment questions |
| BUILD_ISSUES_AND_SOLUTIONS.md | Build/deployment problems |
| PRESET_FEATURE_GUIDE.md | Feature implementation details |

### Asking for Help

When uncertain about changes:

1. **Read existing code**: Look for similar patterns
2. **Check tests**: See how features are tested
3. **Review docs**: Comprehensive guides exist
4. **Ask specific questions**: Provide context and attempted solutions

### Code Review Checklist

Before submitting changes:

- [ ] Code compiles without warnings
- [ ] All tests pass (cargo test)
- [ ] Code is formatted (cargo fmt)
- [ ] No clippy warnings (cargo clippy -- -D warnings)
- [ ] New code has tests
- [ ] Public items have doc comments
- [ ] No `unwrap()` in production code
- [ ] Lock hold times are minimal
- [ ] Error messages are descriptive
- [ ] Changes documented (if user-facing)

---

## Project-Specific Patterns

### Blocklist Format

```
||domain.com^       # Block domain.com
||ads.example.net^  # Block ads.example.net
```

**Parsing**:
```rust
domain.trim_matches('|')      // Remove leading/trailing |
      .trim_end_matches('^')  // Remove trailing ^
      .to_string()            // Convert to owned String
```

### DNS Response Codes

- **NXDomain**: Domain does not exist (used for blocked domains)
- **NOERROR**: Query succeeded
- **SERVFAIL**: Server failure

### API Authentication

**Current**: None (⚠️ not production-ready for public deployment)

**Future**: Add API key authentication
```rust
// Future implementation
#[derive(Deserialize)]
struct AuthRequest {
    api_key: String,
    // ... other fields
}

async fn validate_api_key(key: &str) -> bool {
    // Validate against configured keys
}
```

---

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2025-11-23 | Initial CLAUDE.md creation with comprehensive guide |

---

## Additional Resources

### Official Documentation
- [Rust Book](https://doc.rust-lang.org/book/)
- [Async Book](https://rust-lang.github.io/async-book/)
- [API Guidelines](https://rust-lang.github.io/api-guidelines/)

### Framework Docs
- [actix-web](https://actix.rs/)
- [ratatui](https://ratatui.rs/)
- [tokio](https://tokio.rs/)
- [hickory-dns](https://github.com/hickory-dns/hickory-dns)

### DNS Standards
- [RFC 1035](https://www.rfc-editor.org/rfc/rfc1035) - DNS Specification

---

**This document should be updated whenever significant architectural changes are made to the codebase.**
