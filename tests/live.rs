//! Live checks against the real PedidosYa API, using TEST shippings only
//! (`isTest: true`; they are never delivered and are cancelled at the end).
//!
//! Ignored by default. Run with a token:
//!
//! ```sh
//! PEDIDOSYA_TOKEN=... cargo test -p pedidosya-courier --test live -- --ignored --nocapture --test-threads=1
//! ```
//!
//! Waypoints default to two points in Santo Domingo; override them for
//! accounts in other cities with `PEDIDOSYA_LIVE_CITY`, `..._PICKUP`/`..._DROPOFF`
//! (address) and `..._PICKUP_LAT`/`_LNG`, `..._DROPOFF_LAT`/`_LNG`.

use chrono::{Duration, Utc};
use pedidosya_courier::models::{
    CoverageRequest, CoverageWaypoint, Environment, Item, ListShippings, ShippingRequest,
    ShippingStatus, Waypoint, WaypointType,
};
use pedidosya_courier::{Client, Error};

fn env_or(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_owned())
}

fn env_f64(key: &str, default: f64) -> f64 {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

fn client() -> Client {
    let token = std::env::var("PEDIDOSYA_TOKEN").expect("set PEDIDOSYA_TOKEN to run live tests");
    Client::new(token).expect("client")
}

struct Points {
    city: String,
    pick_up: (String, f64, f64),
    drop_off: (String, f64, f64),
}

fn points() -> Points {
    Points {
        city: env_or("PEDIDOSYA_LIVE_CITY", "Santo Domingo"),
        pick_up: (
            env_or("PEDIDOSYA_LIVE_PICKUP", "Av. Abraham Lincoln 1003"),
            env_f64("PEDIDOSYA_LIVE_PICKUP_LAT", 18.4719),
            env_f64("PEDIDOSYA_LIVE_PICKUP_LNG", -69.9395),
        ),
        drop_off: (
            env_or("PEDIDOSYA_LIVE_DROPOFF", "Calle El Conde 101"),
            env_f64("PEDIDOSYA_LIVE_DROPOFF_LAT", 18.4735),
            env_f64("PEDIDOSYA_LIVE_DROPOFF_LNG", -69.8866),
        ),
    }
}

fn test_request(reference: &str) -> ShippingRequest {
    let p = points();
    ShippingRequest::new(
        reference,
        Waypoint::pick_up(&p.pick_up.0, &p.city, "Prueba Pickup", "+18095550000")
            .coordinates(p.pick_up.1, p.pick_up.2)
            .instructions("Envío de prueba, no recoger"),
        Waypoint::drop_off(&p.drop_off.0, &p.city, "Prueba Dropoff", "+18095551234")
            .coordinates(p.drop_off.1, p.drop_off.2)
            .instructions("Envío de prueba, no entregar"),
    )
    .item(
        Item::new("Libro de prueba", 500.0)
            .volume_cm3(10.0)
            .weight_kg(0.5)
            .sku("TEST-1"),
    )
    .environment(Environment::Test)
}

/// Prints the outcome of a call that may legitimately fail for a fresh
/// test shipping (e.g. tracking before a rider is assigned). Decoding errors
/// (bugs in this crate) are collected in `bugs` instead of panicking, so the
/// test can still cancel its shipping before failing.
fn report<T: std::fmt::Debug>(
    bugs: &mut Vec<String>,
    name: &str,
    result: Result<T, Error>,
) -> Option<T> {
    match result {
        Ok(value) => {
            println!("[ok]   {name}: {value:#?}");
            Some(value)
        }
        Err(Error::Decode { source, body }) => {
            let bug = format!("{name}: could not decode response: {source}\nbody: {body}");
            println!("[BUG]  {bug}");
            bugs.push(bug);
            None
        }
        Err(e) => {
            println!("[info] {name}: {e}");
            None
        }
    }
}

#[tokio::test]
#[ignore = "calls the real PedidosYa API; needs PEDIDOSYA_TOKEN"]
async fn read_only_endpoints() {
    let client = client();
    let mut bugs = Vec::new();

    let schedules = client.schedules().await.expect("schedules");
    println!("[ok]   schedules: {schedules:?}");
    assert_ne!(schedules.len(), 0);

    report(
        &mut bugs,
        "working zones",
        client.coverage().working_zones().await,
    );
    report(&mut bugs, "webhooks config", client.webhooks().get().await);
    report(
        &mut bugs,
        "webhooks config (test)",
        client.webhooks().get_for(Environment::Test).await,
    );

    let today = Utc::now().date_naive();
    let query = ListShippings::new(today - Duration::days(14), today);
    report(
        &mut bugs,
        "list (14 days)",
        client.shippings().list(&query).await,
    );

    let p = points();
    let coverage = CoverageRequest::new([
        CoverageWaypoint::coordinates(WaypointType::PickUp, p.pick_up.1, p.pick_up.2),
        CoverageWaypoint::coordinates(WaypointType::DropOff, p.drop_off.1, p.drop_off.2),
    ]);
    report(
        &mut bugs,
        "coverage",
        client.coverage().check(&coverage).await,
    );
    assert!(bugs.is_empty(), "decoding bugs: {bugs:#?}");
}

#[tokio::test]
#[ignore = "calls the real PedidosYa API; needs PEDIDOSYA_TOKEN"]
async fn estimate_confirm_and_cancel_test_shipping() {
    let client = client();
    let reference = format!("crate-live-{}", Utc::now().timestamp());

    let estimate = client
        .estimates()
        .create(&test_request(&reference))
        .await
        .expect("estimate");
    println!("[ok]   estimate: {estimate:#?}");
    assert_eq!(estimate.is_test, Some(true));
    assert!(!estimate.delivery_offers.is_empty(), "no delivery offers");

    let offer = &estimate.delivery_offers[0].delivery_offer_id;
    let shipping = client
        .estimates()
        .confirm(&estimate.estimate_id, Some(offer))
        .await
        .expect("confirm");
    println!("[ok]   confirmed: {shipping:#?}");
    assert_eq!(shipping.is_test, Some(true), "must be a TEST shipping");
    let id = shipping.shipping_id.clone();
    let mut bugs = Vec::new();

    // Nothing below may panic before the shipping is cancelled.
    report(&mut bugs, "get", client.shippings().get(&id).await);
    let today = Utc::now().date_naive();
    let query = ListShippings::new(today - Duration::days(1), today + Duration::days(1));
    report(
        &mut bugs,
        "list",
        client.shippings().list(&query).await.map(|p| p.items.len()),
    );
    report(
        &mut bugs,
        "tracking",
        client.shippings().tracking(&id).await,
    );
    report(
        &mut bugs,
        "callbacks",
        client.shippings().callbacks(&id).await,
    );
    report(
        &mut bugs,
        "delivery pin",
        client.shippings().delivery_pin(&id).await,
    );
    report(
        &mut bugs,
        "proof of delivery",
        client.shippings().proof_of_delivery(&id).await,
    );
    report(
        &mut bugs,
        "update instructions",
        client
            .shippings()
            .update_delivery_instructions(&id, "Prueba: instrucciones actualizadas")
            .await,
    );
    report(
        &mut bugs,
        "labels (PDF bytes)",
        client
            .shippings()
            .labels(std::slice::from_ref(&id))
            .await
            .map(|b| b.len()),
    );

    let cancelled = client
        .shippings()
        .cancel(&id, "Prueba automática del crate")
        .await
        .expect("cancel");
    println!("[ok]   cancel: {cancelled:#?}");
    assert_eq!(cancelled.status, Some(ShippingStatus::Cancelled));
    assert!(bugs.is_empty(), "decoding bugs: {bugs:#?}");
}

#[tokio::test]
#[ignore = "calls the real PedidosYa API; needs PEDIDOSYA_TOKEN"]
async fn create_and_cancel_test_shipping_directly() {
    let client = client();
    let reference = format!("crate-live-direct-{}", Utc::now().timestamp());

    let shipping = client
        .shippings()
        .create(&test_request(&reference))
        .await
        .expect("create");
    println!("[ok]   create: {shipping:#?}");
    assert_eq!(shipping.is_test, Some(true), "must be a TEST shipping");

    let cancelled = client
        .shippings()
        .cancel(&shipping.shipping_id, "Prueba automática del crate")
        .await
        .expect("cancel");
    println!("[ok]   cancel: {cancelled:#?}");
    assert_eq!(cancelled.status, Some(ShippingStatus::Cancelled));
}

/// Changes the account's webhook configuration (test environment), then
/// restores the original configuration.
#[tokio::test]
#[ignore = "calls the real PedidosYa API and changes webhook config; needs PEDIDOSYA_TOKEN"]
async fn webhooks_set_get_delete_and_restore() {
    use pedidosya_courier::models::{
        WebhookSubscription, WebhookTopic, WebhookUrl, WebhooksConfig,
    };

    let client = client();
    let original = client.webhooks().get().await.expect("get original");
    println!("[ok]   original: {original:#?}");

    let url = "https://example.com/pedidosya-courier-live-test";
    let config = WebhooksConfig::new([WebhookSubscription::shipping_status(Environment::Test)
        .url(WebhookUrl::new(url).authorization_key("live-test-key"))]);

    let mut failure = None;
    match client.webhooks().set(&config).await {
        Ok(stored) => println!("[ok]   set: {stored:#?}"),
        Err(e) => failure = Some(format!("set: {e}")),
    }
    if failure.is_none() {
        match client.webhooks().get_for(Environment::Test).await {
            Ok(test) => {
                println!("[ok]   get (test): {test:#?}");
                let found = test
                    .subscriptions
                    .iter()
                    .any(|s| s.is_test && s.urls.iter().any(|u| u.url == url));
                if !found {
                    failure = Some("set subscription not found in get".into());
                }
            }
            Err(e) => failure = Some(format!("get: {e}")),
        }
        match client.webhooks().get_for(Environment::Production).await {
            Ok(prod) => println!("[ok]   get (production) after set: {prod:#?}"),
            Err(e) => failure = Some(format!("get production: {e}")),
        }
    }
    if failure.is_none() {
        match client
            .webhooks()
            .delete(&WebhookTopic::ShippingStatus, Environment::Test)
            .await
        {
            Ok(deleted) => println!("[ok]   delete: {deleted:#?}"),
            Err(e) => failure = Some(format!("delete: {e}")),
        }
        match client.webhooks().get().await {
            Ok(after) => println!("[ok]   after delete: {after:#?}"),
            Err(e) => failure = Some(format!("get after delete: {e}")),
        }
    }

    // Always restore what was there before.
    if !original.subscriptions.is_empty() {
        let restored = client
            .webhooks()
            .set(&original)
            .await
            .expect("restore original config");
        println!("[ok]   restored: {restored:#?}");
    }
    let now = client.webhooks().get().await.expect("get final");
    assert_eq!(
        now.subscriptions.len(),
        original.subscriptions.len(),
        "config not restored"
    );
    assert!(failure.is_none(), "{}", failure.unwrap_or_default());
}
