//! Endpoint tests against a local mock server, using the response examples
//! from PedidosYa's OpenAPI spec as fixtures.

use chrono::{NaiveDate, NaiveTime, Weekday};
use pedidosya_courier::models::{
    CancelCode, CoverageRequest, CoverageWaypoint, Currency, DeliveryMode, DeliveryOfferId,
    Environment, EstimateId, Item, ListShippings, ShippingId, ShippingRequest, ShippingStatus,
    Waypoint, WaypointType, WebhookSubscription, WebhookTopic, WebhookUrl, WebhooksConfig,
};
use pedidosya_courier::{Client, Error, ErrorCode};
use serde_json::json;
use wiremock::matchers::{body_json, body_partial_json, header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

const TOKEN: &str = "test-token";

fn fixture(name: &str) -> serde_json::Value {
    let path = format!("{}/tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR"));
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {path}: {e}"));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("parse {path}: {e}"))
}

async fn setup() -> (MockServer, Client) {
    let server = MockServer::start().await;
    let client = Client::builder(TOKEN)
        .base_url(server.uri())
        .build()
        .expect("client");
    (server, client)
}

fn sample_request() -> ShippingRequest {
    ShippingRequest::new(
        "pedido-1042",
        Waypoint::pick_up(
            "Av. Winston Churchill 93",
            "Santo Domingo",
            "Doña Carmen",
            "+18095550000",
        ),
        Waypoint::drop_off(
            "Calle El Conde 45",
            "Santo Domingo",
            "María",
            "+18095551234",
        )
        .coordinates(18.4734, -69.8840)
        .instructions("Tocar el timbre"),
    )
    .item(
        Item::new("Pica pollo familiar", 850.0)
            .quantity(2)
            .weight_kg(1.5),
    )
    .environment(Environment::Test)
}

#[tokio::test]
async fn sends_raw_token_and_parses_schedules() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/v3/schedules"))
        // PedidosYa wants the raw token, no "Bearer" prefix.
        .and(header("authorization", TOKEN))
        .and(header(
            "user-agent",
            concat!("pedidosya-courier/", env!("CARGO_PKG_VERSION")),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixture("200_schedules.json")))
        .expect(1)
        .mount(&server)
        .await;

    let schedules = client.schedules().await.expect("schedules");
    assert_eq!(schedules.len(), 2);
    assert_eq!(schedules[0].day, Weekday::Mon);
    assert_eq!(schedules[0].from, NaiveTime::from_hms_opt(7, 0, 0).unwrap());
    assert_eq!(schedules[0].to, NaiveTime::from_hms_opt(23, 0, 0).unwrap());
    assert_eq!(schedules[1].day, Weekday::Tue);
}

