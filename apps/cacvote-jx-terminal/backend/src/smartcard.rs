use std::fmt::Debug;
use std::ops::Deref;
use std::sync::{Arc, Mutex};

use auth_rs::card_details::CardDetails;
use auth_rs::{CardReader, SharedCardReaders, Watcher};
use types_rs::cacvote::SmartcardStatus;

/// Watches for smartcard events.
pub fn watch() -> Watcher {
    Watcher::watch()
}

/// Provides access to the current smartcard status.
#[derive(Clone)]
pub(crate) struct StatusGetter {
    #[allow(dead_code)]
    ctx: pcsc::Context,
    #[allow(dead_code)]
    readers_with_cards: SharedCardReaders,
    #[allow(dead_code)]
    last_selected_card_reader_info: Arc<Mutex<Option<(String, CardDetails)>>>,
}

impl Debug for StatusGetter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StatusGetter")
            .field("readers_with_cards", &self.readers_with_cards)
            .finish()
    }
}

impl StatusGetter {
    #[allow(dead_code)]
    #[must_use]
    pub(crate) fn new(readers_with_cards: SharedCardReaders) -> Self {
        Self {
            ctx: pcsc::Context::establish(pcsc::Scope::User).unwrap(),
            readers_with_cards,
            last_selected_card_reader_info: Arc::new(Mutex::new(None)),
        }
    }

    /// Gets the current smartcard status.
    #[allow(dead_code)]
    #[must_use]
    pub(crate) fn get(&self) -> SmartcardStatus {
        let readers = self.readers_with_cards.lock().unwrap();

        if readers.is_empty() {
            SmartcardStatus::NoCard
        } else {
            SmartcardStatus::Card
        }
    }

    #[allow(dead_code)]
    #[must_use]
    pub(crate) fn get_card_details(&self) -> Option<CardDetails> {
        let readers = self.readers_with_cards.lock().unwrap();
        let reader_name = readers.first()?;

        let mut cached_card_details = self.last_selected_card_reader_info.lock().unwrap();
        if let Some((ref name, ref details)) = cached_card_details.deref() {
            let same_reader_has_card = readers.iter().any(|reader| reader == name);

            if same_reader_has_card {
                return Some(details.clone());
            }
        }

        let reader = CardReader::new(self.ctx.clone(), reader_name.clone());
        let card_details = match reader.read_card_details() {
            Ok(card_details) => card_details,
            Err(e) => {
                tracing::error!("error reading card details: {e}");
                return None;
            }
        };

        *cached_card_details = Some((reader_name.to_string(), card_details));
        cached_card_details.clone().map(|(_, details)| details)
    }
}
