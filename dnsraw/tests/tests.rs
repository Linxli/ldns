// tests/integration_test.rs
use dnsraw;

#[test]
fn test_resolve_integration() {
    assert_eq!(dnsraw::resolve("srf.ch"), "89.106.200.1");
}

#[test]
fn test_blocking_true() {
    assert_eq!(dnsraw::blocklookup::check_dn_block_list("img.web.de"), true);
}

#[test]
fn test_blocking_false() {
    assert_eq!(dnsraw::blocklookup::check_dn_block_list("srf.ch"), false);
}
