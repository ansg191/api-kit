use displaydoc::Display;
use thiserror::Error;

#[derive(Debug, Display, Error)]
#[non_exhaustive]
pub enum IntoHttpError {
    /// Endpoint wasn't supported by versions, but no unstable fallback path was defined.
    NoUnstablePath,
    /// Endpoint was removed.
    EndpointRemoved,
    /// Missing authorization.
    MissingAuth,
    /// JSON serialization error: {0}
    Json(#[from] serde_json::Error),
    /// URL serialization error: {0}
    Url(#[from] UrlError),
    /// Query serialization error: {0}
    Query(#[from] serde_urlencoded::ser::Error),
    /// Invalid header value
    Header(#[from] http::header::InvalidHeaderValue),
    /// HTTP construction failed: {0}
    Http(#[from] http::Error),
}

#[derive(Debug, Display, Error)]
#[non_exhaustive]
pub enum FromHttpRequestError {
    /// Deserialization error: {0}
    Deserialize(DeserializeError),
    /// Method mismatch.
    MethodMismatch {
        /// Expected method.
        expected: http::Method,
        /// Actual received method.
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

#[derive(Debug, Error)]
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

#[derive(Debug, Display, Error)]
#[non_exhaustive]
pub enum DeserializeError {
    /// Error parsing JSON
    Json(#[from] serde_json::Error),
    /// Error parsing query string: {0}
    Uri(#[from] serde_urlencoded::de::Error),
    /// Error converting header to string
    Header(#[from] http::header::ToStrError),
    /// Missing header: {0}
    MissingHeader(http::HeaderName),
}

#[derive(Debug, Display, PartialEq, Eq, Error)]
pub enum UrlError {
    /// Generic error message: {0}
    Message(String),
    /// Top level serializer only supports structs
    TopLevel,
    /// Invalid endpoint
    InvalidEndpoint,
    /// Value not supported
    ValueNotSupported,
    /// Key not found: {0}
    KeyNotFound(&'static str),
    /// Unfilled field: {0}
    UnfilledField(String),
}

impl serde::ser::Error for UrlError {
    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        Self::Message(msg.to_string())
    }
}
