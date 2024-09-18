use std::{ffi::CString, fmt::Debug};

use pcsc::{Card, Context};
use tokio::{
    sync::{mpsc, oneshot},
    task,
};

/// Asynchronous wrapper around a PC/SC card using Tokio.
pub struct AsyncCard {
    sender: mpsc::UnboundedSender<CardOperation>,
    task_handle: Option<task::JoinHandle<()>>,
}

impl AsyncCard {
    /// Connect to a card by card reader name.
    pub fn connect(ctx: &Context, name: &str) -> Result<Self, Error> {
        let reader = CString::new(name.as_bytes())?;
        let card = ctx.connect(&reader, pcsc::ShareMode::Shared, pcsc::Protocols::ANY)?;
        Ok(Self::from_card(card)?)
    }

    /// Create an [`AsyncCard`] from a [`pcsc::Card`].
    pub fn from_card(card: Card) -> Result<Self, pcsc::Error> {
        let (tx, mut rx) = mpsc::unbounded_channel::<CardOperation>();

        // Spawn a blocking Tokio task to handle the card operations
        let task_handle = tokio::task::spawn_blocking(move || {
            let mut receive_buffer = [0; 1024];
            tracing::debug!("starting async card loop");
            while let Some(op) = rx.blocking_recv() {
                tracing::debug!("received operation: {:?}", op);
                match op {
                    CardOperation::Transmit(data, response) => {
                        let result = card
                            .transmit(&data, &mut receive_buffer)
                            .map(|response| response.to_vec());
                        let _ = response.send(result);
                    }
                    CardOperation::Disconnect => {
                        tracing::debug!("received disconnect request");
                        break;
                    }
                };
            }
            tracing::debug!("exiting async card loop");
        });

        Ok(Self {
            sender: tx,
            task_handle: Some(task_handle),
        })
    }

    /// Transmit data to the card and return the response.
    pub async fn transmit(&self, data: Vec<u8>) -> Result<Vec<u8>, Error> {
        let (tx, rx) = oneshot::channel();

        self.sender
            .send(CardOperation::Transmit(data, tx))
            .map_err(|e| Error::SendError(format!("could not initiate transmit: {e}")))?;
        rx.await?
            .map_err(|e| Error::SendError(format!("failed to transmit: {e:?}")))
    }

    /// Disconnect from the card.
    pub async fn disconnect(mut self) -> Result<(), Error> {
        tracing::debug!("sending disconnect request");
        self.sender
            .send(CardOperation::Disconnect)
            .map_err(|e| Error::SendError(format!("could not initiate disconnect: {e}")))?;
        if let Some(task_handle) = self.task_handle.take() {
            task_handle
                .await
                .map_err(|e| Error::SendError(format!("could not await task handle: {e}")))?;
        }
        tracing::debug!("disconnected");
        Ok(())
    }
}

impl Drop for AsyncCard {
    fn drop(&mut self) {
        let _ = self.sender.send(CardOperation::Disconnect);
    }
}

/// Define the operations we want to perform on the Card. Used to communicate
/// with the Tokio task.
#[derive(Debug)]
enum CardOperation {
    /// Transmit data to the card and return the response via the oneshot channel.
    Transmit(Vec<u8>, oneshot::Sender<Result<Vec<u8>, pcsc::Error>>),

    /// Disconnect the card.
    Disconnect,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("NUL byte in string")]
    NulError(#[from] std::ffi::NulError),

    /// PC/SC error
    #[error("PC/SC error: {0}")]
    Pcsc(#[from] pcsc::Error),

    #[error("receive error: {0}")]
    RecvError(#[from] oneshot::error::RecvError),

    #[error("send error: {0}")]
    SendError(String),
}
