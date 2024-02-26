use std::fmt::Debug;

use tracing::field::DebugValue;

pub(crate) struct HexDebug<T: Debug>(pub T);

impl<T: Debug> Debug for HexDebug<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:02x?}", self.0)
    }
}

/// Wraps a `tracing` field value so that it is displayed as hex in debug output.
///
/// # Example
///
/// ```ignore
/// use crate::hex_debug::hex_debug;
///
/// let data = vec![0x01, 0x02, 0x03];
/// tracing::info!(data = hex_debug(&data));
/// ```
pub(crate) fn hex_debug<T: Debug>(t: T) -> DebugValue<HexDebug<T>> {
    tracing::field::debug(HexDebug(t))
}
