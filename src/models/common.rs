use serde::{Deserialize, Serialize};

use crate::macros::{id_type, string_enum};

id_type! {
    /// Identifier of a shipping order, e.g. `"15562302161927091277656"`.
    ShippingId
}

id_type! {
    /// Identifier of a shipping estimate (quote), returned by
    /// [`Estimates::create`](crate::api::Estimates::create).
    EstimateId
}

id_type! {
    /// Identifier of one delivery offer inside an [`Estimate`](crate::models::Estimate).
    DeliveryOfferId
}

string_enum! {
    /// Status of a shipping order.
    pub enum ShippingStatus {
        /// Requested but rejected because of invalid data.
        Rejected => "REJECTED",
        /// Confirmed and waiting to be dispatched.
        Confirmed => "CONFIRMED",
        /// A rider has been assigned.
        InProgress => "IN_PROGRESS",
        /// The rider is near the pick-up point.
        NearPickup => "NEAR_PICKUP",
        /// The rider picked up the items.
        PickedUp => "PICKED_UP",
        /// The rider is near the drop-off point.
        NearDropoff => "NEAR_DROPOFF",
        /// Delivered.
        Completed => "COMPLETED",
        /// Cancelled; see the cancel code and reason.
        Cancelled => "CANCELLED",
        /// Expired before being completed (seen in callback examples).
        Expired => "EXPIRED",
    }
}

impl ShippingStatus {
    /// `true` once the shipping can no longer change: completed, cancelled,
    /// rejected or expired.
    #[must_use]
    pub fn is_final(&self) -> bool {
        matches!(
            self,
            Self::Completed | Self::Cancelled | Self::Rejected | Self::Expired
        )
    }
}

string_enum! {
    /// Why a shipping was cancelled. PedidosYa also sends a Spanish
    /// `cancel_reason` text alongside the code.
    pub enum CancelCode {
        /// The rider can't find the pick-up or drop-off address.
        AddressDataMissing => "ADDRESS_DATA_MISSING",
        /// No rider available at the moment.
        NoRiderAvailable => "NO_RIDER_AVAILABLE",
        /// Outside the delivery zone.
        OutOfDeliveryZone => "OUT_OF_DELIVERY_ZONE",
        /// The delivery schedule was delayed.
        DelayedDeliverySchedule => "DELAYED_DELIVERY_SCHEDULE",
        /// Wrong coordinates.
        CoordinateError => "COORDINATE_ERROR",
        /// The package was damaged or lost.
        PackageDamageLoose => "PACKAGE_DAMAGE_LOOSE",
        /// The order was not delivered.
        OrderNotDelivered => "ORDER_NOT_DELIVERED",
        /// Inappropriate conduct.
        InappropriateConduct => "INAPPROPRIATE_CONDUCT",
        /// The rider can't be reached.
        UnreachableRider => "UNREACHABLE_RIDER",
        /// The package contradicts the terms and conditions.
        TycPackageContradiction => "TYC_PACKAGE_CONTRADICTION",
        /// A purchase was requested.
        PurchaseRequested => "PURCHASE_REQUESTED",
        /// The customer can't pay.
        UserCannotPay => "USER_CANNOT_PAY",
        /// A coupon was not applied.
        CouponNotApplied => "COUPON_NOT_APPLIED",
        /// Duplicated order.
        DuplicatedOrder => "DUPLICATED_ORDER",
        /// The customer can't be reached at drop-off.
        UnreachableUserDropoff => "UNREACHABLE_USER_DROPOFF",
        /// Suspicious client.
        SuspiciousClient => "SUSPICIOUS_CLIENT",
        /// Cancelled by the user.
        UserCancelled => "USER_CANCELLED",
        /// Technical problem.
        TechnicalProblem => "TECHNICAL_PROBLEM",
        /// Bad weather.
        BadWeather => "BAD_WEATHER",
        /// The sender can't be reached at pick-up.
        UnreachableUserPickup => "UNREACHABLE_USER_PICKUP",
        /// Wrong package content.
        ContentWrong => "CONTENT_WRONG",
        /// The order was modified.
        OrderModification => "ORDER_MODIFICATION",
        /// Outside the fleet's working hours.
        OutOfFleetTime => "OUT_OF_FLEET_TIME",
        /// Test order.
        TestOrder => "TEST_ORDER",
        /// Wrong content, reported by the rider.
        ContentWrongRider => "CONTENT_WRONG_RIDER",
    }
}

string_enum! {
    /// How a shipping will be delivered.
    pub enum DeliveryMode {
        /// As soon as possible.
        Express => "EXPRESS",
        /// Within a scheduled time window.
        Scheduled => "SCHEDULED",
        /// Scheduled window (spelling used in some PedidosYa responses).
        Schedule => "SCHEDULE",
        /// Goes through a PedidosYa hub before delivery.
        CrossDocking => "CROSS_DOCKING",
    }
}

