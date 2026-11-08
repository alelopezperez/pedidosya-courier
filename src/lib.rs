//! Unofficial async client for the [PedidosYa Courier API] (v3).
//!
//! Create shippings, get quotes, check coverage, track riders, download
//! labels, and configure and receive status webhooks.
//!
//! [PedidosYa Courier API]: https://developers.pedidosya.com/api-doc
//!
//! # Quick start
//!
//! ```no_run
//! use pedidosya_courier::Client;
//! use pedidosya_courier::models::{Environment, Item, ShippingRequest, Waypoint};
//!
//! # async fn run() -> pedidosya_courier::Result<()> {
//! // Token from the PedidosYa Envíos dashboard: Integraciones → Generar token.
//! let client = Client::new(std::env::var("PEDIDOSYA_TOKEN").unwrap_or_default())?;
//!
//! let request = ShippingRequest::new(
//!     "pedido-1042",
//!     Waypoint::pick_up("Av. Winston Churchill 93", "Santo Domingo", "Pica Pollo Doña Carmen", "+18095550000"),
//!     Waypoint::drop_off("Calle El Conde 45", "Santo Domingo", "María", "+18095551234")
//!         .instructions("Tocar el timbre"),
//! )
//! .item(Item::new("Pica pollo familiar", 850.0).quantity(2).weight_kg(1.5))
//! .environment(Environment::Test);
//!
//! // 1. Get a quote with the available delivery offers...
//! let estimate = client.estimates().create(&request).await?;
//!
//! // 2. ...and confirm one of them.
//! let offer = estimate.delivery_offers.first().map(|o| &o.delivery_offer_id);
//! let shipping = client.estimates().confirm(&estimate.estimate_id, offer).await?;
//! println!("shipping {} is {:?}", shipping.shipping_id, shipping.status);
//!
//! // Share the live tracking page with your customer.
//! if let Some(url) = &shipping.share_location_url {
//!     println!("track it at {url}");
//! }
//! # Ok(())
//! # }
//! ```
//!
//! # Overview
//!
//! | Method | Endpoints |
//! |---|---|
//! | [`Client::shippings`] | create, get, list, cancel, tracking, proof of delivery, PIN, delivery instructions, callbacks, labels |
//! | [`Client::estimates`] | create a quote, confirm it |
//! | [`Client::coverage`] | check coverage, working zones |
//! | [`Client::webhooks`] | get, set and delete webhook configuration |
//! | [`Client::schedules`] | fleet working hours |
//! | [`webhook`] module | parse and authenticate incoming callbacks |
//!
//! # Errors
//!
//! Every call returns [`Result<T>`]. Error responses from PedidosYa become
//! [`Error::Api`], with the status, the documented [`ErrorCode`] and the
//! message:
//!
//! ```no_run
//! # async fn run(client: pedidosya_courier::Client, request: pedidosya_courier::models::ShippingRequest) {
//! use pedidosya_courier::{Error, ErrorCode};
//!
//! match client.shippings().create(&request).await {
//!     Ok(shipping) => println!("created {}", shipping.shipping_id),
//!     Err(Error::Api(e)) if e.code() == Some(&ErrorCode::WaypointsOutOfZone) => {
//!         println!("we don't deliver there yet");
//!     }
//!     Err(e) => eprintln!("pedidosya error: {e}"),
//! }
//! # }
//! ```
//!
//! # Forward compatibility
//!
//! The API's own examples contain values its specification doesn't list.
//! This crate never fails on them: enums keep unknown values in an
//! `Unknown(String)` variant, and new response fields are ignored. Response
//! structs and enums are `#[non_exhaustive]`, so adding fields and variants
//! is not a breaking change.
//!
//! # Features
//!
//! - `rustls` (default): TLS with rustls.
//! - `native-tls`: TLS with the platform's library (OpenSSL on Linux).
//!
//! # Disclaimer
//!
//! This crate is not affiliated with or endorsed by PedidosYa.

#![forbid(unsafe_code)]
#![warn(
    missing_docs,
    missing_debug_implementations,
    rust_2018_idioms,
    unreachable_pub
)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

mod macros;

pub mod api;
mod client;
mod error;
pub mod models;
pub mod webhook;

pub use client::{Client, ClientBuilder, DEFAULT_BASE_URL};
pub use error::{ApiError, Error, ErrorCode, HttpError, Result};

#[cfg(test)]
mod tests {
    use super::*;

    // C-SEND-SYNC: these must stay usable across threads and tasks.
    fn assert_send_sync<T: Send + Sync>() {}

    #[test]
    fn public_types_are_send_and_sync() {
        assert_send_sync::<Client>();
        assert_send_sync::<Error>();
        assert_send_sync::<models::Shipping>();
        assert_send_sync::<webhook::Callback>();
    }
}
