use std::fmt;
use std::sync::Arc;
use std::time::Duration;

use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue};
use reqwest::{Method, RequestBuilder, Url};
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::api::{Coverage, Estimates, Shippings, Webhooks};
use crate::error::{ApiError, Error, Result};
use crate::models::Schedule;

/// Production base URL of the PedidosYa Courier API.
pub const DEFAULT_BASE_URL: &str = "https://courier-api.pedidosya.com";

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);
const DEFAULT_CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
const DEFAULT_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

/// Client for the PedidosYa Courier API.
///
/// Cheap to clone: clones share one connection pool. Create it once and pass
/// it around (or store it in your application state).
///
/// Each PedidosYa account (branch) has its own token, so use one client per
/// account.
///
/// ```no_run
/// # async fn run() -> pedidosya_courier::Result<()> {
/// use pedidosya_courier::Client;
///
/// let client = Client::new(std::env::var("PEDIDOSYA_TOKEN").unwrap_or_default())?;
/// for day in client.schedules().await? {
///     println!("{}: {} - {} UTC", day.day, day.from, day.to);
/// }
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
pub struct Client {
    inner: Arc<Inner>,
}

struct Inner {
    http: reqwest::Client,
    base_url: Url,
}

impl Client {
    /// A client for the production API with default settings.
    ///
    /// The token is generated in the PedidosYa Envíos dashboard
    /// (Integraciones → Generar token) and doesn't expire.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Config`] if the token is empty or not a valid header
    /// value, or if the HTTP client can't be created.
    pub fn new(token: impl Into<String>) -> Result<Self> {
        Self::builder(token).build()
    }

    /// Starts configuring a client.
    pub fn builder(token: impl Into<String>) -> ClientBuilder {
        ClientBuilder {
            token: token.into(),
            base_url: DEFAULT_BASE_URL.to_owned(),
            timeout: DEFAULT_TIMEOUT,
            connect_timeout: DEFAULT_CONNECT_TIMEOUT,
            user_agent: DEFAULT_USER_AGENT.to_owned(),
        }
    }

    /// Shipping orders: create, list, get, cancel, track, labels, ...
    #[must_use]
    pub fn shippings(&self) -> Shippings<'_> {
        Shippings::new(self)
    }

    /// Price quotes, which can then be confirmed into shippings.
    #[must_use]
    pub fn estimates(&self) -> Estimates<'_> {
        Estimates::new(self)
    }

    /// Delivery zones and coverage checks.
    #[must_use]
    pub fn coverage(&self) -> Coverage<'_> {
        Coverage::new(self)
    }

    /// Webhook (callback) configuration.
    #[must_use]
    pub fn webhooks(&self) -> Webhooks<'_> {
        Webhooks::new(self)
    }

    /// Days and hours (UTC) in which PedidosYa's fleet delivers.
    ///
    /// `GET /v3/schedules`
    ///
    /// # Errors
    ///
    /// See [`Error`].
    pub async fn schedules(&self) -> Result<Vec<Schedule>> {
        self.json(self.request(Method::GET, &["v3", "schedules"]))
            .await
    }

    /// The base URL requests are sent to.
    #[must_use]
    pub fn base_url(&self) -> &str {
        self.inner.base_url.as_str()
    }

    // ----- internal helpers used by the API handlers -----

    /// Builds a request to `base_url` + path segments. Segments are
    /// percent-encoded, so IDs can't change the path.
    pub(crate) fn request(&self, method: Method, segments: &[&str]) -> RequestBuilder {
        let mut url = self.inner.base_url.clone();
        {
            // `build` guarantees the base URL can have a path.
            let mut path = url.path_segments_mut().expect("base URL can be a base");
            path.pop_if_empty().extend(segments);
        }
        self.inner.http.request(method, url)
    }

    /// Sends the request and returns the successful response, or an
    /// [`ApiError`] for 4xx/5xx statuses.
    pub(crate) async fn execute(&self, request: RequestBuilder) -> Result<reqwest::Response> {
        let response = request.send().await?;
        let status = response.status();
        if status.is_success() {
            return Ok(response);
        }
        let body = response.text().await.unwrap_or_default();
        Err(ApiError::from_response(status.as_u16(), body).into())
    }

    /// Sends the request and decodes a JSON body.
    pub(crate) async fn json<T: DeserializeOwned>(&self, request: RequestBuilder) -> Result<T> {
        let bytes = self.execute(request).await?.bytes().await?;
        serde_json::from_slice(&bytes).map_err(|source| Error::Decode {
            source,
            body: String::from_utf8_lossy(&bytes).into_owned(),
        })
    }

    /// Sends a JSON body and decodes a JSON response.
    pub(crate) async fn send_json<B, T>(
        &self,
        method: Method,
        segments: &[&str],
        body: &B,
    ) -> Result<T>
    where
        B: Serialize + ?Sized,
        T: DeserializeOwned,
    {
        self.json(self.request(method, segments).json(body)).await
    }
}