#[tokio::test]
async fn create_shipping_sends_expected_body() {
    let (server, client) = setup().await;
    Mock::given(method("POST"))
        .and(path("/v3/shippings"))
        .and(body_partial_json(json!({
            "referenceId": "pedido-1042",
            "isTest": true,
            "items": [{ "description": "Pica pollo familiar", "value": 850.0, "quantity": 2,
                        "volume": 0.0, "weight": 1.5 }],
            "waypoints": [
                { "type": "PICK_UP", "addressStreet": "Av. Winston Churchill 93", "city": "Santo Domingo",
                  "name": "Doña Carmen", "phone": "+18095550000" },
                { "type": "DROP_OFF", "latitude": 18.4734, "longitude": -69.8840,
                  "instructions": "Tocar el timbre" }
            ]
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixture("200_shippings_id.json")))
        .expect(1)
        .mount(&server)
        .await;

    let shipping = client
        .shippings()
        .create(&sample_request())
        .await
        .expect("create");
    assert_eq!(shipping.shipping_id.as_str(), "15562302161927091277656");
    assert_eq!(shipping.status, Some(ShippingStatus::Confirmed));
    assert_eq!(shipping.waypoints.len(), 2);
    assert_eq!(shipping.waypoints[0].kind, WaypointType::PickUp);

    let route = shipping.route.expect("route");
    // Not in the spec's enum, but present in its examples.
    assert_eq!(route.delivery_mode, Some(DeliveryMode::CrossDocking));
    assert_eq!(route.estimated_driving_time, Some(120));
    let pricing = route.pricing.expect("pricing");
    // The spec says `subtotal`, the examples say `subTotal`.
    assert_eq!(pricing.subtotal, Some(180.0));
    assert_eq!(pricing.currency, Some(Currency::Uyu));
}

#[tokio::test]
async fn request_omits_unset_optional_fields() {
    let request = ShippingRequest::new(
        "ref",
        Waypoint::pick_up("A 1", "Santo Domingo", "A", "+1"),
        Waypoint::drop_off("B 2", "Santo Domingo", "B", "+2"),
    );
    let value = serde_json::to_value(&request).unwrap();
    assert_eq!(value["isTest"], json!(false));
    for key in ["deliveryTime", "notificationMail", "requirements"] {
        assert!(value.get(key).is_none(), "{key} should be omitted");
    }
    assert!(value["waypoints"][0].get("latitude").is_none());
}

#[tokio::test]
async fn estimate_parses_offers_and_route_alias() {
    let (server, client) = setup().await;
    Mock::given(method("POST"))
        .and(path("/v3/shippings/estimates"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(fixture("200_shippings_estimates.json")),
        )
        .mount(&server)
        .await;

    let estimate = client
        .estimates()
        .create(&sample_request())
        .await
        .expect("estimate");
    assert_eq!(estimate.estimate_id.as_str(), "520644687847685483");
    let modes: Vec<_> = estimate
        .delivery_offers
        .iter()
        .map(|o| o.delivery_mode.clone())
        .collect();
    assert_eq!(
        modes,
        vec![
            Some(DeliveryMode::Express),
            Some(DeliveryMode::Schedule),
            Some(DeliveryMode::CrossDocking)
        ]
    );
    // The example uses `routes` instead of `route`.
    assert_eq!(estimate.route.and_then(|r| r.distance), Some(123.0));
}

#[tokio::test]
async fn confirm_estimate_with_and_without_offer() {
    let (server, client) = setup().await;
    Mock::given(method("POST"))
        .and(path("/v3/shippings/estimates/est-1/confirm"))
        .and(body_json(json!({ "deliveryOfferId": "offer-9" })))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixture("200_shippings_id.json")))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/v3/shippings/estimates/est-2/confirm"))
        .and(body_json(json!({})))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixture("200_shippings_id.json")))
        .expect(1)
        .mount(&server)
        .await;

    let offer = DeliveryOfferId::new("offer-9");
    client
        .estimates()
        .confirm(&EstimateId::new("est-1"), Some(&offer))
        .await
        .expect("with offer");
    client
        .estimates()
        .confirm(&EstimateId::new("est-2"), None)
        .await
        .expect("without offer");
}

#[tokio::test]
async fn list_sends_dates_and_cursor() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/v3/shippings"))
        .and(query_param("fromDate", "2026-11-01"))
        .and(query_param("toDate", "2026-11-07"))
        .and(query_param("nextPageId", "cursor-1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixture("200_shippings.json")))
        .expect(1)
        .mount(&server)
        .await;

    let query = ListShippings::new(
        NaiveDate::from_ymd_opt(2026, 11, 1).unwrap(),
        NaiveDate::from_ymd_opt(2026, 11, 7).unwrap(),
    )
    .page("cursor-1");
    let page = client.shippings().list(&query).await.expect("list");
    assert_eq!(page.items.len(), 3);
    assert_eq!(page.items[0].cancel_code, Some(CancelCode::UserCancelled));

    let next = page.next_query(&query).expect("has next page");
    assert_eq!(
        next.next_page_id.as_deref(),
        Some("asbajhbd-123-sfasf-85h5hh-asddsad")
    );
    assert_eq!(next.from_date, query.from_date);
}

#[tokio::test]
async fn cancel_sends_reason() {
    let (server, client) = setup().await;
    Mock::given(method("POST"))
        .and(path("/v3/shippings/abc/cancel"))
        .and(body_json(json!({ "reasonText": "cliente canceló" })))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(fixture("200_shippings_id_cancel.json")),
        )
        .expect(1)
        .mount(&server)
        .await;

    let shipping = client
        .shippings()
        .cancel(&ShippingId::new("abc"), "cliente canceló")
        .await
        .expect("cancel");
    assert_eq!(shipping.status, Some(ShippingStatus::Cancelled));
    assert!(shipping.status.as_ref().unwrap().is_final());
    assert_eq!(
        shipping.cancel_reason.as_deref(),
        Some("Cancelado por el usuario")
    );
}

#[tokio::test]
async fn ids_are_percent_encoded_in_paths() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/v3/shippings/a%2Fb%3Fc/tracking"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(fixture("200_shippings_id_tracking.json")),
        )
        .expect(1)
        .mount(&server)
        .await;

    let tracking = client
        .shippings()
        .tracking(&ShippingId::new("a/b?c"))
        .await
        .expect("tracking");
    assert_eq!(tracking.delivery_name.as_deref(), Some("Tim"));
    assert_eq!(tracking.delivery_transport.as_deref(), Some("Bike"));
}

