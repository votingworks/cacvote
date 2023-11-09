fn main() {
    let watcher = auth_rs::Watcher::watch();

    for event in watcher.events() {
        println!("{:?}", event);
        println!("readers with cards: {:?}", watcher.readers_with_cards());
    }
}
