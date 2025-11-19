/*!
Library entry point for the `dnsraw` crate.

This exposes convenient, synchronous wrappers for integration tests and re-exports
selected async functionality from internal modules.
*/

use std::net::IpAddr;

#[path = "resolver.rs"]
mod resolver;

#[path = "blocklookup.rs"]
mod blocklookup_impl;

// Expose API module for testing
pub mod api;

/// Resolve a domain and return a best-effort single IP address as a string.
///
/// Behavior:
/// - Prefers the first IPv4 address if available; otherwise returns the first address found.
/// - Returns an empty string if resolution fails.
///
/// Note: This function creates a Tokio runtime internally to call the async resolver.
pub fn resolve(domain: &str) -> String {
    let rt = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");
    let addrs: Vec<IpAddr> = match rt.block_on(resolver::resolve_domain(domain)) {
        Ok(v) => v,
        Err(_) => return String::new(),
    };

    // Prefer IPv4 if present, otherwise return the first IP
    if let Some(v4) = addrs.iter().find_map(|ip| match ip {
        IpAddr::V4(v4) => Some(v4.to_string()),
        _ => None,
    }) {
        v4
    } else {
        addrs.first().map(|ip| ip.to_string()).unwrap_or_default()
    }
}

/// Synchronous-facing `blocklookup` module for tests.
///
/// Re-exports the `load_file` and `check_blocklist_update` functions, and provides a
/// sync wrapper `check_dn_block_list(&str) -> bool` for convenience in tests.
pub mod blocklookup {
    pub use super::blocklookup_impl::{check_blocklist_update, load_file};

    use super::blocklookup_impl;
    use hickory_proto::rr::domain::Name;

    /// Synchronous wrapper around the async blocklist check.
    ///
    /// - Returns `true` if the provided domain matches the block list,
    ///   `false` otherwise (including when parsing the domain fails).
    /// - Ensures the block list file is loaded before the check.
    pub fn check_dn_block_list(domain: &str) -> bool {
        let rt = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");
        rt.block_on(async {
            // Load the local file (doesn't trigger download)
            let _ = blocklookup_impl::load_file(None).await;

            let name = match Name::from_ascii(domain) {
                Ok(n) => n,
                Err(_) => return false,
            };

            blocklookup_impl::check_dn_block_list(name).await
        })
    }
}
