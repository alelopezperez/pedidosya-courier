use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::common::{DeliveryMode, DeliveryOfferId, EstimateId, Item, Pricing, Route, Waypoint};

/// A quote for a shipping, with one or more delivery offers to choose from.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Estimate {
    /// Estimate ID; confirm it with [`Estimates::confirm`](crate::api::Estimates::confirm).
    pub estimate_id: EstimateId,
    /// Your reference.
    pub reference_id: Option<String>,
    /// Whether this is a test estimate.
    pub is_test: Option<bool>,
    /// Items in the package.
    #[serde(default)]
    pub items: Vec<Item>,
    /// Pick-up and drop-off points.
    #[serde(default)]
    pub waypoints: Vec<Waypoint>,
    /// Available delivery offers (express, scheduled windows, ...).
    #[serde(default)]
    pub delivery_offers: Vec<DeliveryOffer>,
    /// Route information (distance).
    // The spec says `route`, its example says `routes`: accept both.
    #[serde(alias = "routes")]
    pub route: Option<Route>,
    /// Email receiving status notifications.
    pub notification_mail: Option<String>,
}

/// One way the estimated shipping can be delivered, with its price.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct DeliveryOffer {
    /// Offer ID, to confirm this specific offer.
    pub delivery_offer_id: DeliveryOfferId,
    /// Delivery mode.
    pub delivery_mode: Option<DeliveryMode>,
    /// Estimated pick-up time (UTC).
    pub estimated_pick_up_time: Option<DateTime<Utc>>,
    /// Estimated driving time from pick-up to drop-off, in minutes.
    pub estimated_driving_time: Option<u32>,
    /// Start of the promised delivery window (UTC).
    pub delivery_time_from: Option<DateTime<Utc>>,
    /// End of the promised delivery window (UTC).
    pub delivery_time_to: Option<DateTime<Utc>>,
    /// The offer can't be confirmed after this time (UTC).
    pub confirmation_time_limit: Option<DateTime<Utc>>,
    /// Price of this offer.
    pub pricing: Option<Pricing>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ConfirmEstimateBody<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) delivery_offer_id: Option<&'a DeliveryOfferId>,
}
