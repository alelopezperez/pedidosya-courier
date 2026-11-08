//! Request and response types.
//!
//! Response types are `#[non_exhaustive]` with public fields: read them
//! freely; new fields can be added in minor releases. Request types are built
//! with constructors and chained setters (e.g. [`ShippingRequest::new`]).

mod common;
mod coverage;
mod estimate;
mod shipping;
mod webhooks;

pub use common::{
    CancelCode, Currency, DeliveryMode, DeliveryOfferId, Environment, EstimateId, Item, ItemType,
    Pricing, Route, ShippingId, ShippingStatus, Waypoint, WaypointType,
};
pub use coverage::{
    CoverageRequest, CoverageResult, CoverageWaypoint, Polygon, Schedule, WorkingZone,
};
pub use estimate::{DeliveryOffer, Estimate};
pub use shipping::{
    DeliveryPin, ListShippings, ProofOfDelivery, Requirements, Shipping, ShippingPage,
    ShippingRequest, Tracking,
};
pub use webhooks::{
    CallbackInvocation, NotificationType, ShippingEvent, WebhookSubscription, WebhookTopic,
    WebhookUrl, WebhooksConfig,
};

pub(crate) use estimate::ConfirmEstimateBody;
pub(crate) use webhooks::CallbackInvocations;
