//! Quote a test shipping, confirm it, then follow it.
//!
//! ```sh
//! PEDIDOSYA_TOKEN=... cargo run --example create_shipping
//! ```
//!
//! Uses `Environment::Test`, so nothing is actually delivered.

use pedidosya_courier::models::{Environment, Item, ShippingRequest, Waypoint};
use pedidosya_courier::{Client, Error, ErrorCode};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let token = std::env::var("PEDIDOSYA_TOKEN").map_err(|_| "set PEDIDOSYA_TOKEN first")?;
    let client = Client::new(token)?;

    let request = ShippingRequest::new(
        "example-order-1",
        Waypoint::pick_up(
            "Plaza Independencia 755",
            "Montevideo",
            "Oficina",
            "+59898765432",
        ),
        Waypoint::drop_off(
            "La Cumparsita 1475",
            "Montevideo",
            "Agustín",
            "+59812345678",
        )
        .instructions("Entregar en mano"),
    )
    .item(Item::new("Libro", 1250.6).volume_cm3(10.0).weight_kg(0.5))
    .environment(Environment::Test);

    let estimate = match client.estimates().create(&request).await {
        Ok(estimate) => estimate,
        Err(Error::Api(e)) if e.code() == Some(&ErrorCode::WaypointsOutOfZone) => {
            println!("Those addresses are outside PedidosYa's delivery zones.");
            return Ok(());
        }
        Err(e) => return Err(e.into()),
    };

    println!("Estimate {}:", estimate.estimate_id);
    for offer in &estimate.delivery_offers {
        let price = offer.pricing.as_ref().and_then(|p| p.total);
        println!(
            "  {} {:?} -> {:?}",
            offer.delivery_offer_id, offer.delivery_mode, price
        );
    }

    let offer = estimate
        .delivery_offers
        .first()
        .map(|o| &o.delivery_offer_id);
    let shipping = client
        .estimates()
        .confirm(&estimate.estimate_id, offer)
        .await?;
    println!("Shipping {} is {:?}", shipping.shipping_id, shipping.status);
    if let Some(url) = &shipping.share_location_url {
        println!("Tracking page: {url}");
    }

    let latest = client.shippings().get(&shipping.shipping_id).await?;
    println!("Current status: {:?}", latest.status);
    Ok(())
}