#[tokio::test]
async fn delivery_pin_and_proof_of_delivery() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/v3/shippings/s1/proofOfDelivery/pin"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(fixture("200_shippings_id_proofofdelivery_pin.json")),
        )
        .mount(&server)
        .await;
    // "hola" in base64
    Mock::given(method("GET"))
        .and(path("/v3/shippings/s1/proofOfDelivery"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({ "proofOfDelivery": "aG9sYQ==" })),
        )
        .mount(&server)
        .await;

    let id = ShippingId::new("s1");
    assert_eq!(
        client.shippings().delivery_pin(&id).await.expect("pin").pin,
        "1357"
    );

    let proof = client
        .shippings()
        .proof_of_delivery(&id)
        .await
        .expect("proof");
    assert_eq!(proof.decode_jpeg().expect("base64"), b"hola");
    // The image itself never shows up in Debug output.
    assert!(!format!("{proof:?}").contains("aG9sYQ"));
}

#[tokio::test]
async fn labels_returns_pdf_bytes_and_validates_input() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/v3/shippings/labels"))
        .and(query_param("values", "1,2"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(b"%PDF-1.7 fake".to_vec(), "application/pdf"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let pdf = client
        .shippings()
        .labels(&[ShippingId::new("1"), ShippingId::new("2")])
        .await
        .expect("labels");
    assert!(pdf.starts_with(b"%PDF"));

    // Rejected locally, without calling the API.
    assert!(matches!(
        client.shippings().labels(&[]).await,
        Err(Error::InvalidInput(_))
    ));
    let too_many: Vec<_> = (0..51).map(|i| ShippingId::new(i.to_string())).collect();
    assert!(matches!(
        client.shippings().labels(&too_many).await,
        Err(Error::InvalidInput(_))
    ));
}

#[tokio::test]
async fn update_delivery_instructions() {
    let (server, client) = setup().await;
    Mock::given(method("PUT"))
        .and(path("/v3/shippings/s1/deliveryInstructions"))
        .and(body_json(json!({ "instructions": "Dejar en portería" })))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({ "instructions": "Dejar en portería" })),
        )
        .expect(1)
        .mount(&server)
        .await;

    let stored = client
        .shippings()
        .update_delivery_instructions(&ShippingId::new("s1"), "Dejar en portería")
        .await
        .expect("update");
    assert_eq!(stored, "Dejar en portería");
}

#[tokio::test]
async fn coverage_check() {
    let (server, client) = setup().await;
    Mock::given(method("POST"))
        .and(path("/v3/estimates/coverage"))
        .and(body_json(json!({ "waypoints": [
            { "type": "PICK_UP", "addressStreet": "Av. Winston Churchill 93", "city": "Santo Domingo" },
            { "type": "DROP_OFF", "latitude": 18.4734, "longitude": -69.884 }
        ]})))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixture("200_estimates_coverage.json")))
        .expect(1)
        .mount(&server)
        .await;

    let request = CoverageRequest::new([
        CoverageWaypoint::address(
            WaypointType::PickUp,
            "Av. Winston Churchill 93",
            "Santo Domingo",
        ),
        CoverageWaypoint::coordinates(WaypointType::DropOff, 18.4734, -69.884),
    ]);
    let result = client.coverage().check(&request).await.expect("coverage");
    assert_eq!(result.status, Some(200));
    assert_eq!(result.message.as_deref(), Some("OK"));
}

