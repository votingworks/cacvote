use std::process;

#[tokio::main]
async fn main() {
    let mut watcher = auth_rs::Watcher::watch();

    while let Some(event) = watcher.recv().await {
        println!("{event:?}");
        println!("readers with cards: {:?}", watcher.readers_with_cards());
    }

    // FIXME: why does tokio not exit on its own?
    // Try something from https://tokio.rs/tokio/topics/shutdown#waiting-for-things-to-finish-shutting-down
    process::exit(0);
}
