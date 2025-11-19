mod api;
mod blocklookup;
mod resolver;
mod udplistener;
use udplistener::listener;

#[tokio::main]
async fn main() {
    println!("Starting DNS server...");

    // Load the blocklist file on startup (if it exists)
    // This ensures DNS queries work immediately instead of waiting for download
    println!("Loading blocklist from disk...");
    let count = blocklookup::load_file(None).await; // None = read from disk
    println!("Loaded {} domains from blocklist", count);

    // spawn a API web server
    tokio::spawn(api::server());

    // Spawn background task to check for blocklist updates every hour
    tokio::spawn(blocklookup::check_blocklist_update(1));

    println!("DNS server ready!");

    if let Err(e) = listener().await {
        eprintln!("An Error happened: {}", e);
        std::process::exit(1);
    }
}
