use hickory_proto::rr::domain::Name;
use reqwest::{Url, get};
use std::fs::read_to_string;
use tokio::fs::write;
use tokio::time;

const DNS_LIST: &str = "/Users/fabio/ldns/dnsblock.txt";

pub fn check_dn_block_list(qname: Name) -> bool {
    //let is_blocked = AtomicBool::new(false);
    // return is_blocked.load(Ordering::Relaxed);
    let name = qname.to_string();

    let content =
        read_to_string(DNS_LIST).expect("an access Error happend to the file PATH {dns_list}!!!");
    let is_blocked: bool = content
        .lines()
        .filter(|line| line.starts_with("||") && !line.starts_with("||["))
        .map(|line| {
            line.trim_start_matches("||")
                .trim_end_matches("^")
                .to_string()
        })
        .any(|dom| dom.matches(&name.trim_end_matches(".")).count() > 0);

    is_blocked
}

pub async fn check_blocklist_update(time: u64) {
    let web_addr =
        "https://gitlab.com/hagezi/mirror/-/raw/main/dns-blocklists/adblock/ultimate.txt";

    loop {
        match get(web_addr).await {
            Ok(resp) => {
                if resp.status().is_success() {
                    match resp.text().await {
                        Ok(body) => {
                            if let Err(e) = write(DNS_LIST, body.clone()).await {
                                eprintln!("Error writing to file: {}", e);
                            } else {
                                println!("blocklist has been updated!")
                            }
                        }
                        Err(e) => eprint!("error parsing responese to text: {}", e),
                    }
                } else {
                    eprint!("Failed to access the websit, no 200: {}", resp.status());
                }
            }
            Err(e) => eprint!("Failed to fetch blocklist: {}", e),
        };

        time::sleep(time::Duration::from_secs(time * 3600)).await;
    }
}
