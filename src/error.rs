//! Error types.

use std::fmt::Write as _;

use serde::Deserialize;

use crate::macros::string_enum;

/// Shorthand for `Result<T, pedidosya_courier::Error>`.
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Everything that can go wrong when calling the PedidosYa Courier API.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// PedidosYa answered with an error status (4xx or 5xx).
    #[error(transparent)]
    Api(#[from] ApiError),

    /// The request never got a response: connection, TLS, timeout, ...
    #[error(transparent)]
    Http(#[from] HttpError),

    /// The response arrived but its body didn't match the expected shape.
    #[error("failed to decode response body: {source}")]
    Decode {
        /// The underlying JSON error.
        source: serde_json::Error,
        /// The raw response body, to help debugging.
        body: String,
    },

    /// A base64 field (an image) could not be decoded.
    #[error("invalid base64 data: {0}")]
    Base64(String),

    /// The client was configured with invalid values (token, base URL, ...).
    #[error("invalid client configuration: {0}")]
    Config(String),

    /// An argument was rejected before sending the request.
    #[error("invalid input: {0}")]
    InvalidInput(String),
}

impl Error {
    /// Returns the API error if this is an error response from PedidosYa.
    #[must_use]
    pub fn as_api(&self) -> Option<&ApiError> {
        match self {
            Self::Api(err) => Some(err),
            _ => None,
        }
    }

    /// The HTTP status code, if PedidosYa answered with an error status.
    pub fn status(&self) -> Option<u16> {
        self.as_api().map(ApiError::status)
    }
}

/// An error response returned by PedidosYa (any 4xx or 5xx status).
///
/// PedidosYa error bodies look like
/// `{"status": 400, "message": "...", "code": "WAYPOINTS_NOT_FOUND"}`;
/// `code` is missing on some errors (for example `403 Invalid access token`).
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[error("{}", self.describe())]
pub struct ApiError {
    status: u16,
    code: Option<ErrorCode>,
    message: Option<String>,
    details: Option<serde_json::Value>,
    body: String,
}

impl ApiError {
    pub(crate) fn from_response(status: u16, body: String) -> Self {
        #[derive(Deserialize)]
        struct Body {
            code: Option<ErrorCode>,
            message: Option<String>,
            // Some errors (e.g. an invalid token) send a list instead:
            // `{"code": "INVALID_PARAMS", "messages": ["invalid token"]}`.
            #[serde(default)]
            messages: Vec<String>,
            details: Option<serde_json::Value>,
        }

        let parsed = serde_json::from_str::<Body>(&body).ok();
        let (code, message, details) = match parsed {
            Some(b) => {
                let message = b
                    .message
                    .or_else(|| (!b.messages.is_empty()).then(|| b.messages.join("; ")));
                (b.code, message, b.details)
            }
            None => (None, None, None),
        };
        Self {
            status,
            code,
            message,
            details,
            body,
        }
    }

    /// HTTP status code, e.g. `400`.
    #[must_use]
    pub fn status(&self) -> u16 {
        self.status
    }

    /// Machine-readable error code, when PedidosYa sends one.
    #[must_use]
    pub fn code(&self) -> Option<&ErrorCode> {
        self.code.as_ref()
    }

    /// Human-readable message from PedidosYa (in English or Spanish).
    #[must_use]
    pub fn message(&self) -> Option<&str> {
        self.message.as_deref()
    }

    /// Extra structured details, e.g. `{"maxDistanceAllowed": 10000,
    /// "totalDistance": 120000}` for `MAX_DISTANCE_EXCEEDED`.
    #[must_use]
    pub fn details(&self) -> Option<&serde_json::Value> {
        self.details.as_ref()
    }

    /// The raw response body.
    #[must_use]
    pub fn body(&self) -> &str {
        &self.body
    }

    /// `true` when the token is missing, invalid or revoked.
    ///
    /// PedidosYa answers a malformed token with `400 INVALID_PARAMS`
    /// ("invalid token") rather than `401`, so this checks more than the status.
    #[must_use]
    pub fn is_unauthorized(&self) -> bool {
        matches!(self.status, 401 | 403)
            || self.code == Some(ErrorCode::InvalidToken)
            || self
                .message
                .as_deref()
                .is_some_and(|m| m.to_ascii_lowercase().contains("invalid token"))
    }

    /// `true` for `404`.
    #[must_use]
    pub fn is_not_found(&self) -> bool {
        self.status == 404
    }

    fn describe(&self) -> String {
        let mut out = format!("pedidosya returned {}", self.status);
        if let Some(code) = &self.code {
            let _ = write!(out, " ({code})");
        }
        match &self.message {
            Some(message) => {
                let _ = write!(out, ": {message}");
            }
            None if !self.body.is_empty() => {
                let _ = write!(out, ": {}", truncate(&self.body, 200));
            }
            None => {}
        }
        out
    }
}

/// A transport-level failure (no usable HTTP response).
///
/// Wraps the HTTP library's error without exposing it, so the HTTP library
/// can be upgraded without breaking this crate's API.
#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct HttpError(reqwest::Error);

impl HttpError {
    /// `true` if the request timed out.
    #[must_use]
    pub fn is_timeout(&self) -> bool {
        self.0.is_timeout()
    }

    /// `true` if the connection could not be established.
    #[must_use]
    pub fn is_connect(&self) -> bool {
        self.0.is_connect()
    }
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Self::Http(HttpError(err))
    }
}

