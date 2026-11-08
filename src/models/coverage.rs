use chrono::{DateTime, NaiveTime, Timelike, Utc, Weekday};
use serde::{Deserialize, Deserializer, Serialize};

use super::common::WaypointType;
use super::shipping::decode_base64;
use crate::error::Result;

/// Request for [`Coverage::check`](crate::api::Coverage::check): are these
/// points inside PedidosYa's delivery zones?
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct CoverageRequest {
    /// Points to check. Only one pick-up is allowed.
    pub waypoints: Vec<CoverageWaypoint>,
}

impl CoverageRequest {
    /// Checks a pick-up point and one or more drop-off points.
    pub fn new(waypoints: impl IntoIterator<Item = CoverageWaypoint>) -> Self {
        Self {
            waypoints: waypoints.into_iter().collect(),
        }
    }
}

/// A point to check for coverage: an address and city, or coordinates.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct CoverageWaypoint {
    /// Pick-up or drop-off.
    #[serde(rename = "type")]
    pub kind: WaypointType,
    /// Street and door number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address_street: Option<String>,
    /// City name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub city: Option<String>,
    /// Latitude.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latitude: Option<f64>,
    /// Longitude.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub longitude: Option<f64>,
}

impl CoverageWaypoint {
    /// A point identified by its address.
    pub fn address(
        kind: WaypointType,
        address_street: impl Into<String>,
        city: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            address_street: Some(address_street.into()),
            city: Some(city.into()),
            latitude: None,
            longitude: None,
        }
    }

    /// A point identified by its coordinates.
    #[must_use]
    pub fn coordinates(kind: WaypointType, latitude: f64, longitude: f64) -> Self {
        Self {
            kind,
            address_street: None,
            city: None,
            latitude: Some(latitude),
            longitude: Some(longitude),
        }
    }
}

/// Result of a coverage check. An error response (for example
/// `WAYPOINTS_OUT_OF_ZONE`) means the points are not covered.
#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct CoverageResult {
    /// Status code inside the body (`200` when covered).
    pub status: Option<u16>,
    /// Message, e.g. `"OK"`.
    pub message: Option<String>,
    /// Optional JPG map of the points, base64-encoded.
    pub coverage_static_map: Option<String>,
}

impl CoverageResult {
    /// Decodes the map image, if PedidosYa sent one.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Base64`](crate::Error::Base64) if the data is not valid base64.
    pub fn decode_static_map(&self) -> Result<Option<Vec<u8>>> {
        self.coverage_static_map
            .as_deref()
            .map(decode_base64)
            .transpose()
    }
}

impl std::fmt::Debug for CoverageResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CoverageResult")
            .field("status", &self.status)
            .field("message", &self.message)
            .field(
                "coverage_static_map",
                &self
                    .coverage_static_map
                    .as_ref()
                    .map(|m| format!("<{} bytes>", m.len())),
            )
            .finish()
    }
}

/// A zone where PedidosYa delivers.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct WorkingZone {
    /// Zone name.
    pub name: Option<String>,
    /// Zone shape.
    pub polygon: Option<Polygon>,
    /// Format of the polygon, e.g. `"GEOJSON"`.
    pub polygon_format: Option<String>,
    /// Last change (UTC).
    pub last_updated: Option<DateTime<Utc>>,
}

/// A GeoJSON-style polygon.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Polygon {
    /// Geometry type, usually `"Polygon"`.
    #[serde(rename = "type")]
    pub kind: Option<String>,
    /// Rings of `[longitude, latitude]` positions, as in GeoJSON.
    #[serde(default)]
    pub coordinates: Vec<Vec<Vec<f64>>>,
}

/// A day and time window (UTC) in which PedidosYa's fleet delivers.
///
/// Windows can cross midnight UTC: in the Dominican Republic (UTC-4) a
/// 7:00-23:00 local schedule is `from = 11:00`, `to = 03:00` (next day).
/// Check [`crosses_midnight`](Self::crosses_midnight).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Schedule {
    /// Day of the week.
    #[serde(with = "iso_weekday")]
    pub day: Weekday,
    /// Start time (UTC).
    #[serde(deserialize_with = "time_of_datetime")]
    pub from: NaiveTime,
    /// End time (UTC).
    #[serde(deserialize_with = "time_of_datetime")]
    pub to: NaiveTime,
}

impl Schedule {
    /// `true` if the window ends on the next day (UTC), i.e. `to <= from`.
    #[must_use]
    pub fn crosses_midnight(&self) -> bool {
        self.to <= self.from
    }

    /// `true` if `time` (UTC) falls inside the window.
    #[must_use]
    pub fn contains(&self, time: NaiveTime) -> bool {
        if self.crosses_midnight() {
            time >= self.from || time < self.to
        } else {
            time >= self.from && time < self.to
        }
    }
}

/// The API sends times as full timestamps on a dummy date
/// (`2010-01-01T08:00:00Z`); only the time of day matters.
fn time_of_datetime<'de, D: Deserializer<'de>>(deserializer: D) -> Result<NaiveTime, D::Error> {
    let at = DateTime::<Utc>::deserialize(deserializer)?;
    Ok(at.time().with_nanosecond(0).unwrap_or_else(|| at.time()))
}

/// ISO weekday numbers: Monday = 1 ... Sunday = 7.
mod iso_weekday {
    use chrono::Weekday;
    use serde::{Deserialize, Deserializer, Serializer, de::Error};

    #[allow(
        clippy::trivially_copy_pass_by_ref,
        reason = "serde `with` requires `&T`"
    )]
    pub(super) fn serialize<S: Serializer>(
        day: &Weekday,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        serializer.serialize_u32(day.number_from_monday())
    }

    pub(super) fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Weekday, D::Error> {
        let n = u8::deserialize(deserializer)?;
        match n {
            1 => Ok(Weekday::Mon),
            2 => Ok(Weekday::Tue),
            3 => Ok(Weekday::Wed),
            4 => Ok(Weekday::Thu),
            5 => Ok(Weekday::Fri),
            6 => Ok(Weekday::Sat),
            7 => Ok(Weekday::Sun),
            other => Err(D::Error::custom(format!(
                "invalid ISO weekday {other}, expected 1-7"
            ))),
        }
    }
}
