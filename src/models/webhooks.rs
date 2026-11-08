use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::common::{Environment, ShippingId};
use crate::macros::string_enum;

string_enum! {
    /// Topic of a webhook subscription. PedidosYa only offers shipping status
    /// changes for now.
    pub enum WebhookTopic {
        /// Shipping status changes.
        ShippingStatus => "SHIPPING_STATUS",
    }
}

string_enum! {
    /// How notifications are delivered. Only webhooks exist for now.
    pub enum NotificationType {
        /// HTTP POST to your URL.
        Webhook => "WEBHOOK",
    }
}

/// All webhook subscriptions of the account.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct WebhooksConfig {
    /// Subscriptions (one per topic and environment).
    #[serde(rename = "webhooksConfiguration", default)]
    pub subscriptions: Vec<WebhookSubscription>,
}

impl WebhooksConfig {
    /// A configuration with the given subscriptions.
    pub fn new(subscriptions: impl IntoIterator<Item = WebhookSubscription>) -> Self {
        Self {
            subscriptions: subscriptions.into_iter().collect(),
        }
    }
}

/// Where PedidosYa sends notifications for one topic and environment.
///
/// ```
/// use pedidosya_courier::models::{Environment, WebhookSubscription, WebhookUrl};
///
/// let subscription = WebhookSubscription::shipping_status(Environment::Production)
///     .url(WebhookUrl::new("https://api.example.com/webhooks/pedidosya").authorization_key("s3cret"));
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct WebhookSubscription {
    /// `true` for the test environment.
    #[serde(default)]
    pub is_test: bool,
    /// Delivery method.
    pub notification_type: NotificationType,
    /// What to be notified about.
    pub topic: WebhookTopic,
    /// Up to 3 URLs.
    #[serde(default)]
    pub urls: Vec<WebhookUrl>,
}

impl WebhookSubscription {
    /// Shipping status notifications for an environment. Add URLs with
    /// [`url`](Self::url).
    #[must_use]
    pub fn shipping_status(environment: Environment) -> Self {
        Self {
            is_test: environment.is_test(),
            notification_type: NotificationType::Webhook,
            topic: WebhookTopic::ShippingStatus,
            urls: Vec::new(),
        }
    }

    /// Adds a destination URL.
    #[must_use]
    pub fn url(mut self, url: WebhookUrl) -> Self {
        self.urls.push(url);
        self
    }
}

/// A webhook destination.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct WebhookUrl {
    /// Your endpoint. It can't contain `<` or `>`.
    pub url: String,
    /// Secret PedidosYa sends back in the `Authorization` header of every
    /// callback. Check it with
    /// [`webhook::verify_authorization`](crate::webhook::verify_authorization).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorization_key: Option<String>,
}

impl WebhookUrl {
    /// A destination without an authorization key.
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            authorization_key: None,
        }
    }

    /// Sets the secret sent in the `Authorization` header of each callback.
    #[must_use]
    pub fn authorization_key(mut self, key: impl Into<String>) -> Self {
        self.authorization_key = Some(key.into());
        self
    }
}

impl std::fmt::Debug for WebhookUrl {
    // Never print the secret.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WebhookUrl")
            .field("url", &self.url)
            .field(
                "authorization_key",
                &self.authorization_key.as_ref().map(|_| "<redacted>"),
            )
            .finish()
    }
}

string_enum! {
    /// Event that triggered a callback.
    pub enum ShippingEvent {
        /// Shipping confirmed.
        Confirmed => "CONFIRMED",
        /// Rider assigned.
        InProgress => "IN_PROGRESS",
        /// Rider near pick-up.
        NearPickup => "NEAR_PICKUP",
        /// Items picked up.
        PickedUp => "PICKED_UP",
        /// Rider near drop-off.
        NearDropoff => "NEAR_DROPOFF",
        /// Delivered.
        Completed => "COMPLETED",
        /// Cancelled.
        Cancelled => "CANCELLED",
        /// Expired.
        Expired => "EXPIRED",
        /// A delivery PIN was created (see the security code in the callback).
        SecurityCodeCreated => "SECURITY_CODE_CREATED",
    }
}

/// A callback PedidosYa sent (or tried to send) for a shipping.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct CallbackInvocation {
    /// Delivery method.
    pub notification_type: Option<NotificationType>,
    /// URL that was called.
    // The spec says `callBackUrl`; the live API sends `callbackUrl`, with the
    // URL wrapped in brackets (`"[https://...]"`). Accept both, strip brackets.
    #[serde(alias = "callBackUrl", default, deserialize_with = "unbracket")]
    pub callback_url: Option<String>,
    /// HTTP status your endpoint answered with (`200` means delivered).
    /// `-1` means PedidosYa could not reach your endpoint at all.
    pub response_code: Option<i32>,
    /// Shipping ID.
    pub shipping_id: Option<ShippingId>,
    /// Topic.
    pub topic: Option<WebhookTopic>,
    /// Event that triggered the callback.
    pub event_code: Option<ShippingEvent>,
    /// When the callback was sent (UTC).
    pub created_at: Option<DateTime<Utc>>,
}

impl CallbackInvocation {
    /// `true` if your endpoint answered with a 2xx status.
    #[must_use]
    pub fn delivered(&self) -> bool {
        matches!(self.response_code, Some(200..=299))
    }
}

fn unbracket<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<String>, D::Error> {
    let value = Option::<String>::deserialize(deserializer)?;
    Ok(value.map(
        |url| match url.strip_prefix('[').and_then(|u| u.strip_suffix(']')) {
            Some(inner) => inner.to_owned(),
            None => url,
        },
    ))
}

#[derive(Debug, Deserialize)]
pub(crate) struct CallbackInvocations {
    #[serde(default)]
    pub(crate) callbacks: Vec<CallbackInvocation>,
}
