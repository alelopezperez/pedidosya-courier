//! Parsing every callback example from PedidosYa's OpenAPI spec.

use pedidosya_courier::models::{CancelCode, ShippingEvent, WebhookTopic};
use pedidosya_courier::webhook::{self, SecurityType};

fn fixture(name: &str) -> Vec<u8> {
    let path = format!("{}/tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR"));
    std::fs::read(&path).unwrap_or_else(|e| panic!("read {path}: {e}"))
}

#[test]
fn every_callback_example_parses_to_a_known_event() {
    let cases = [
        (
            "callback_shipping_status_confirmed.json",
            ShippingEvent::Confirmed,
        ),
        (
            "callback_shipping_status_in_progress.json",
            ShippingEvent::InProgress,
        ),
        (
            "callback_shipping_status_near_pickup.json",
            ShippingEvent::NearPickup,
        ),
        (
            "callback_shipping_status_picked_up.json",
            ShippingEvent::PickedUp,
        ),
        (
            "callback_shipping_status_near_dropoff.json",
            ShippingEvent::NearDropoff,
        ),
        (
            "callback_shipping_status_completed.json",
            ShippingEvent::Completed,
        ),
        (
            "callback_shipping_status_cancelled.json",
            ShippingEvent::Cancelled,
        ),
        (
            "callback_shipping_status_expired.json",
            ShippingEvent::Expired,
        ),
        (
            "callback_security_code_created.json",
            ShippingEvent::SecurityCodeCreated,
        ),
    ];
    for (file, expected) in cases {
        let callback = webhook::parse(&fixture(file)).unwrap_or_else(|e| panic!("{file}: {e}"));
        assert_eq!(callback.topic, WebhookTopic::ShippingStatus, "{file}");
        assert_eq!(callback.data.status, expected, "{file}");
        assert!(callback.generated.is_some(), "{file}");
    }
}

#[test]
fn cancelled_callback_has_code_and_reason() {
    let callback = webhook::parse(&fixture("callback_shipping_status_cancelled.json")).unwrap();
    assert_eq!(
        callback.data.cancel_code,
        Some(CancelCode::NoRiderAvailable)
    );
    assert_eq!(
        callback.data.cancel_reason.as_deref(),
        Some("No hay cadete disponible en este momento")
    );
}

#[test]
fn security_code_callback_has_pin() {
    let callback = webhook::parse(&fixture("callback_security_code_created.json")).unwrap();
    assert_eq!(callback.data.security_type, Some(SecurityType::Pin));
    assert_eq!(callback.data.security_code.as_deref(), Some("65421"));
    assert_eq!(callback.is_test, Some(true));
    assert_eq!(callback.confirmation_code.as_deref(), Some("9869921368"));
}

#[test]
fn invalid_body_is_an_error() {
    assert!(webhook::parse(b"{}").is_err());
    assert!(webhook::parse(b"not json").is_err());
}
