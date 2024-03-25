use std::fmt::Debug;
use std::ops::Deref;
use std::sync::{Arc, Mutex};

use auth_rs::card_details::CardDetailsWithAuthInfo;
use auth_rs::vx_card::VxCard;
use auth_rs::{CardReader, SharedCardReaders};
use types_rs::cacvote::SmartcardStatus;

pub(crate) type DynStatusGetter = Arc<dyn StatusGetterTrait + Send + Sync>;

#[cfg_attr(test, mockall::automock)]
pub(crate) trait StatusGetterTrait {
    fn get(&self) -> SmartcardStatus;
    fn get_card_details(&self) -> Option<CardDetailsWithAuthInfo>;
}

/// Provides access to the current smartcard status.
#[derive(Clone)]
pub(crate) struct StatusGetter {
    #[allow(dead_code)]
    ctx: pcsc::Context,
    #[allow(dead_code)]
    readers_with_cards: SharedCardReaders,
    #[allow(dead_code)]
    last_selected_card_reader_info: Arc<Mutex<Option<(String, CardDetailsWithAuthInfo)>>>,
    card: Option<Arc<VxCard>>,
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
            card: None,
        }
    }

    pub(crate) fn refresh_card_details(&mut self) {
        let readers = self.readers_with_cards.lock().unwrap();
        let Some(reader_name) = readers.first() else {
            return;
        };

        let mut cached_card_details = self.last_selected_card_reader_info.lock().unwrap();
        if let Some((ref name, ref details)) = cached_card_details.deref() {
            let same_reader_has_card = readers.iter().any(|reader| reader == name);

            if same_reader_has_card {
                return;
            }
        }

        let reader = CardReader::new(self.ctx.clone(), reader_name.clone());
        match reader.get_card() {
            Ok(card) => {
                self.card = Some(Arc::new(card));
            }
            Err(e) => {
                tracing::error!("error getting card: {e}");
            }
        }

        if let Some(card) = self.card.as_ref() {
            match card.read_card_details() {
                Ok(card_details_with_auth_info) => {
                    *cached_card_details =
                        Some((reader_name.to_string(), card_details_with_auth_info));
                }
                Err(e) => {
                    tracing::error!("error reading card details: {e}");
                }
            }
        }
    }
}

impl StatusGetterTrait for StatusGetter {
    /// Gets the current smartcard status.
    #[allow(dead_code)]
    #[must_use]
    fn get(&self) -> SmartcardStatus {
        let readers = self.readers_with_cards.lock().unwrap();

        if readers.is_empty() {
            SmartcardStatus::NoCard
        } else {
            SmartcardStatus::Card
        }
    }

    #[allow(dead_code)]
    #[must_use]
    fn get_card_details(&self) -> Option<CardDetailsWithAuthInfo> {
        self.refresh_card_details();
        let Ok(cached_card_details) = self.last_selected_card_reader_info.lock() else {
            return None;
        };

        cached_card_details.map(|(_, details)| details)
    }
}