string_enum! {
    /// ISO 4217 currency of a price.
    pub enum Currency {
        /// Uruguayan peso.
        Uyu => "UYU",
        /// Chilean peso.
        Clp => "CLP",
        /// Argentine peso.
        Ars => "ARS",
        /// US dollar.
        Usd => "USD",
        /// Brazilian real.
        Brl => "BRL",
        /// Colombian peso.
        Cop => "COP",
        /// Peruvian sol.
        Pen => "PEN",
        /// Venezuelan bolívar.
        Ves => "VES",
        /// Mexican peso.
        Mxn => "MXN",
        /// Panamanian balboa.
        Pab => "PAB",
        /// Paraguayan guaraní.
        Pyg => "PYG",
        /// Costa Rican colón.
        Crc => "CRC",
        /// Bolivian boliviano.
        Bob => "BOB",
        /// Dominican peso.
        Dop => "DOP",
    }
}

/// Price of a delivery.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Pricing {
    /// Amount before taxes.
    // The spec says `subtotal`, real responses say `subTotal`: accept both.
    #[serde(rename = "subTotal", alias = "subtotal")]
    pub subtotal: Option<f64>,
    /// Taxes.
    pub taxes: Option<f64>,
    /// Total amount.
    pub total: Option<f64>,
    /// Currency of all amounts.
    pub currency: Option<Currency>,
}

/// Route and delivery details of a shipping.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Route {
    /// Delivery mode.
    pub delivery_mode: Option<DeliveryMode>,
    /// Estimated pick-up time (UTC).
    pub estimated_pick_up_time: Option<chrono::DateTime<chrono::Utc>>,
    /// Estimated delivery time (UTC).
    pub estimated_delivery_time: Option<chrono::DateTime<chrono::Utc>>,
    /// Estimated driving time from pick-up to drop-off, in minutes.
    pub estimated_driving_time: Option<u32>,
    /// Start of the promised delivery window (UTC).
    pub delivery_time_from: Option<chrono::DateTime<chrono::Utc>>,
    /// End of the promised delivery window (UTC).
    pub delivery_time_to: Option<chrono::DateTime<chrono::Utc>>,
    /// Total distance in meters.
    pub distance: Option<f64>,
    /// Price of the delivery.
    pub pricing: Option<Pricing>,
}

string_enum! {
    /// What the rider does at a waypoint.
    pub enum WaypointType {
        /// Collect the items here.
        PickUp => "PICK_UP",
        /// Deliver the items here.
        DropOff => "DROP_OFF",
    }
}

/// A pick-up or drop-off point.
///
/// Used both in requests and responses. Build one with [`Waypoint::pick_up`]
/// or [`Waypoint::drop_off`].
///
/// ```
/// use pedidosya_courier::models::Waypoint;
///
/// let dropoff = Waypoint::drop_off("Calle El Conde 45", "Santo Domingo", "María", "+18095551234")
///     .address_additional("Apto 2B")
///     .coordinates(18.4734, -69.8840)
///     .instructions("Tocar el timbre");
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Waypoint {
    /// Pick-up or drop-off.
    #[serde(rename = "type")]
    pub kind: WaypointType,
    /// Street and door number.
    pub address_street: String,
    /// Second address line (apartment, floor, ...).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address_additional: Option<String>,
    /// City name, e.g. `"Santo Domingo"`.
    pub city: String,
    /// Latitude. If missing, PedidosYa geocodes the address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latitude: Option<f64>,
    /// Longitude. If missing, PedidosYa geocodes the address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub longitude: Option<f64>,
    /// Contact phone: optional `+` prefix followed by digits.
    pub phone: String,
    /// Contact person.
    pub name: String,
    /// Instructions for the rider at this point.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
    /// Cash the rider collects at drop-off (only if cash collection is
    /// enabled for the account).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collect_money: Option<f64>,
    /// Cash the rider pays at pick-up (responses only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pay_money: Option<f64>,
    /// Delivery fee the rider collects at drop-off (responses only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collect_delivery_fee_money: Option<f64>,
    /// Delivery fee the rider pays at pick-up (responses only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pay_delivery_fee_money: Option<f64>,
    /// When the instructions were last changed with
    /// [`Shippings::update_delivery_instructions`](crate::api::Shippings::update_delivery_instructions)
    /// (responses only, not in the spec).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_updated_instructions: Option<chrono::DateTime<chrono::Utc>>,
}

impl Waypoint {
    /// A pick-up point.
    pub fn pick_up(
        address_street: impl Into<String>,
        city: impl Into<String>,
        name: impl Into<String>,
        phone: impl Into<String>,
    ) -> Self {
        Self::new(WaypointType::PickUp, address_street, city, name, phone)
    }

