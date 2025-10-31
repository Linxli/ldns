mod blocklookup;
mod resolver;
mod udplistener;
use udplistener::listener;

#[tokio::main]
async fn main() {
    let traffic = "start of the program";

    //#[allow(unused_must_use)]
    blocklookup::check_blocklist_update(1);

    println!("{:?}", traffic);

    if let Err(e) = listener().await {
        eprint!("An Error happend: {}", e);
        std::process::exit(1);
    }
}
