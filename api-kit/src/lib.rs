#![cfg_attr(docsrs, feature(doc_auto_cfg))]

use bytes::{BufMut, Bytes, BytesMut};
pub use http;

use crate::{
    auth::Authenticator,
    error::{FromHttpRequestError, FromHttpResponseError, IntoHttpError},
    metadata::{Version, VersionHistory},
};

pub mod auth;
pub mod error;
pub mod metadata;
mod url;

/// An API endpoint.
///
/// This is the base trait for all API endpoints.
/// It defines the type of the versioning,
/// the type of all errors, and a history of the endpoint per version.
pub trait Endpoint: Sized {
    /// The type representation of a version.
    type Version: Version;
    /// The error type returned by this endpoint.
    type Error: EndpointError;

    /// A history of this endpoint.
    const HISTORY: VersionHistory<'static, Self::Version>;
}

/// An incoming request.
///
/// This is a server-side trait
/// that handles converting incoming HTTP requests into the implementing type.
///
/// This trait must be paired with a corresponding [`OutgoingResponse`] implementation.
pub trait IncomingRequest: Endpoint {
    type OutgoingResponse: OutgoingResponse<IncomingRequest = Self>;

    /// Try to convert an incoming HTTP request into the implementing type.
    ///
    /// # Arguments
    ///
    /// * `req`: the incoming HTTP request object.
    /// * `path_args`: optional path arguments passed in.
    ///
    /// Returns: `Result<Self, FromHttpRequestError>`
    fn try_from_http_request<'a, B, I, P>(
        req: http::Request<B>,
        path_args: I,
    ) -> Result<Self, FromHttpRequestError>
    where
        B: AsRef<[u8]>,
        I: IntoIterator<Item = &'a P>,
        P: AsRef<str> + 'a;
}

pub trait OutgoingResponse: Sized {
    type IncomingRequest: IncomingRequest;

    fn try_into_http_response<B>(self) -> Result<http::Response<B>, IntoHttpError>
    where
        B: Default + BufMut;
}

pub trait OutgoingRequest: Endpoint + Clone {
    type IncomingResponse: IncomingResponse<OutgoingRequest = Self>;

    fn try_into_http_request<A>(
        self,
        base_url: &str,
        auth: A,
        considering_versions: &[Self::Version],
    ) -> Result<http::Request<BytesMut>, IntoHttpError>
    where
        A: Authenticator;
}

pub trait IncomingResponse: Sized {
    type OutgoingRequest: OutgoingRequest;

    fn try_from_http_response(
        res: http::Response<Bytes>,
    ) -> Result<Self, FromHttpResponseError<<Self::OutgoingRequest as Endpoint>::Error>>;
}

pub trait EndpointError: Sized + Send + 'static {
    fn try_into_http_response<B>(self) -> Result<http::Response<B>, IntoHttpError>
    where
        B: Default + BufMut;
    fn from_http_response<T: AsRef<[u8]>>(response: http::Response<T>) -> Self;
}
