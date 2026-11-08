use reqwest::Method;

use crate::Client;
use crate::error::Result;
use crate::models::{
    ConfirmEstimateBody, DeliveryOfferId, Estimate, EstimateId, Shipping, ShippingRequest,
};

/// Estimate (quote) endpoints. Get it with [`Client::estimates`].
///
/// The two-step flow: [`create`](Self::create) an estimate to see prices and
/// delivery offers, then [`confirm`](Self::confirm) one offer to turn it into
/// a shipping.
#[derive(Debug, Clone, Copy)]
pub struct Estimates<'a> {
    client: &'a Client,
}

impl<'a> Estimates<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Gets prices and delivery offers for a shipping, without creating it.
    ///
    /// `POST /v3/shippings/estimates`
    ///
    /// ```no_run
    /// # async fn run(client: pedidosya_courier::Client, request: pedidosya_courier::models::ShippingRequest) -> pedidosya_courier::Result<()> {
    /// let estimate = client.estimates().create(&request).await?;
    /// for offer in &estimate.delivery_offers {
    ///     let total = offer.pricing.as_ref().and_then(|p| p.total);
    ///     println!("{:?}: {:?}", offer.delivery_mode, total);
    /// }
    ///
    /// // Confirm the first offer.
    /// let offer = estimate.delivery_offers.first().map(|o| &o.delivery_offer_id);
    /// let shipping = client.estimates().confirm(&estimate.estimate_id, offer).await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// See [`Error`](crate::Error).
    pub async fn create(&self, request: &ShippingRequest) -> Result<Estimate> {
        self.client
            .send_json(Method::POST, &["v3", "shippings", "estimates"], request)
            .await
    }

    /// Confirms an estimate, creating the shipping. With `offer = None`,
    /// PedidosYa picks the first available offer.
    ///
    /// `POST /v3/shippings/estimates/{estimateId}/confirm`
    ///
    /// # Errors
    ///
    /// [`Error::Api`](crate::Error::Api) with status `409` if the estimate
    /// can't be confirmed anymore (for example it expired); see
    /// [`Error`](crate::Error).
    pub async fn confirm(
        &self,
        id: &EstimateId,
        offer: Option<&DeliveryOfferId>,
    ) -> Result<Shipping> {
        self.client
            .send_json(
                Method::POST,
                &["v3", "shippings", "estimates", id.as_str(), "confirm"],
                &ConfirmEstimateBody {
                    delivery_offer_id: offer,
                },
            )
            .await
    }
}