#[tokio::test]
async fn webhooks_set_get_delete() {
    let (server, client) = setup().await;
    let config = WebhooksConfig::new([WebhookSubscription::shipping_status(Environment::Test)
        .url(WebhookUrl::new("https://api.example.com/hook").authorization_key("s3cret"))]);
    let expected = json!({ "webhooksConfiguration": [{
        "isTest": true, "notificationType": "WEBHOOK", "topic": "SHIPPING_STATUS",
        "urls": [{ "url": "https://api.example.com/hook", "authorizationKey": "s3cret" }]
    }]});

    Mock::given(method("PUT"))
        .and(path("/v3/webhooks-configuration"))
        .and(body_json(&expected))
        .respond_with(ResponseTemplate::new(200).set_body_json(&expected))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/v3/webhooks-configuration"))
        .and(query_param("isTest", "true"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&expected))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/v3/webhooks-configuration"))
        .and(query_param("topic", "SHIPPING_STATUS"))
        .and(query_param("isTest", "true"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(&expected["webhooksConfiguration"][0]),
        )
        .expect(1)
        .mount(&server)
        .await;

    let stored = client.webhooks().set(&config).await.expect("set");
    assert_eq!(stored, config);
    assert_eq!(
        client
            .webhooks()
            .get_for(Environment::Test)
            .await
            .expect("get"),
        config
    );
    let deleted = client
        .webhooks()
        .delete(&WebhookTopic::ShippingStatus, Environment::Test)
        .await
        .expect("delete");
    assert_eq!(deleted.urls[0].url, "https://api.example.com/hook");
    // The secret never shows up in Debug output.
    assert!(!format!("{config:?}").contains("s3cret"));
}

#[tokio::test]
async fn api_error_with_code() {
    let (server, client) = setup().await;
    Mock::given(method("POST"))
        .and(path("/v3/shippings"))
        .respond_with(
            ResponseTemplate::new(400)
                .set_body_json(fixture("400_shippings_waypoints_not_found.json")),
        )
        .mount(&server)
        .await;

    let err = client
        .shippings()
        .create(&sample_request())
        .await
        .unwrap_err();
    assert_eq!(err.status(), Some(400));
    let api = err.as_api().expect("api error");
    assert_eq!(api.code(), Some(&ErrorCode::WaypointsNotFound));
    assert!(api.message().unwrap().contains("lat/long"));
    assert!(err.to_string().contains("WAYPOINTS_NOT_FOUND"));
}

#[tokio::test]
async fn api_error_without_code_and_with_details() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/v3/schedules"))
        .respond_with(ResponseTemplate::new(403).set_body_json(fixture("403_shippings.json")))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/v3/estimates/coverage"))
        .respond_with(
            ResponseTemplate::new(400).set_body_json(fixture("400_estimates_coverage.json")),
        )
        .mount(&server)
        .await;

    let err = client.schedules().await.unwrap_err();
    let api = err.as_api().expect("api error");
    assert!(api.is_unauthorized());
    assert_eq!(api.code(), None);
    assert_eq!(api.message(), Some("Invalid access token"));

    let err = client
        .coverage()
        .check(&CoverageRequest::new([]))
        .await
        .unwrap_err();
    let api = err.as_api().expect("api error");
    assert_eq!(api.code(), Some(&ErrorCode::MaxDistanceExceeded));
    assert_eq!(api.details().unwrap()["maxDistanceAllowed"], json!(10000));
}

#[tokio::test]
async fn non_json_error_body_is_kept() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/v3/working-zones"))
        .respond_with(ResponseTemplate::new(502).set_body_string("<html>Bad Gateway</html>"))
        .mount(&server)
        .await;

    let err = client.coverage().working_zones().await.unwrap_err();
    let api = err.as_api().expect("api error");
    assert_eq!(api.status(), 502);
    assert_eq!(api.message(), None);
    assert_eq!(api.body(), "<html>Bad Gateway</html>");
}

#[tokio::test]
async fn unexpected_success_body_is_a_decode_error() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/v3/schedules"))
        .respond_with(ResponseTemplate::new(200).set_body_string("not json"))
        .mount(&server)
        .await;

    match client.schedules().await {
        Err(Error::Decode { body, .. }) => assert_eq!(body, "not json"),
        other => panic!("expected decode error, got {other:?}"),
    }
}

#[test]
fn builder_validates_configuration() {
    assert!(matches!(Client::new("  "), Err(Error::Config(_))));
    assert!(matches!(Client::new("bad\ntoken"), Err(Error::Config(_))));
    assert!(matches!(
        Client::builder("t").base_url("not a url").build(),
        Err(Error::Config(_))
    ));
    assert!(matches!(
        Client::builder("t").base_url("ftp://example.com").build(),
        Err(Error::Config(_))
    ));

    let client = Client::new("super-secret-token").expect("client");
    assert_eq!(client.base_url(), "https://courier-api.pedidosya.com/");
    assert!(!format!("{client:?}").contains("super-secret-token"));
}

#[test]
fn unknown_enum_values_round_trip() {
    let status: ShippingStatus = serde_json::from_value(json!("ON_THE_MOON")).unwrap();
    assert_eq!(status, ShippingStatus::Unknown("ON_THE_MOON".into()));
    assert_eq!(serde_json::to_value(&status).unwrap(), json!("ON_THE_MOON"));
    assert_eq!(ShippingStatus::from("COMPLETED"), ShippingStatus::Completed);
    assert_eq!(ShippingStatus::NearDropoff.to_string(), "NEAR_DROPOFF");
}

// ----- Regressions found against the real API (see tests/live.rs) -----

