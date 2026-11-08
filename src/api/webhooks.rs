use reqwest::Method;

use crate::Client;
use crate::error::Result;
use crate::models::{Environment, WebhookSubscription, WebhookTopic, WebhooksConfig};

/// Webhook configuration endpoints. Get it with [`Client::webhooks`].
///
/// To receive the callbacks themselves, see the [`webhook`](crate::webhook)
/// module.
#[derive(Debug, Clone, Copy)]
pub struct Webhooks<'a> {
    client: &'a Client,
}

impl<'a> Webhooks<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// All webhook subscriptions, for both environments.
    ///
    /// `GET /v3/webhooks-configuration`
    ///
    /// # Errors
    ///
    /// See [`Error`](crate::Error).
    pub async fn get(&self) -> Result<WebhooksConfig> {
        self.client
            .json(
                self.client
                    .request(Method::GET, &["v3", "webhooks-configuration"]),
            )
            .await
    }

    /// Webhook subscriptions of one environment.
    ///
    /// `GET /v3/webhooks-configuration?isTest=...`
    ///
    /// # Errors
    ///
    /// See [`Error`](crate::Error).
    pub async fn get_for(&self, environment: Environment) -> Result<WebhooksConfig> {
        let request = self
            .client
            .request(Method::GET, &["v3", "webhooks-configuration"])
            .query(&[("isTest", environment.is_test())]);
        self.client.json(request).await
    }

    /// Creates or updates webhook subscriptions, keyed by topic and
    /// environment. Subscriptions not included are kept: setting only the
    /// test environment leaves production untouched (verified against the
    /// live API). Returns the stored configuration.
    ///
    /// `PUT /v3/webhooks-configuration`
    ///
    /// ```no_run
    /// # async fn run(client: pedidosya_courier::Client) -> pedidosya_courier::Result<()> {
    /// use pedidosya_courier::models::{Environment, WebhookSubscription, WebhookUrl, WebhooksConfig};
    ///
    /// let config = WebhooksConfig::new([WebhookSubscription::shipping_status(Environment::Production)
    ///     .url(WebhookUrl::new("https://api.example.com/webhooks/pedidosya").authorization_key("s3cret"))]);
    /// client.webhooks().set(&config).await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// See [`Error`](crate::Error).
    pub async fn set(&self, config: &WebhooksConfig) -> Result<WebhooksConfig> {
        self.client
            .send_json(Method::PUT, &["v3", "webhooks-configuration"], config)
            .await
    }

    /// Deletes the subscription of one topic in one environment. Returns the
    /// deleted subscription.
    ///
    /// `DELETE /v3/webhooks-configuration?topic=...&isTest=...`
    ///
    /// # Errors
    ///
    /// [`Error::Api`](crate::Error::Api) with status `404` if there is no such
    /// subscription; see [`Error`](crate::Error).
    pub async fn delete(
        &self,
        topic: &WebhookTopic,
        environment: Environment,
    ) -> Result<WebhookSubscription> {
        let request = self
            .client
            .request(Method::DELETE, &["v3", "webhooks-configuration"])
            .query(&[("topic", topic.as_str())])
            .query(&[("isTest", environment.is_test())]);
        self.client.json(request).await
    }
}
