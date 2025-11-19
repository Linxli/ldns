use hickory_proto::rr::domain::Name;
use lazy_static::lazy_static;
use reqwest::header::{ETAG, IF_NONE_MATCH, LAST_MODIFIED, USER_AGENT};
use std::collections::HashSet;
use std::fs::read_to_string;
use std::sync::Arc;
use tokio::fs;
use tokio::sync::RwLock;
use tokio::time;

//static BLOCKLIST: OnceCell<String> = OnceCell::new();
const DNS_LIST: &str = "dnsblock.txt";
const DNS_LIST_URL: &str =
    "https://gitlab.com/hagezi/mirror/-/raw/main/dns-blocklists/adblock/ultimate.txt";
lazy_static! {
    static ref BLOCKLIST: Arc<RwLock<HashSet<String>>> = Arc::new(RwLock::new(HashSet::new()));
}

fn parse_blocklist(raw: String) -> HashSet<String> {
    raw.lines()
        .filter(|line| line.starts_with("||") && !line.starts_with("||["))
        .map(|line| {
            line.trim_start_matches("||")
                .trim_end_matches("^")
                .to_lowercase()
        })
        .collect()
}

/// Loads a blocklist from file or provided bytes
///
/// Returns the number of domains loaded
///
/// Teaching moment: Using Option<Vec<u8>> to distinguish:
/// - None: Read from disk file
/// - Some(bytes): Use these bytes (even if empty!)
pub async fn load_file(bytes: Option<Vec<u8>>) -> usize {
    let raw_content = match bytes {
        Some(data) => {
            // Use the provided bytes (from API download)
            String::from_utf8(data).expect("converting the bytes to utf8 failed")
        }
        None => {
            // No bytes provided - read from disk (for startup)
            match read_to_string(DNS_LIST) {
                Ok(v) => v,
                Err(e) => {
                    eprint!("File loading didn't work, and no bytes provided: {}", e);
                    String::new() // Empty blocklist if nothing available
                }
            }
        }
    };

    // Parse the raw string into a HashSet (heavy work done here!)
    let parsed = parse_blocklist(raw_content);

    // Get the count before moving the data
    let count = parsed.len();

    println!("Loaded {} blocked domains into memory", count);

    // Acquire write lock and update the global blocklist
    // DNS queries wait here (but only for a few milliseconds!)
    let mut blocklist = BLOCKLIST.write().await;
    *blocklist = parsed;
    // Lock automatically released when 'blocklist' goes out of scope

    // Return the count so callers know how many domains were loaded
    count
}

pub async fn check_dn_block_list(qname: Name) -> bool {
    // Normalize the query name (remove trailing dot, lowercase)
    let name = qname.to_string().trim_end_matches(".").to_lowercase();

    // Acquire read lock (many threads can do this simultaneously!)
    let blocklist = BLOCKLIST.read().await;

    // Check if this exact domain is blocked - O(1) lookup!
    if blocklist.contains(&name) {
        return true;
    }

    // Also check if any parent domain is blocked
    // e.g., if "ads.google.com" isn't blocked, check "google.com"
    let parts: Vec<&str> = name.split('.').collect();
    for i in 1..parts.len() {
        let parent = parts[i..].join(".");
        if blocklist.contains(&parent) {
            return true;
        }
    }

    false
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
                        load_file(Some(bytes.clone().to_vec())).await;
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
