use auth_rs::CardReader;

fn main() {
    let mut card_reader = CardReader::new().unwrap();
    let watcher = card_reader.watch();

    for event in watcher.receiver() {
        println!("{:?}", event);
    }
}