#[test]
fn real_callbacks_response() {
    // Real API: `callbackUrl` (spec: `callBackUrl`), URL wrapped in brackets,
    // and `responseCode: -1` when the endpoint couldn't be reached.
    let value = fixture("live_callbacks.json");
    let server_response: serde_json::Value = value["callbacks"][0].clone();
    let invocation: pedidosya_courier::models::CallbackInvocation =
        serde_json::from_value(server_response).expect("callback invocation");
    assert_eq!(
        invocation.callback_url.as_deref(),
        Some("https://api.example.com/webhooks/pedidosya")
    );
    assert_eq!(invocation.response_code, Some(-1));
    assert!(!invocation.delivered());
}

#[tokio::test]
async fn real_callbacks_endpoint() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/v3/shippings/s1/callbacks"))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixture("live_callbacks.json")))
        .mount(&server)
        .await;
    let callbacks = client
        .shippings()
        .callbacks(&ShippingId::new("s1"))
        .await
        .expect("callbacks");
    assert_eq!(callbacks.len(), 2);
    assert_eq!(
        callbacks[1].event_code,
        Some(pedidosya_courier::models::ShippingEvent::Cancelled)
    );
}

#[test]
fn real_cancelled_shipping_keeps_undocumented_fields() {
    let shipping: pedidosya_courier::models::Shipping =
        serde_json::from_value(fixture("live_shipping_cancelled.json")).expect("shipping");
    assert_eq!(shipping.status, Some(ShippingStatus::Cancelled));
    assert_eq!(shipping.is_test, Some(true));
    assert_eq!(shipping.items[0].category_name.as_deref(), Some("Envío"));
    assert!(shipping.items[0].category_id.is_some());
    assert!(shipping.waypoints[1].last_updated_instructions.is_some());
    let pricing = shipping.route.unwrap().pricing.unwrap();
    assert_eq!(pricing.currency, Some(Currency::Dop));
    assert_eq!(pricing.total, Some(240.0));
}

#[test]
fn real_estimate_parses() {
    let estimate: pedidosya_courier::models::Estimate =
        serde_json::from_value(fixture("live_estimate.json")).expect("estimate");
    assert_eq!(
        estimate.delivery_offers[0].delivery_mode,
        Some(DeliveryMode::Express)
    );
    assert_eq!(estimate.route.unwrap().distance, Some(7206.0));
}

#[test]
fn real_schedules_cross_midnight() {
    let schedules: Vec<pedidosya_courier::models::Schedule> =
        serde_json::from_value(fixture("live_schedules.json")).expect("schedules");
    assert_eq!(schedules.len(), 7);
    let sunday = schedules[0];
    assert_eq!(sunday.day, Weekday::Sun);
    // 11:00 -> 03:00 UTC is 7:00 -> 23:00 in Santo Domingo.
    assert!(sunday.crosses_midnight());
    assert!(sunday.contains(NaiveTime::from_hms_opt(23, 30, 0).unwrap()));
    assert!(sunday.contains(NaiveTime::from_hms_opt(1, 0, 0).unwrap()));
    assert!(!sunday.contains(NaiveTime::from_hms_opt(5, 0, 0).unwrap()));
}

#[tokio::test]
async fn real_invalid_token_error() {
    // Real API: 400 with a `messages` list instead of `message`.
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/v3/schedules"))
        .respond_with(
            ResponseTemplate::new(400)
                .set_body_json(json!({ "code": "INVALID_PARAMS", "messages": ["invalid token"] })),
        )
        .mount(&server)
        .await;
    let err = client.schedules().await.unwrap_err();
    let api = err.as_api().expect("api error");
    assert_eq!(api.code(), Some(&ErrorCode::InvalidParams));
    assert_eq!(api.message(), Some("invalid token"));
    assert!(api.is_unauthorized());
}

#[tokio::test]
async fn list_rejects_bad_date_ranges_locally() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/v3/shippings"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({ "items": [], "nextPageId": null })),
        )
        .expect(2)
        .mount(&server)
        .await;
    let day = |d| NaiveDate::from_ymd_opt(2026, 11, d).unwrap();

    // Same day and the maximum range are fine.
    client
        .shippings()
        .list(&ListShippings::new(day(7), day(7)))
        .await
        .expect("same day");
    client
        .shippings()
        .list(&ListShippings::new(day(1), day(15)))
        .await
        .expect("14 days");

    // Too long or reversed: rejected without calling the API.
    for query in [
        ListShippings::new(day(1), day(16)),
        ListShippings::new(day(7), day(1)),
    ] {
        assert!(matches!(
            client.shippings().list(&query).await,
            Err(Error::InvalidInput(_))
        ));
    }
}
