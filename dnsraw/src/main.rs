mod resolver;
mod udplistener;
use udplistener::listener;

#[tokio::main]
async fn main() {
    let traffic = "start of the program";
    println!("{:?}", traffic);

    listener().await;
}
