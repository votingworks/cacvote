#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("no reader is connected")]
    NoReader,
    #[error("no card is present in the reader")]
    NoCard,
    #[error("PC/SC error: {0}")]
    PcscError(#[from] pcsc::Error),
    #[error("unknown error: {0}")]
    UnknownError(String),
}

pub struct JavaCard {
    ctx: pcsc::Context,
    card: pcsc::Card,
}

impl JavaCard {
    pub fn connect() -> Result<Self, Error> {
        // Establish a PC/SC context.
        let ctx = pcsc::Context::establish(pcsc::Scope::User)?;

        // List available readers.
        let mut readers_buf = [0; 2048];
        let mut readers = ctx
            .list_readers(&mut readers_buf)
            .map_err(|_| Error::NoReader)?;

        // Use the first reader.
        let reader = readers.next().ok_or_else(|| Error::NoReader)?;

        // Connect to the card.
        let card = ctx
            .connect(reader, pcsc::ShareMode::Exclusive, pcsc::Protocols::ANY)
            .map_err(|e| match e {
                pcsc::Error::NoSmartcard => Error::NoCard,
                e => e.into(),
            })?;

        Ok(Self { ctx, card })
    }
}
