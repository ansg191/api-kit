#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum IntoHttpError {
    #[error("endpoint was not supported by versions, but no unstable fallback path was defined")]
    NoUnstablePath,
    #[error("endpoint was removed")]
    EndpointRemoved,
    #[error("missing authorization")]
    MissingAuth,
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("url error: {0}")]
    Url(#[from] UrlError),
    #[error("query error: {0}")]
    Query(#[from] serde_urlencoded::ser::Error),
    #[error("header error: {0}")]
    Header(#[from] http::header::InvalidHeaderValue),
    #[error("http construction failed: {0}")]
    Http(#[from] http::Error),
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum FromHttpRequestError {
    #[error("deserialize error: {0}")]
    Deserialize(DeserializeError),
    #[error("method mismatch: expected {expected:?}, got {actual:?}")]
    MethodMismatch {
        expected: http::Method,
        actual: http::Method,
    },
}

impl<T> From<T> for FromHttpRequestError
where
    T: Into<DeserializeError>,
{
    fn from(err: T) -> Self {
        Self::Deserialize(err.into())
    }
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum FromHttpResponseError<E> {
    #[error("deserialize error: {0}")]
    Deserialize(DeserializeError),
    #[error("endpoint error: {0}")]
    EndpointError(E),
}

impl<T, E> From<T> for FromHttpResponseError<E>
where
    T: Into<DeserializeError>,
{
    fn from(err: T) -> Self {
        Self::Deserialize(err.into())
    }
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum DeserializeError {
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Uri(#[from] serde_urlencoded::de::Error),
    #[error(transparent)]
    Header(#[from] http::header::ToStrError),
    #[error("missing header: {0}")]
    MissingHeader(http::HeaderName),
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum UrlError {
    #[error("{0}")]
    Message(String),
    #[error("Top level serializer only supports structs")]
    TopLevel,
    #[error("Invalid endpoint")]
    InvalidEndpoint,
    #[error("Value not supported")]
    ValueNotSupported,
    #[error("Key not found: {0}")]
    KeyNotFound(&'static str),
    #[error("Unfilled field: {0}")]
    UnfilledField(String),
}

impl serde::ser::Error for UrlError {
    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        Self::Message(msg.to_string())
    }
}