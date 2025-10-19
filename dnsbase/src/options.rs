use clap::Parser;
use std::net::SocketAddr;

#[derive(Parser, Clone, Debug)]
pub struct Options {
    /// UDP socket to listen.
    #[clap(long, short, default_value = "0.0.0.0:1053", env = "DNSBASE_UDP")]
    pub udp: Vec<SocketAddr>,

    ///TCP socket to listen on:
    #[clap(long, short, env = "DNSBASE_TCP")]
    pub tcp: Vec<SocketAddr>,

    /// Domain Name
    #[clap(long, short, default_value = "dnsbase.dev", env = "DNSBASE_DOMAIN")]
    pub domain: String,
}
