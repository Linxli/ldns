use std::net::IpAddr;
use tokio::net::lookup_host;

pub async fn resolve_domain(domain_name: &str) -> std::io::Result<Vec<IpAddr>> {
    //lookup the ip of domain name
    println!("starting to do lookup");
    match lookup_host((domain_name.trim_end_matches('.'), 0)).await {
        Ok(addrs) => {
            // adding the recived data in a logical way
            let ips: Vec<IpAddr> = addrs.map(|x| x.ip()).collect();
            println!("{:?}", ips);
            Ok(ips)
        }
        Err(e) => {
            eprint!("DNS lookup failed, reason .. find it out yourself: {} ", e);
            Err(e)
        }
    }
}
