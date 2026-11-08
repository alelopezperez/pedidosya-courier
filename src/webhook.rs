//! Receiving callbacks (webhooks) from PedidosYa.
//!
//! PedidosYa `POST`s a JSON [`Callback`] to the URLs configured with
//! [`Webhooks::set`](crate::api::Webhooks::set) every time a shipping changes
//! status. There is no signature: authenticity comes from the
//! `authorizationKey` you configured, which PedidosYa sends back as the
//! `Authorization` header. Check it with [`verify_authorization`] before
//! trusting the body.
//!
//! This module doesn't depend on any web framework. With axum, for example:
//!
//! ```ignore
//! async fn pedidosya_webhook(headers: HeaderMap, body: Bytes) -> StatusCode {
//!     let header = headers.get(AUTHORIZATION).and_then(|v| v.to_str().ok());
//!     if !webhook::verify_authorization(header, &state.pedidosya_webhook_key) {
//!         return StatusCode::UNAUTHORIZED;
//!     }
//!     let Ok(callback) = webhook::parse(&body) else {
//!         return StatusCode::BAD_REQUEST;
//!     };
//!     // ... update your order with callback.data.status ...
//!     StatusCode::OK // answer 200 quickly; PedidosYa records the status code
//! }
//! ```

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::macros::string_enum;
use crate::models::{CancelCode, ShippingEvent, ShippingId, WebhookTopic};

/// A notification sent by PedidosYa to your webhook URL.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Callback {
    /// Topic, currently always [`WebhookTopic::ShippingStatus`].
    pub topic: WebhookTopic,
    /// Shipping ID.
    pub id: ShippingId,
    /// Your reference for the shipping.
    pub reference_id: Option<String>,
    /// Shipping confirmation code.
    pub confirmation_code: Option<String>,
    /// Whether the shipping is a test one.
    pub is_test: Option<bool>,
    /// When the event happened (UTC).
    pub generated: Option<DateTime<Utc>>,
    /// When the notification was sent (UTC).
    pub transmitted: Option<DateTime<Utc>>,
    /// Event details.
    pub data: CallbackData,
}

/// Details of a [`Callback`]. Which fields are set depends on the event.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct CallbackData {
    /// What happened, e.g. [`ShippingEvent::PickedUp`].
    pub status: ShippingEvent,
    /// Set when cancelled.
    pub cancel_code: Option<CancelCode>,
    /// Spanish text for the cancel code, set when cancelled.
    pub cancel_reason: Option<String>,
    /// Estimated pick-up time (UTC), while the shipping hasn't been picked up.
    pub estimated_pick_up_time: Option<DateTime<Utc>>,
    /// Estimated drop-off time (UTC), while the shipping is on its way.
    pub estimated_drop_off_time: Option<DateTime<Utc>>,
    /// Kind of delivery security code, for [`ShippingEvent::SecurityCodeCreated`].
    pub security_type: Option<SecurityType>,
    /// The delivery security code (PIN), for [`ShippingEvent::SecurityCodeCreated`].
    pub security_code: Option<String>,
}

string_enum! {
    /// Kind of delivery security code.
    pub enum SecurityType {
        /// A PIN the customer tells the rider.
        Pin => "PIN",
    }
}

/// Parses a callback body.
///
/// ```
/// let body = br#"{
///     "topic": "SHIPPING_STATUS",
///     "id": "ASDF-12345678",
///     "referenceId": "pedido-1042",
///     "generated": "2026-11-07T19:00:00Z",
///     "transmitted": "2026-11-07T19:00:02Z",
///     "data": { "status": "PICKED_UP" }
/// }"#;
///
/// let callback = pedidosya_courier::webhook::parse(body)?;
/// assert_eq!(callback.data.status, pedidosya_courier::models::ShippingEvent::PickedUp);
/// # Ok::<(), serde_json::Error>(())
/// ```
///
/// # Errors
///
/// Returns the JSON error if the body is not a valid callback.
pub fn parse(body: &[u8]) -> Result<Callback, serde_json::Error> {
    serde_json::from_slice(body)
}

/// Checks the `Authorization` header of a callback against the
/// `authorizationKey` you configured.
///
/// Uses a constant-time comparison, so the response time doesn't leak how
/// much of the key matched. A `Bearer ` prefix is tolerated.
///
/// ```
/// use pedidosya_courier::webhook::verify_authorization;
///
/// assert!(verify_authorization(Some("s3cret"), "s3cret"));
/// assert!(!verify_authorization(Some("wrong"), "s3cret"));
/// assert!(!verify_authorization(None, "s3cret"));
/// ```
#[must_use]
pub fn verify_authorization(header: Option<&str>, expected_key: &str) -> bool {
    let Some(received) = header else {
        return false;
    };
    if expected_key.is_empty() {
        return false;
    }
    let received = received.trim();
    let received = received.strip_prefix("Bearer ").unwrap_or(received);
    constant_time_eq(received.as_bytes(), expected_key.as_bytes())
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let diff = a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y));
    // `black_box` stops the optimizer from short-circuiting the loop.
    std::hint::black_box(diff) == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_authorization_cases() {
        assert!(verify_authorization(Some("key"), "key"));
        assert!(verify_authorization(Some("Bearer key"), "key"));
        assert!(verify_authorization(Some("  key "), "key"));
        assert!(!verify_authorization(Some("ke"), "key"));
        assert!(!verify_authorization(Some("kez"), "key"));
        assert!(!verify_authorization(Some(""), ""));
        assert!(!verify_authorization(None, "key"));
    }
}
