use std::fmt::Debug;
use std::sync::{Arc, Mutex};

use auth_rs::card_details::CardDetailsWithAuthInfo;
use auth_rs::vx_card::{VxCard, CARD_VX_ADMIN_CERT};
use auth_rs::{CardReader, SharedCardReaders};
use openssl::x509::X509;
use types_rs::cacvote::SmartcardStatus;

pub(crate) type DynSmartcard = Arc<dyn SmartcardTrait + Send + Sync>;

#[cfg_attr(test, mockall::automock)]
pub(crate) trait SmartcardTrait {
    fn get_status(&self) -> SmartcardStatus;
    fn get_card_details(&self) -> Option<CardDetailsWithAuthInfo>;

    #[allow(clippy::needless_lifetimes)] // automock needs the lifetimes
    fn sign<'a, 'b, 'c>(&'a self, data: &'b [u8], pin: Option<&'c str>) -> Result<Signed, String>;
}

#[derive(Debug)]
pub(crate) struct Signed {
    pub(crate) data: Vec<u8>,
    pub(crate) cert_stack: Vec<X509>,
}

/// Provides access to the current smartcard.
#[derive(Clone)]
pub(crate) struct Smartcard(Arc<Mutex<SmartcardInner>>);

impl Smartcard {
    pub(crate) fn new(readers_with_cards: SharedCardReaders) -> Self {
        Self(Arc::new(Mutex::new(SmartcardInner::new(
            readers_with_cards,
        ))))
    }
}

impl Debug for Smartcard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0.lock() {
            Ok(inner) => inner.fmt(f),
            Err(e) => write!(f, "Smartcard {{ error: {e} }}"),
        }
    }
}

#[derive(Clone)]
struct SmartcardInner {
    #[allow(dead_code)]
    ctx: pcsc::Context,
    #[allow(dead_code)]
    readers_with_cards: SharedCardReaders,
    #[allow(dead_code)]
    last_selected_card_reader_info: Option<(String, CardDetailsWithAuthInfo)>,
    card: Option<Arc<VxCard>>,
}

impl Debug for SmartcardInner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Smartcard")
            .field("readers_with_cards", &self.readers_with_cards)
            .finish()
    }
}

impl SmartcardInner {
    #[allow(dead_code)]
    #[must_use]
    pub(crate) fn new(readers_with_cards: SharedCardReaders) -> Self {
        Self {
            ctx: pcsc::Context::establish(pcsc::Scope::User).unwrap(),
            readers_with_cards,
            last_selected_card_reader_info: None,
            card: None,
        }
    }

    pub(crate) fn refresh_card_details(&mut self) {
        let readers = self.readers_with_cards.lock().unwrap();
        let Some(reader_name) = readers.first() else {
            return;
        };

        let cached_card_details = &self.last_selected_card_reader_info;
        if let Some((ref name, _)) = cached_card_details {
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

        if let Some(ref card) = self.card {
            match card.read_card_details() {
                Ok(card_details_with_auth_info) => {
                    self.last_selected_card_reader_info =
                        Some((reader_name.to_string(), card_details_with_auth_info));
                }
                Err(e) => {
                    tracing::error!("error reading card details: {e}");
                }
            }
        }
    }
}

impl SmartcardTrait for Smartcard {
    /// Gets the current smartcard status.
    #[allow(dead_code)]
    #[must_use]
    fn get_status(&self) -> SmartcardStatus {
        let inner = match self.0.lock() {
            Ok(inner) => inner,
            Err(e) => {
                tracing::error!("error getting smartcard lock: {e}");
                return SmartcardStatus::NoCard;
            }
        };

        let readers = inner.readers_with_cards.lock().unwrap();

        if readers.is_empty() {
            SmartcardStatus::NoCard
        } else {
            SmartcardStatus::Card
        }
    }

    #[allow(dead_code)]
    #[must_use]
    fn get_card_details(&self) -> Option<CardDetailsWithAuthInfo> {
        let mut inner = match self.0.lock() {
            Ok(inner) => inner,
            Err(e) => {
                tracing::error!("error getting smartcard lock: {e}");
                return None;
            }
        };
        inner.refresh_card_details();
        inner
            .last_selected_card_reader_info
            .clone()
            .map(|(_, details)| details)
    }

    #[allow(dead_code)]
    fn sign(&self, data: &[u8], pin: Option<&str>) -> Result<Signed, String> {
        let inner = match self.0.lock() {
            Ok(inner) => inner,
            Err(e) => {
                return Err(format!("error getting smartcard lock: {e}"));
            }
        };
        let card = inner.card.as_ref().ok_or("no card")?;
        let (data, public_key) = card
            .sign(CARD_VX_ADMIN_CERT, data, pin)
            .map_err(|e| format!("error signing: {e}"))?;

        Ok(Signed {
            data,
            cert_stack: vec![public_key],
        })
    }
}
