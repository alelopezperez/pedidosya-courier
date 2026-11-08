use reqwest::Method;

use crate::Client;
use crate::error::Result;
use crate::models::{CoverageRequest, CoverageResult, WorkingZone};

/// Coverage endpoints. Get it with [`Client::coverage`].
#[derive(Debug, Clone, Copy)]
pub struct Coverage<'a> {
    client: &'a Client,
}

impl<'a> Coverage<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Checks whether the points are inside PedidosYa's delivery zones.
    ///
    /// `POST /v3/estimates/coverage`
    ///
    /// ```no_run
    /// # async fn run(client: pedidosya_courier::Client) -> pedidosya_courier::Result<()> {
    /// use pedidosya_courier::models::{CoverageRequest, CoverageWaypoint, WaypointType};
    ///
    /// let request = CoverageRequest::new([
    ///     CoverageWaypoint::address(WaypointType::PickUp, "Av. Winston Churchill 93", "Santo Domingo"),
    ///     CoverageWaypoint::coordinates(WaypointType::DropOff, 18.4734, -69.8840),
    /// ]);
    /// match client.coverage().check(&request).await {
    ///     Ok(_) => println!("covered"),
    ///     Err(e) if e.status() == Some(400) => println!("not covered: {e}"),
    ///     Err(e) => return Err(e),
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// [`Error::Api`](crate::Error::Api) with codes such as
    /// `WAYPOINTS_OUT_OF_ZONE` or `MAX_DISTANCE_EXCEEDED` when the points are
    /// not covered; see [`Error`](crate::Error).
    pub async fn check(&self, request: &CoverageRequest) -> Result<CoverageResult> {
        self.client
            .send_json(Method::POST, &["v3", "estimates", "coverage"], request)
            .await
    }

    /// The zones (polygons) where PedidosYa delivers.
    ///
    /// `GET /v3/working-zones`
    ///
    /// # Errors
    ///
    /// See [`Error`](crate::Error).
    pub async fn working_zones(&self) -> Result<Vec<WorkingZone>> {
        self.client
            .json(self.client.request(Method::GET, &["v3", "working-zones"]))
            .await
    }
}
