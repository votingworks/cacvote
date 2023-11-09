use std::fmt::Debug;
use std::ops::Deref;
use std::sync::{Arc, Mutex};

use auth_rs::card_details::CardDetails;
use auth_rs::{CardReader, SharedCardReaders, Watcher};
use types_rs::rave::jx::SmartcardStatus;

/// Watches for smartcard events.
pub fn watch() -> Watcher {
    Watcher::watch()
}

/// Provides access to the current smartcard status.
#[derive(Clone)]
pub(crate) struct StatusGetter {
    ctx: pcsc::Context,
    readers_with_cards: SharedCardReaders,
    last_selected_card_reader_info: Arc<Mutex<Option<(String, CardDetails)>>>,
}

impl Debug for StatusGetter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StatusGetter")
            .field("readers_with_cards", &self.readers_with_cards)
            .field(
                "last_selected_card_reader_info",
                &self.last_selected_card_reader_info,
            )
            .finish()
    }
}

impl StatusGetter {
    pub(crate) fn new(readers_with_cards: SharedCardReaders) -> Self {
        Self {
            ctx: pcsc::Context::establish(pcsc::Scope::User).unwrap(),
            readers_with_cards,
            last_selected_card_reader_info: Arc::new(Mutex::new(None)),
        }
    }

    /// Gets the current smartcard status.
    pub(crate) fn get(&self) -> SmartcardStatus {
        let readers = self.readers_with_cards.lock().unwrap();

        if readers.is_empty() {
            SmartcardStatus::NoCard
        } else {
            SmartcardStatus::Card
        }
    }

    pub(crate) fn get_card_details(&self) -> Option<CardDetails> {
        let readers = self.readers_with_cards.lock().unwrap();

        let mut cached_card_details = self.last_selected_card_reader_info.lock().unwrap();
        if let Some((ref name, ref details)) = cached_card_details.deref() {
            let same_reader_has_card = readers.iter().any(|reader| reader == name);

            if same_reader_has_card {
                return Some(details.clone());
            }
        }

        *cached_card_details = match readers {
            ref readers if readers.is_empty() => None,
            ref readers => {
                let name = readers.first().unwrap();
                let reader = CardReader::new(self.ctx.clone(), name.clone());
                let card_details = reader.read_card_details().unwrap();
                Some((name.to_string(), card_details))
            }
        };

        cached_card_details.clone().map(|(_, details)| details)
    }
}
