use std::net::IpAddr;
use tokio::net::lookup_host;

pub async fn resolve_domain(domain_name: &String) -> std::io::Result<Vec<IpAddr>> {
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

#[allow(dead_code)]
pub fn get_ip(addrs: Vec<IpAddr>) -> Vec<u8> {
    let bytes: Vec<u8> = addrs
        .into_iter()
        .flat_map(|ip| match ip {
            IpAddr::V4(ipv4) => ipv4.octets().to_vec(),
            IpAddr::V6(ipv6) => ipv6.octets().to_vec(),
        })
        .collect();
    println!("IPs get disected {:?}", &bytes[..]);
    return bytes;
}
