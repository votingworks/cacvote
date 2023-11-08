use auth_rs::CardReader;

fn main() {
    tracing::subscriber::set_global_default(
        tracing_subscriber::FmtSubscriber::builder()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .finish(),
    )
    .unwrap();

    tracing::debug!("Starting watch example");

    let mut card_reader = CardReader::new().unwrap();
    let watcher = card_reader.watch();

    watcher.receiver().iter().for_each(|event| {
        println!("Event: {:?}", event);
    });

    // for _ in 0..10 {
    //     println!("Poll Update: Status: {:?}", status.get());
    //     thread::sleep(Duration::from_secs(1));
    // }

    // watcher.stop();
}
