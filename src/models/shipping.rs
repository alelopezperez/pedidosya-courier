use base64::Engine as _;
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

use super::common::{
    CancelCode, Environment, EstimateId, Item, Route, ShippingId, ShippingStatus, Waypoint,
};
use crate::error::{Error, Result};

/// A shipping order (or estimate) to send to PedidosYa.
///
/// The same request shape is used to create a shipping directly
/// ([`Shippings::create`](crate::api::Shippings::create)) or to ask for a
/// quote first ([`Estimates::create`](crate::api::Estimates::create)).
///
/// ```
/// use pedidosya_courier::models::{Item, ShippingRequest, Waypoint};
///
/// let request = ShippingRequest::new(
///     "pedido-1042",
///     Waypoint::pick_up("Av. Winston Churchill 93", "Santo Domingo", "Pica Pollo Doña Carmen", "+18095550000"),
///     Waypoint::drop_off("Calle El Conde 45", "Santo Domingo", "María", "+18095551234"),
/// )
/// .item(Item::new("Pica pollo familiar", 850.0).quantity(2))
/// .notification_mail("maria@example.com");
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ShippingRequest {
    /// Your own reference (order number). Riders use it to identify the
    /// package at pick-up.
    pub reference_id: String,
    /// Desired delivery time (UTC), for scheduled deliveries.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delivery_time: Option<DateTime<Utc>>,
    /// `true` creates a test shipping that is never delivered.
    pub is_test: bool,
    /// Items in the package (1 to 99).
    pub items: Vec<Item>,
    /// Exactly one pick-up and one drop-off.
    pub waypoints: Vec<Waypoint>,
    /// Email that receives status notifications for the end customer.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notification_mail: Option<String>,
    /// Extra requirements.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requirements: Option<Requirements>,
}

impl ShippingRequest {
    /// A new request with your reference ID and both waypoints. Add items
    /// with [`item`](Self::item) or [`items`](Self::items).
    pub fn new(reference_id: impl Into<String>, pick_up: Waypoint, drop_off: Waypoint) -> Self {
        Self {
            reference_id: reference_id.into(),
            delivery_time: None,
            is_test: false,
            items: Vec::new(),
            waypoints: vec![pick_up, drop_off],
            notification_mail: None,
            requirements: None,
        }
    }

    /// Adds one item.
    #[must_use]
    pub fn item(mut self, item: Item) -> Self {
        self.items.push(item);
        self
    }

    /// Adds several items.
    #[must_use]
    pub fn items(mut self, items: impl IntoIterator<Item = Item>) -> Self {
        self.items.extend(items);
        self
    }

    /// Chooses production or test. Defaults to production.
    #[must_use]
    pub fn environment(mut self, environment: Environment) -> Self {
        self.is_test = environment.is_test();
        self
    }

    /// Requests delivery at a specific time (UTC) instead of as soon as possible.
    #[must_use]
    pub fn delivery_time(mut self, at: DateTime<Utc>) -> Self {
        self.delivery_time = Some(at);
        self
    }

    /// Sets the email that receives status notifications.
    #[must_use]
    pub fn notification_mail(mut self, email: impl Into<String>) -> Self {
        self.notification_mail = Some(email.into());
        self
    }

    /// Includes the delivery fee in the cash the rider collects (only if
    /// cash collection is enabled for the account).
    #[must_use]
    pub fn include_delivery_fee(mut self, include: bool) -> Self {
        self.requirements = Some(Requirements {
            include_delivery_fee: Some(include),
        });
        self
    }
}

/// Extra requirements of a shipping.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Requirements {
    /// Include the delivery fee in the cash collected by the rider.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_delivery_fee: Option<bool>,
}

/// A shipping order as returned by PedidosYa.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Shipping {
    /// Shipping ID.
    pub shipping_id: ShippingId,
    /// The estimate this shipping came from, if it was confirmed from one.
    pub estimate_id: Option<EstimateId>,
    /// Your reference.
    pub reference_id: Option<String>,
    /// Whether this is a test shipping.
    pub is_test: Option<bool>,
    /// Current status.
    pub status: Option<ShippingStatus>,
    /// Confirmation code, set once the shipping is confirmed.
    pub confirmation_code: Option<String>,
    /// Why it was cancelled (only when cancelled).
    pub cancel_code: Option<CancelCode>,
    /// Spanish text for the cancel code (only when cancelled).
    pub cancel_reason: Option<String>,
    /// Whether a proof of delivery (signature or photo) is available.
    pub proof_of_delivery: Option<bool>,
    /// Public tracking page you can share with the customer.
    pub share_location_url: Option<String>,
    /// Email receiving status notifications.
    pub notification_mail: Option<String>,
    /// Items in the package.
    #[serde(default)]
    pub items: Vec<Item>,
    /// Pick-up and drop-off points.
    #[serde(default)]
    pub waypoints: Vec<Waypoint>,
    /// Route, times and price.
    pub route: Option<Route>,
    /// When the shipping was created (UTC).
    pub created_at: Option<DateTime<Utc>>,
    /// Last change of data or status (UTC).
    pub last_updated: Option<DateTime<Utc>>,
}

