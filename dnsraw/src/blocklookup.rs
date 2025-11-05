use hickory_proto::rr::domain::Name;
use lazy_static::lazy_static;
use reqwest::header::{ETAG, IF_NONE_MATCH, LAST_MODIFIED, USER_AGENT};
use std::fs::read_to_string;
use std::sync::Arc;
use tokio::fs;
use tokio::sync::RwLock;
use tokio::time;

//static BLOCKLIST: OnceCell<String> = OnceCell::new();
const DNS_LIST: &str = "/Users/fabio/ldns/dnsblock.txt";
const DNS_LIST_URL: &str =
    "https://gitlab.com/hagezi/mirror/-/raw/main/dns-blocklists/adblock/ultimate.txt";
lazy_static! {
    static ref FILE_BLOCK_DATA: Arc<RwLock<String>> = Arc::new(RwLock::new(String::new()));
}

pub async fn load_file() {
    let mut data = FILE_BLOCK_DATA.write().await;
    *data = read_to_string(DNS_LIST).expect("Error at loading the blocklist!!!");
}

pub async fn check_dn_block_list(qname: Name) -> bool {
    let name = qname.to_string();
    let content = FILE_BLOCK_DATA.read().await.to_string();
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
                        load_file().await; // loading the data into the global string
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
