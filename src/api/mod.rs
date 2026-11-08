//! Endpoint groups, returned by methods on [`Client`](crate::Client).

mod coverage;
mod estimates;
mod shippings;
mod webhooks;

pub use coverage::Coverage;
pub use estimates::Estimates;
pub use shippings::{MAX_LABELS, Shippings};
pub use webhooks::Webhooks;
