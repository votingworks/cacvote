use auth_rs::{CardReader, Event};

fn main() {
    let mut card_reader = CardReader::new().unwrap();
    let watcher = card_reader.watch();

    for event in watcher.receiver() {
        println!("{:?}", event);

        match event.unwrap() {
            Event::CardInserted { reader } => {
                watcher.stop();
                println!("card details: {:?}", card_reader.read_card_details(reader));
            }
            _ => {}
        }
    }
}
