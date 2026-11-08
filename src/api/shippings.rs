use bytes::Bytes;
use reqwest::Method;
use serde::Serialize;

use crate::Client;
use crate::error::{Error, Result};
use crate::models::{
    CallbackInvocation, CallbackInvocations, DeliveryPin, ListShippings, ProofOfDelivery, Shipping,
    ShippingId, ShippingPage, ShippingRequest, Tracking,
};

/// Maximum number of shippings per [`Shippings::labels`] call.
pub const MAX_LABELS: usize = 50;

/// Shipping order endpoints. Get it with [`Client::shippings`].
#[derive(Debug, Clone, Copy)]
pub struct Shippings<'a> {
    client: &'a Client,
}

impl<'a> Shippings<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Creates a shipping directly, without an estimate first.
    ///
    /// `POST /v3/shippings`
    ///
    /// ```no_run
    /// # async fn run(client: pedidosya_courier::Client) -> pedidosya_courier::Result<()> {
    /// use pedidosya_courier::models::{Item, ShippingRequest, Waypoint};
    ///
    /// let request = ShippingRequest::new(
    ///     "pedido-1042",
    ///     Waypoint::pick_up("Av. Winston Churchill 93", "Santo Domingo", "Doña Carmen", "+18095550000"),
    ///     Waypoint::drop_off("Calle El Conde 45", "Santo Domingo", "María", "+18095551234"),
    /// )
    /// .item(Item::new("Pica pollo familiar", 850.0));
    ///
    /// let shipping = client.shippings().create(&request).await?;
    /// println!("created {} ({:?})", shipping.shipping_id, shipping.status);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// [`Error::Api`] with codes such as `WAYPOINTS_NOT_FOUND`,
    /// `WAYPOINTS_OUT_OF_ZONE` or `ACCOUNT_NOT_ONLINE`; see [`Error`].
    pub async fn create(&self, request: &ShippingRequest) -> Result<Shipping> {
        self.client
            .send_json(Method::POST, &["v3", "shippings"], request)
            .await
    }

    /// Gets one shipping.
    ///
    /// `GET /v3/shippings/{shippingId}`
    ///
    /// # Errors
    ///
    /// [`Error::Api`] with status `404` if it doesn't exist; see [`Error`].
    pub async fn get(&self, id: &ShippingId) -> Result<Shipping> {
        self.client
            .json(
                self.client
                    .request(Method::GET, &["v3", "shippings", id.as_str()]),
            )
            .await
    }

    /// Lists shippings created between two dates, one page at a time.
    /// The range can span at most [`ListShippings::MAX_RANGE_DAYS`] days.
    ///
    /// `GET /v3/shippings`
    ///
    /// ```no_run
    /// # async fn run(client: pedidosya_courier::Client) -> pedidosya_courier::Result<()> {
    /// use chrono::NaiveDate;
    /// use pedidosya_courier::models::ListShippings;
    ///
    /// let mut query = ListShippings::new(
    ///     NaiveDate::from_ymd_opt(2026, 11, 1).unwrap(),
    ///     NaiveDate::from_ymd_opt(2026, 11, 7).unwrap(),
    /// );
    /// loop {
    ///     let page = client.shippings().list(&query).await?;
    ///     for shipping in &page.items {
    ///         println!("{} {:?}", shipping.shipping_id, shipping.status);
    ///     }
    ///     match page.next_query(&query) {
    ///         Some(next) => query = next,
    ///         None => break,
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// [`Error::InvalidInput`] if `from_date` is after `to_date` or the range
    /// is too long; otherwise see [`Error`].
    pub async fn list(&self, query: &ListShippings) -> Result<ShippingPage> {
        query.validate()?;
        self.client
            .json(
                self.client
                    .request(Method::GET, &["v3", "shippings"])
                    .query(query),
            )
            .await
    }

    /// Cancels a shipping.
    ///
    /// `POST /v3/shippings/{shippingId}/cancel`
    ///
    /// # Errors
    ///
    /// [`Error::Api`] with status `409` if it can no longer be cancelled;
    /// see [`Error`].
    pub async fn cancel(&self, id: &ShippingId, reason: &str) -> Result<Shipping> {
        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct Body<'a> {
            reason_text: &'a str,
        }
        self.client
            .send_json(
                Method::POST,
                &["v3", "shippings", id.as_str(), "cancel"],
                &Body {
                    reason_text: reason,
                },
            )
            .await
    }

    /// Live position of the rider. Available from `IN_PROGRESS` until the
    /// shipping is completed or cancelled; PedidosYa updates it about once a
    /// minute, so don't poll faster than that.
    ///
    /// `GET /v3/shippings/{shippingId}/tracking`
    ///
    /// # Errors
    ///
    /// See [`Error`].
    pub async fn tracking(&self, id: &ShippingId) -> Result<Tracking> {
        self.client
            .json(
                self.client
                    .request(Method::GET, &["v3", "shippings", id.as_str(), "tracking"]),
            )
            .await
    }

    /// Proof of delivery (signature or photo) of a completed shipping.
    ///
    /// `GET /v3/shippings/{shippingId}/proofOfDelivery`
    ///
    /// # Errors
    ///
    /// See [`Error`].
    pub async fn proof_of_delivery(&self, id: &ShippingId) -> Result<ProofOfDelivery> {
        self.client
            .json(self.client.request(
                Method::GET,
                &["v3", "shippings", id.as_str(), "proofOfDelivery"],
            ))
            .await
    }

    /// PIN the customer gives the rider to confirm the delivery.
    ///
    /// `GET /v3/shippings/{shippingId}/proofOfDelivery/pin`
    ///
    /// # Errors
    ///
    /// See [`Error`].
    pub async fn delivery_pin(&self, id: &ShippingId) -> Result<DeliveryPin> {
        self.client
            .json(self.client.request(
                Method::GET,
                &["v3", "shippings", id.as_str(), "proofOfDelivery", "pin"],
            ))
            .await
    }

    /// Changes the delivery instructions of a shipping that hasn't been
    /// delivered yet. Returns the instructions now stored.
    ///
    /// `PUT /v3/shippings/{shippingId}/deliveryInstructions`
    ///
    /// # Errors
    ///
    /// [`Error::Api`] with code `INVALID_STATUS` if the shipping is already
    /// completed or cancelled; see [`Error`].
    pub async fn update_delivery_instructions(
        &self,
        id: &ShippingId,
        instructions: &str,
    ) -> Result<String> {
        #[derive(Serialize, serde::Deserialize)]
        struct Instructions<T> {
            instructions: T,
        }
        let updated: Instructions<Option<String>> = self
            .client
            .send_json(
                Method::PUT,
                &["v3", "shippings", id.as_str(), "deliveryInstructions"],
                &Instructions { instructions },
            )
            .await?;
        Ok(updated
            .instructions
            .unwrap_or_else(|| instructions.to_owned()))
    }

    /// Callbacks PedidosYa sent (or tried to send) for a shipping, with the
    /// HTTP status your endpoint answered. Useful to debug webhooks.
    ///
    /// `GET /v3/shippings/{shippingId}/callbacks`
    ///
    /// # Errors
    ///
    /// See [`Error`].
    pub async fn callbacks(&self, id: &ShippingId) -> Result<Vec<CallbackInvocation>> {
        let response: CallbackInvocations = self
            .client
            .json(
                self.client
                    .request(Method::GET, &["v3", "shippings", id.as_str(), "callbacks"]),
            )
            .await?;
        Ok(response.callbacks)
    }

    /// Printable labels for up to [`MAX_LABELS`] confirmed shippings, as a
    /// single PDF file.
    ///
    /// `GET /v3/shippings/labels`
    ///
    /// ```no_run
    /// # async fn run(client: pedidosya_courier::Client) -> pedidosya_courier::Result<()> {
    /// use pedidosya_courier::models::ShippingId;
    ///
    /// let pdf = client.shippings().labels(&[ShippingId::new("123"), ShippingId::new("456")]).await?;
    /// std::fs::write("labels.pdf", &pdf).expect("write labels");
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// [`Error::InvalidInput`] if `ids` is empty or has more than
    /// [`MAX_LABELS`] entries; otherwise see [`Error`].
    pub async fn labels(&self, ids: &[ShippingId]) -> Result<Bytes> {
        if ids.is_empty() {
            return Err(Error::InvalidInput(
                "at least one shipping ID is required".into(),
            ));
        }
        if ids.len() > MAX_LABELS {
            return Err(Error::InvalidInput(format!(
                "at most {MAX_LABELS} labels per request, got {}",
                ids.len()
            )));
        }
        let values = ids
            .iter()
            .map(ShippingId::as_str)
            .collect::<Vec<_>>()
            .join(",");
        let request = self
            .client
            .request(Method::GET, &["v3", "shippings", "labels"])
            .query(&[("values", values)]);
        Ok(self.client.execute(request).await?.bytes().await?)
    }
}