    /// A drop-off point.
    pub fn drop_off(
        address_street: impl Into<String>,
        city: impl Into<String>,
        name: impl Into<String>,
        phone: impl Into<String>,
    ) -> Self {
        Self::new(WaypointType::DropOff, address_street, city, name, phone)
    }

    fn new(
        kind: WaypointType,
        address_street: impl Into<String>,
        city: impl Into<String>,
        name: impl Into<String>,
        phone: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            address_street: address_street.into(),
            address_additional: None,
            city: city.into(),
            latitude: None,
            longitude: None,
            phone: phone.into(),
            name: name.into(),
            instructions: None,
            collect_money: None,
            pay_money: None,
            collect_delivery_fee_money: None,
            pay_delivery_fee_money: None,
            last_updated_instructions: None,
        }
    }

    /// Sets the second address line (apartment, floor, ...).
    #[must_use]
    pub fn address_additional(mut self, value: impl Into<String>) -> Self {
        self.address_additional = Some(value.into());
        self
    }

    /// Sets exact coordinates, so PedidosYa doesn't have to geocode the address.
    #[must_use]
    pub fn coordinates(mut self, latitude: f64, longitude: f64) -> Self {
        self.latitude = Some(latitude);
        self.longitude = Some(longitude);
        self
    }

    /// Sets instructions for the rider at this point.
    #[must_use]
    pub fn instructions(mut self, value: impl Into<String>) -> Self {
        self.instructions = Some(value.into());
        self
    }

    /// Sets the cash amount the rider collects at drop-off.
    ///
    /// Only valid on drop-off points, and only if cash collection is enabled
    /// for the PedidosYa account.
    #[must_use]
    pub fn collect_money(mut self, amount: f64) -> Self {
        self.collect_money = Some(amount);
        self
    }
}

string_enum! {
    /// Kind of item, used by PedidosYa to choose the right vehicle.
    pub enum ItemType {
        /// Regular item.
        Standard => "STANDARD",
        /// Needs careful handling.
        Fragile => "FRAGILE",
        /// Needs to stay cold.
        Cold => "COLD",
    }
}

/// An item inside a shipping.
///
/// ```
/// use pedidosya_courier::models::{Item, ItemType};
///
/// let item = Item::new("Pica pollo familiar", 850.0)
///     .quantity(2)
///     .volume_cm3(4_000.0)
///     .weight_kg(1.5)
///     .kind(ItemType::Standard);
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Item {
    /// Item type.
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub kind: Option<ItemType>,
    /// What is being shipped.
    pub description: String,
    /// Value of one unit (between 0 and 1,000,000).
    pub value: f64,
    /// Number of units (1 to 10,000).
    pub quantity: u32,
    /// Volume of one unit in cm³ (0 to 80,840).
    pub volume: f64,
    /// Weight of one unit in kg (0 to 20).
    pub weight: f64,
    /// Your SKU, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sku: Option<String>,
    /// PedidosYa category ID. Optional in requests; PedidosYa assigns one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category_id: Option<u64>,
    /// PedidosYa category name, e.g. `"Envío"` (responses only, not in the spec).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category_name: Option<String>,
}

impl Item {
    /// A new item with its description and the value of one unit.
    ///
    /// Defaults: quantity `1`, volume `0` cm³, weight `0` kg. PedidosYa uses
    /// volume and weight to pick a vehicle, so set them when you know them.
    pub fn new(description: impl Into<String>, value: f64) -> Self {
        Self {
            kind: None,
            description: description.into(),
            value,
            quantity: 1,
            volume: 0.0,
            weight: 0.0,
            sku: None,
            category_id: None,
            category_name: None,
        }
    }

    /// Sets the number of units.
    #[must_use]
    pub fn quantity(mut self, quantity: u32) -> Self {
        self.quantity = quantity;
        self
    }

    /// Sets the volume of one unit, in cm³.
    #[must_use]
    pub fn volume_cm3(mut self, volume: f64) -> Self {
        self.volume = volume;
        self
    }

    /// Sets the weight of one unit, in kg.
    #[must_use]
    pub fn weight_kg(mut self, weight: f64) -> Self {
        self.weight = weight;
        self
    }

    /// Sets the item type.
    #[must_use]
    pub fn kind(mut self, kind: ItemType) -> Self {
        self.kind = Some(kind);
        self
    }

    /// Sets your SKU.
    #[must_use]
    pub fn sku(mut self, sku: impl Into<String>) -> Self {
        self.sku = Some(sku.into());
        self
    }
}

/// Whether to use PedidosYa's production or test environment.
///
/// Test shippings are never delivered, and PedidosYa sends their status
/// callbacks every minute until they complete.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Environment {
    /// Real shippings.
    #[default]
    Production,
    /// Test shippings and test webhooks.
    Test,
}

impl Environment {
    pub(crate) fn is_test(self) -> bool {
        matches!(self, Self::Test)
    }
}
