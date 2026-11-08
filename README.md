# pedidosya-courier

Unofficial async Rust client for the [PedidosYa Courier API](https://developers.pedidosya.com/api-doc) (v3).

Create shippings, get quotes, check coverage, track riders, download labels,
and configure and receive status webhooks.

> Not affiliated with or endorsed by PedidosYa.

## Install

```toml
[dependencies]
pedidosya-courier = "0.1"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

## Usage

Generate a token in the PedidosYa Envíos dashboard (**Integraciones → Generar
token**). It doesn't expire; each account (branch) has its own.

```rust,no_run
use pedidosya_courier::Client;
use pedidosya_courier::models::{Environment, Item, ShippingRequest, Waypoint};

#[tokio::main]
async fn main() -> pedidosya_courier::Result<()> {
    let client = Client::new(std::env::var("PEDIDOSYA_TOKEN").unwrap_or_default())?;

    let request = ShippingRequest::new(
        "pedido-1042",
        Waypoint::pick_up("Av. Winston Churchill 93", "Santo Domingo", "Pica Pollo Doña Carmen", "+18095550000"),
        Waypoint::drop_off("Calle El Conde 45", "Santo Domingo", "María", "+18095551234"),
    )
    .item(Item::new("Pica pollo familiar", 850.0).quantity(2))
    .environment(Environment::Test);

    // Quote, then confirm the first delivery offer.
    let estimate = client.estimates().create(&request).await?;
    let offer = estimate.delivery_offers.first().map(|o| &o.delivery_offer_id);
    let shipping = client.estimates().confirm(&estimate.estimate_id, offer).await?;

    println!("{} is {:?}", shipping.shipping_id, shipping.status);
    Ok(())
}
```

More in [`examples/`](examples/) and in the API docs.

## What's covered

| Client method | Endpoints |
|---|---|
| `shippings()` | create, get, list (paginated), cancel, tracking, proof of delivery, delivery PIN, delivery instructions, callback history, labels (PDF) |
| `estimates()` | create a quote, confirm it |
| `coverage()` | check coverage, working zones |
| `webhooks()` | get, set, delete webhook configuration |
| `schedules()` | fleet working hours |
| `webhook` module | parse incoming callbacks, verify their `Authorization` key |

## Receiving webhooks

PedidosYa doesn't sign callbacks. Instead, it sends the `authorizationKey` you
configured as the `Authorization` header. Check it before trusting the body:

```rust,ignore
use pedidosya_courier::webhook;

let header = headers.get("authorization").and_then(|v| v.to_str().ok());
if !webhook::verify_authorization(header, &my_key) {
    return StatusCode::UNAUTHORIZED;
}
let callback = webhook::parse(&body)?;
println!("{} -> {}", callback.id, callback.data.status);
```

## Design notes

- **Forward compatible.** PedidosYa's own examples contain enum values missing
  from its spec (e.g. `CROSS_DOCKING`). Enums keep unknown values in an
  `Unknown(String)` variant instead of failing, and response types are
  `#[non_exhaustive]`.
- **Typed IDs.** `ShippingId`, `EstimateId` and `DeliveryOfferId` can't be mixed up.
- **No runtime lock-in.** The crate doesn't depend on tokio directly; it works
  wherever `reqwest` does.
- **Secrets stay out of logs.** `Debug` output redacts the token and webhook keys.

## Features

| Feature | Default | Description |
|---|---|---|
| `rustls` | yes | TLS with rustls (pure Rust) |
| `native-tls` | no | TLS with the platform's library (OpenSSL on Linux) |

## Minimum supported Rust version

1.85.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.
