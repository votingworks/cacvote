use auth_rs::{CardReader, Event, Watcher};

fn main() {
    let ctx = pcsc::Context::establish(pcsc::Scope::User).unwrap();
    let mut watcher = Watcher::watch();
    let mut card_reader: Option<CardReader> = None;

    println!("Insert a card to read its status…");

    for event in watcher.events() {
        match event {
            Ok(Event::CardInserted { reader_name }) => {
                card_reader = Some(CardReader::new(ctx.clone(), reader_name));
                break;
            }
            Err(error) => {
                eprintln!("Error: {:?}", error);
                break;
            }
            _ => {}
        }
    }

    if let Some(card_reader) = card_reader {
        watcher.stop();
        println!("{:#?}", card_reader.read_card_details());
    }
}
