use std::net::IpAddr;
use tokio::net::lookup_host;

pub async fn resolve_DomainNmae(domainName: &String) -> std::io::Result<Vec<IpAddr>> {
    ///lookup the ip of domain name
    let addrs = lookup_host(&domainName).await?;

    /// adding the recived data in a logical way
    let ips: Vec<IpAddr> = addrs.map(|x| x.ip()).collect();

    return ips;
}