impl fmt::Debug for Client {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Client")
            .field("base_url", &self.inner.base_url.as_str())
            .field("token", &"<redacted>")
            .finish_non_exhaustive()
    }
}

/// Configures a [`Client`]. Created with [`Client::builder`].
///
/// ```no_run
/// # fn run() -> pedidosya_courier::Result<()> {
/// use std::time::Duration;
/// use pedidosya_courier::Client;
///
/// let client = Client::builder("my-token")
///     .timeout(Duration::from_secs(10))
///     .user_agent("my-app/1.0")
///     .build()?;
/// # Ok(())
/// # }
/// ```
#[must_use = "call `.build()` to create the client"]
pub struct ClientBuilder {
    token: String,
    base_url: String,
    timeout: Duration,
    connect_timeout: Duration,
    user_agent: String,
}

impl ClientBuilder {
    /// Overrides the API base URL (e.g. a mock server in tests).
    /// Defaults to [`DEFAULT_BASE_URL`].
    pub fn base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }

    /// Total time allowed for each request. Defaults to 30 seconds.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Time allowed to open a connection. Defaults to 10 seconds.
    pub fn connect_timeout(mut self, timeout: Duration) -> Self {
        self.connect_timeout = timeout;
        self
    }

    /// `User-Agent` header. Defaults to `pedidosya-courier/<version>`.
    pub fn user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = user_agent.into();
        self
    }

    /// Creates the client.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Config`] if the token is empty or contains invalid
    /// characters, the base URL is invalid, or the HTTP client can't be built.
    pub fn build(self) -> Result<Client> {
        let token = self.token.trim();
        if token.is_empty() {
            return Err(Error::Config("token is empty".into()));
        }
        // PedidosYa expects the raw token, without a "Bearer" prefix.
        let mut auth = HeaderValue::from_str(token).map_err(|_| {
            Error::Config("token contains characters not allowed in a header".into())
        })?;
        auth.set_sensitive(true);

        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, auth);

        let base_url = Url::parse(&self.base_url)
            .map_err(|e| Error::Config(format!("invalid base URL {:?}: {e}", self.base_url)))?;
        if base_url.cannot_be_a_base() || !matches!(base_url.scheme(), "http" | "https") {
            return Err(Error::Config(format!(
                "base URL must be http(s): {:?}",
                self.base_url
            )));
        }

        let http = reqwest::Client::builder()
            .default_headers(headers)
            .user_agent(self.user_agent)
            .timeout(self.timeout)
            .connect_timeout(self.connect_timeout)
            .build()
            .map_err(|e| Error::Config(format!("failed to build HTTP client: {e}")))?;

        Ok(Client {
            inner: Arc::new(Inner { http, base_url }),
        })
    }
}

impl fmt::Debug for ClientBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ClientBuilder")
            .field("token", &"<redacted>")
            .field("base_url", &self.base_url)
            .field("timeout", &self.timeout)
            .field("connect_timeout", &self.connect_timeout)
            .field("user_agent", &self.user_agent)
            .finish()
    }
}
