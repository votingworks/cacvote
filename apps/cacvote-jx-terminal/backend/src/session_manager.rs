use std::sync::Arc;

use auth_rs::{async_card, card_details::PinInfo, vx_card::VxCard, CardReaderError, CertObject};
use openssl::x509::X509;
use tokio::sync::{mpsc, oneshot, watch};
use types_rs::cacvote;

use crate::db;

/// Manages the smartcard session, including authentication and signing.
/// Monitors the smartcard for insertion and removal.
pub(crate) struct SessionManager {
    /// The inner state of the session manager, uses an Arc to allow cloning.
    inner: Arc<SessionManagerInner>,
}

struct SessionManagerInner {
    /// The sender for the session data watch channel. Updates to the session
    /// are sent here to all subscribers. New subscribers will receive the
    /// current session data.
    session_data_tx: watch::Sender<cacvote::SessionData>,

    /// The sender for the session operations channel. Operations to be
    /// performed on the smartcard are sent here.
    session_ops_tx: mpsc::UnboundedSender<SessionOperation>,
}

impl Clone for SessionManager {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl SessionManager {
    pub(crate) fn new(
        vx_cert_authority_cert: openssl::x509::X509,
        vx_admin_cert_authority_cert: openssl::x509::X509,
        jurisdiction_code: cacvote::JurisdictionCode,
        pool: sqlx::PgPool,
    ) -> Self {
        let (session_data_tx, _) = watch::channel(cacvote::SessionData::Unauthenticated {
            has_smartcard: false,
        });
        let (session_ops_tx, mut session_ops_rx) = mpsc::unbounded_channel();

        // The task that manages the smartcard session. Interacts with the
        // smartcard via the `SessionManager` API using the channels owned by
        // `SessionManagerInner`.
        tokio::spawn({
            let session_data_tx = session_data_tx.clone();
            let session_ops_tx = session_ops_tx.clone();
            let vx_cert_authority_cert = vx_cert_authority_cert.clone();
            let vx_admin_cert_authority_cert = vx_admin_cert_authority_cert.clone();
            let mut db_interval = tokio::time::interval(tokio::time::Duration::from_millis(100));
            let mut last_session_data = None;

            async move {
                // spawn the context within the task
                let ctx = pcsc::Context::establish(pcsc::Scope::User).unwrap();
                let mut watcher = auth_rs::Watcher::watch();
                let mut vx_card: Option<VxCard> = None;

                let refresh_authenticated_session = || async {
                    let mut connection = pool.acquire().await.unwrap();

                    let elections = db::get_elections(&mut connection).await.unwrap();
                    let pending_registration_requests =
                        db::get_pending_registration_requests(&mut connection)
                            .await
                            .unwrap();
                    let registrations = db::get_registrations(&mut connection).await.unwrap();
                    let cast_ballots = db::get_cast_ballots(&mut connection).await.unwrap();
                    cacvote::SessionData::Authenticated {
                        jurisdiction_code: jurisdiction_code.clone(),
                        elections,
                        pending_registration_requests,
                        registrations,
                        cast_ballots,
                    }
                };

                loop {
                    // Select whichever of the following futures completes first.
                    tokio::select! {
                        // Handle smartcard events
                        event = watcher.recv() => {
                            tracing::debug!("received card event={event:?}");
                            match event {
                                Some(Ok(event)) => match event {
                                    auth_rs::Event::CardInserted { reader_name } => {
                                        let card = match async_card::AsyncCard::connect(&ctx, &reader_name) {
                                            Ok(async_card) => VxCard::new(
                                                vx_cert_authority_cert.clone(),
                                                vx_admin_cert_authority_cert.clone(),
                                                async_card
                                            ),
                                            Err(e) => {
                                                tracing::error!("error creating async card: {e}");
                                                continue;
                                            }
                                        };
                                        let card_details = match card.read_card_details().await {
                                            Ok(card_details) => card_details,
                                            Err(e) => {
                                                tracing::error!("error reading card details: {e}");
                                                continue;
                                            }
                                        };

                                        // hold on to the card for future operations
                                        vx_card = Some(card);

                                        match card_details.pin_info {
                                            PinInfo::NoPin => {
                                                // automatically authenticate if no PIN is required
                                                if let Err(e) = session_ops_tx.send(SessionOperation::SetAuthenticated) {
                                                    tracing::error!("error setting authenticated session: {e}");
                                                }
                                            }
                                            PinInfo::HasPin { num_incorrect_pin_attempts } => {
                                                session_data_tx.send_replace(cacvote::SessionData::Authenticating {
                                                    auth_error: if num_incorrect_pin_attempts == 0 {
                                                        None
                                                    } else {
                                                        Some(format!("incorrect PIN attempts: {num_incorrect_pin_attempts}"))
                                                    },
                                                });
                                            }
                                        }
                                    }
                                    auth_rs::Event::CardRemoved { .. } | auth_rs::Event::ReaderRemoved { .. } => {
                                        // clear the card so we don't try to use it
                                        vx_card = None;
                                        session_data_tx.send_replace(cacvote::SessionData::Unauthenticated {
                                            has_smartcard: false,
                                        });
                                    }
                                    auth_rs::Event::ReaderAdded { .. }
                                     => {
                                        // ignore
                                    }
                                },
                                Some(Err(e)) => {
                                    tracing::error!("error receiving smartcard event: {e}");
                                }
                                None => {
                                    tracing::debug!("no more card watcher events, exiting loop");
                                    break;
                                }
                            }
                        }

                        // Handle session operations
                        session_op = session_ops_rx.recv() => {
                            tracing::debug!("received session_op={session_op:?}");
                            match session_op {
                                Some(SessionOperation::SetAuthenticated) => {
                                    let new_session_data = refresh_authenticated_session().await;
                                    last_session_data = Some(new_session_data.clone());
                                    session_data_tx.send_replace(new_session_data);
                                }
                                Some(SessionOperation::CheckPin { pin, respond }) => {
                                    if let Some(ref mut vx_card) = vx_card {
                                        let result = vx_card.check_pin(&pin).await;
                                        tracing::debug!("check_pin result={result:?}");
                                        let _ = respond.send(result);
                                    } else {
                                        let _ = respond.send(Err(CardReaderError::NoCardFound));
                                    }
                                }
                                Some(SessionOperation::Sign { signing_cert, data, pin, respond }) => {
                                    if let Some(ref mut vx_card) = vx_card {
                                        let result = vx_card.sign(signing_cert, &data, pin.as_deref()).await;
                                        let _ = respond.send(result);
                                    } else {
                                        let _ = respond.send(Err(CardReaderError::NoCardFound));
                                    }
                                }
                                None => {
                                    tracing::debug!("session_ops channel closed?!");
                                }
                            }
                        }

                        // check the database periodically for changes
                        _ = db_interval.tick() => {
                            match last_session_data {
                                Some(ref mut last_session_data) => {
                                    if !matches!(last_session_data, cacvote::SessionData::Authenticated { .. }) {
                                        continue;
                                    }

                                    let new_session_data = refresh_authenticated_session().await;

                                    if new_session_data != *last_session_data {
                                        tracing::debug!("session data changed, updating session data");
                                        last_session_data.clone_from(&new_session_data);
                                        session_data_tx.send_replace(new_session_data);
                                    }
                                }
                                _ => {
                                    // do nothing
                                }
                            }
                        }
                    }
                }
            }
        });

        Self {
            inner: Arc::new(SessionManagerInner {
                session_data_tx,
                session_ops_tx,
            }),
        }
    }