/// Filter for [`Shippings::list`](crate::api::Shippings::list).
///
/// ```
/// use chrono::NaiveDate;
/// use pedidosya_courier::models::ListShippings;
///
/// let query = ListShippings::new(
///     NaiveDate::from_ymd_opt(2026, 11, 1).unwrap(),
///     NaiveDate::from_ymd_opt(2026, 11, 7).unwrap(),
/// );
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ListShippings {
    /// First day to include.
    pub from_date: NaiveDate,
    /// Last day to include.
    pub to_date: NaiveDate,
    /// Page cursor from a previous [`ShippingPage::next_page_id`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_page_id: Option<String>,
}

impl ListShippings {
    /// Maximum days between `from_date` and `to_date` accepted by PedidosYa
    /// (its error message says 15; a 15-day difference is rejected in practice).
    pub const MAX_RANGE_DAYS: i64 = 14;

    pub(crate) fn validate(&self) -> Result<()> {
        let days = (self.to_date - self.from_date).num_days();
        if days < 0 {
            return Err(Error::InvalidInput(format!(
                "from_date ({}) is after to_date ({})",
                self.from_date, self.to_date
            )));
        }
        if days > Self::MAX_RANGE_DAYS {
            return Err(Error::InvalidInput(format!(
                "date range is {days} days; PedidosYa allows at most {}",
                Self::MAX_RANGE_DAYS
            )));
        }
        Ok(())
    }

    /// Shippings created between two dates.
    #[must_use]
    pub fn new(from_date: NaiveDate, to_date: NaiveDate) -> Self {
        Self {
            from_date,
            to_date,
            next_page_id: None,
        }
    }

    /// Continues from a previous page.
    #[must_use]
    pub fn page(mut self, next_page_id: impl Into<String>) -> Self {
        self.next_page_id = Some(next_page_id.into());
        self
    }
}

/// One page of shippings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ShippingPage {
    /// Shippings on this page.
    #[serde(default)]
    pub items: Vec<Shipping>,
    /// Cursor for the next page; `None` on the last page.
    pub next_page_id: Option<String>,
}

impl ShippingPage {
    /// The query for the next page, or `None` if this is the last one.
    #[must_use]
    pub fn next_query(&self, current: &ListShippings) -> Option<ListShippings> {
        self.next_page_id
            .as_ref()
            .map(|id| current.clone().page(id.clone()))
    }
}

/// Live position of the rider for a shipping.
///
/// After a shipping is cancelled PedidosYa still answers, with placeholder
/// values: name and transport `"Order Cancelled"` and coordinates `0, 0`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Tracking {
    /// Rider latitude.
    pub latitude: Option<f64>,
    /// Rider longitude.
    pub longitude: Option<f64>,
    /// Rider name.
    pub delivery_name: Option<String>,
    /// Estimated pick-up time (UTC), updated live.
    pub estimated_pick_up_time: Option<DateTime<Utc>>,
    /// Estimated drop-off time (UTC), updated live.
    pub estimated_drop_off_time: Option<DateTime<Utc>>,
    /// Vehicle, e.g. `"Bike"` or `"Motorbike"`.
    pub delivery_transport: Option<String>,
}

/// Proof of delivery (signature or photo) of a completed shipping.
#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ProofOfDelivery {
    /// JPG image, base64-encoded.
    #[serde(rename = "proofOfDelivery")]
    pub image_base64: String,
}

impl ProofOfDelivery {
    /// Decodes the JPG image bytes.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Base64`] if the data is not valid base64.
    pub fn decode_jpeg(&self) -> Result<Vec<u8>> {
        decode_base64(&self.image_base64)
    }
}

impl std::fmt::Debug for ProofOfDelivery {
    // The image can be hundreds of kilobytes: show its size, not its content.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProofOfDelivery")
            .field(
                "image_base64",
                &format_args!("<{} bytes>", self.image_base64.len()),
            )
            .finish()
    }
}

/// PIN the customer gives the rider to confirm delivery.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct DeliveryPin {
    /// The PIN.
    pub pin: String,
}

pub(crate) fn decode_base64(data: &str) -> Result<Vec<u8>> {
    // Tolerate a `data:image/jpeg;base64,` prefix.
    let data = data.split_once("base64,").map_or(data, |(_, rest)| rest);
    base64::engine::general_purpose::STANDARD
        .decode(data.trim())
        .map_err(|e| Error::Base64(e.to_string()))
}
