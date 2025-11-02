use hickory_proto::rr::domain::Name;
use reqwest::header::{ETAG, IF_NONE_MATCH, LAST_MODIFIED, USER_AGENT};
use std::fs::read_to_string;
use tokio::fs;
use tokio::time;

//static BLOCKLIST: OnceCell<String> = OnceCell::new();
const DNS_LIST: &str = "/Users/fabio/ldns/dnsblock.txt";
const DNS_LIST_URL: &str =
    "https://gitlab.com/hagezi/mirror/-/raw/main/dns-blocklists/adblock/ultimate.txt";

// fn load_block_list() -> Result<(), Box<dyn std::error::Error>> {
//     let content = read_to_string(DNS_LIST)?;
//     BLOCKLIST.set(content).map_err(|_| "Blocklist ist already loaded")?;
//     Ok(())
// }

pub fn check_dn_block_list(qname: Name) -> bool {
    let name = qname.to_string();
    let content = read_to_string(DNS_LIST).expect("Error at loading the blocklist!!!"); // BLOCKLIST.get().expect("Blocklist ist not loaded");

    content
        .lines()
        .filter(|line| line.starts_with("||") && !line.starts_with("||["))
        .map(|line| {
            line.trim_start_matches("||")
                .trim_end_matches("^")
                .to_string()
        })
        .any(|dom| name.trim_end_matches(".").contains(&dom))
}

pub async fn check_blocklist_update(interval_hours: u64) {
    println!("Starting blocklist update checker...");
    let client = match reqwest::Client::builder()
        .user_agent("ldns-block-list-updater/1.5")
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to build HTTP client: {}", e);
            return;
        }
    };

    let mut etag: Option<String> = None;
    let mut last_modified: Option<String> = None;

    loop {
        let mut req = client
            .get(DNS_LIST_URL)
            .header(USER_AGENT, "ldns-block-list-updater/1.5");

        if let Some(v) = &etag {
            req = req.header(IF_NONE_MATCH, v);
        }
        if let Some(v) = &last_modified {
            req = req.header(LAST_MODIFIED, v);
        }

        match req.send().await {
            Ok(resp) if resp.status().is_success() => {
                if let Some(v) = resp.headers().get(ETAG).and_then(|v| v.to_str().ok()) {
                    etag = Some(v.to_string());
                }
                if let Some(v) = resp
                    .headers()
                    .get(LAST_MODIFIED)
                    .and_then(|v| v.to_str().ok())
                {
                    last_modified = Some(v.to_string());
                }

                match resp.bytes().await {
                    Ok(bytes) => {
                        let tmp = format!("{}.tmp", DNS_LIST);
                        if let Err(e) = fs::write(&tmp, &bytes).await {
                            eprintln!("Failed to write temp file: {}", e);
                        } else if let Err(e) = fs::rename(&tmp, DNS_LIST).await {
                            eprintln!("Failed to rename temp file: {}", e);
                        } else {
                            println!("Blocklist updated successfully!");
                        }
                    }
                    Err(e) => eprintln!("Failed to read response body: {}", e),
                }
            }
            Ok(resp) if resp.status() == reqwest::StatusCode::NOT_MODIFIED => {
                println!("Blocklist is up to date.");
            }
            // for every other case of return value
            Ok(resp) => {
                eprintln!("HTTP request failed with status: {}", resp.status());
            }
            Err(e) => {
                eprintln!("HTTP request failed: {}", e);
            }
        }

        time::sleep(time::Duration::from_secs(interval_hours * 3600)).await;
    }
}