    /// Attempts to authenticate the session with the given PIN. If it succeeds,
    /// the session data will become [`SessionData::Authenticated`] and this
    /// method will return `Ok(())`.
    pub(crate) async fn authenticate(&self, pin: &str) -> Result<(), String> {
        let (tx, rx) = oneshot::channel();
        self.inner
            .session_ops_tx
            .send(SessionOperation::CheckPin {
                pin: pin.to_owned(),
                respond: tx,
            })
            .map_err(|e| format!("error checking PIN: {e}"))?;
        rx.await
            .map_err(|e| format!("error checking PIN: {e}"))?
            .map_err(|e| format!("error checking PIN: {e}"))?;

        self.inner
            .session_ops_tx
            .send(SessionOperation::SetAuthenticated)
            .map_err(|e| format!("error setting authenticated session: {e}"))?;

        Ok(())
    }

    /// Subscribes to session data updates.
    pub(crate) fn subscribe(&self) -> watch::Receiver<cacvote::SessionData> {
        self.inner.session_data_tx.subscribe()
    }

    /// Signs the given data with the given signing certificate. If a PIN is
    /// required, provide it as the `pin` argument.
    #[allow(dead_code)]
    pub(crate) async fn sign(
        &self,
        signing_cert: CertObject,
        data: Vec<u8>,
        pin: Option<String>,
    ) -> Result<(Vec<u8>, X509), CardReaderError> {
        let (tx, rx) = oneshot::channel();
        self.inner
            .session_ops_tx
            .send(SessionOperation::Sign {
                signing_cert,
                data,
                pin,
                respond: tx,
            })
            .map_err(|e| CardReaderError::Other(format!("error sending sign operation: {e}")))?;
        rx.await
            .map_err(|e| CardReaderError::Other(format!("error receiving sign response: {e}")))?
    }
}

/// Operations that can be performed on the smartcard session. For operations
/// that require a response, a oneshot channel is provided.
#[derive(Debug)]
enum SessionOperation {
    SetAuthenticated,
    CheckPin {
        pin: String,
        respond: oneshot::Sender<Result<(), CardReaderError>>,
    },
    #[allow(dead_code)]
    Sign {
        signing_cert: CertObject,
        data: Vec<u8>,
        pin: Option<String>,
        respond: oneshot::Sender<Result<(Vec<u8>, X509), CardReaderError>>,
    },
}