string_enum! {
    /// Error codes documented by PedidosYa across all endpoints.
    pub enum ErrorCode {
        /// A waypoint address could not be geolocated.
        WaypointsNotFound => "WAYPOINTS_NOT_FOUND",
        /// A waypoint is outside PedidosYa's delivery zones.
        WaypointsOutOfZone => "WAYPOINTS_OUT_OF_ZONE",
        /// The route is longer than allowed (see [`ApiError::details`]).
        MaxDistanceExceeded => "MAX_DISTANCE_EXCEEDED",
        /// Too many waypoints.
        MaxWaypointsExceeded => "MAX_WAYPOINTS_EXCEEDED",
        /// The declared value of the items is too high.
        MaxValueExceeded => "MAX_VALUE_EXCEEDED",
        /// The items' volume is too large.
        MaxVolumeExceeded => "MAX_VOLUME_EXCEEDED",
        /// The items' weight is too high.
        MaxWeightExceeded => "MAX_WEIGHT_EXCEEDED",
        /// The request body was not valid JSON.
        JsonInvalid => "JSON_INVALID",
        /// Scheduled (pre-)orders are not enabled for this account.
        PreorderNotSupported => "PREORDER_NOT_SUPPORTED",
        /// The account is not able to create shippings right now.
        AccountNotOnline => "ACCOUNT_NOT_ONLINE",
        /// The requested delivery time is not valid.
        InvalidDeliveryTime => "INVALID_DELIVERY_TIME",
        /// The shipping or estimate is in a state that doesn't allow this action.
        InvalidStatus => "INVALID_STATUS",
        /// A path parameter (for example the shipping ID) is invalid.
        InvalidParam => "INVALID_PARAM",
        /// A query parameter is invalid.
        InvalidQueryParam => "INVALID_QUERY_PARAM",
        /// Too many labels requested at once (maximum 50).
        InvalidMaxLabelsQuantity => "INVALID_MAX_LABELS_QUANTITY",
        /// The shipping data needed for a label is invalid.
        InvalidShippingData => "INVALID_SHIPPING_DATA",
        /// The shipping doesn't exist or belongs to another account.
        ShippingDataNotFound => "SHIPPING_DATA_NOT_FOUND",
        /// The shipping doesn't exist.
        ShippingNotExist => "SHIPPING_NOT_EXIST",
        /// PedidosYa internal error.
        InternalServerError => "INTERNAL_SERVER_ERROR",
        /// The drop-off point is outside the delivery zone (not in the spec).
        DropOffOutOfZone => "DROP_OFF_OUT_OF_ZONE",
        /// There must be between 1 and 99 items (not in the spec).
        MaxItemsExceeded => "MAX_ITEMS_EXCEEDED",
        /// Invalid parameters, e.g. a malformed token (not in the spec).
        InvalidParams => "INVALID_PARAMS",
        /// Missing or rejected token (not in the spec).
        InvalidToken => "INVALID_TOKEN",
    }
}

fn truncate(s: &str, max: usize) -> &str {
    match s.char_indices().nth(max) {
        Some((idx, _)) => &s[..idx],
        None => s,
    }
}
