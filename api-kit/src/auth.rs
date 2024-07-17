use std::fmt::Debug;

use bytes::BytesMut;
use http::Request;

use crate::error::IntoHttpError;

#[cfg(feature = "basic-auth")]
pub mod basic;
pub mod bearer;

/// Authentication schemes
pub trait AuthScheme: Debug {
    /// Returns the name of the authentication scheme.
    ///
    /// This must return a unique identifier for the scheme.
    fn scheme(&self) -> &'static str;
}

/// Authenticators
///
/// Authenticators are responsible for adding authentication to a request.
/// Authenticators must be a [`AuthScheme`] with a unique identifier.
///
/// The most basic implementation of an authenticator is `()`, which does nothing.
pub trait Authenticator: AuthScheme {
    /// The type of extra data required for authentication.
    ///
    /// For example, a bearer token authenticator would require a token,
    /// while a basic authenticator would require a username and password.
    type AuthData;

    /// Authenticate the request.
    ///
    /// This method must add the necessary headers to the request to authenticate it.
    /// This method is called after the request is built, headers are added,
    /// and the body is serialized.
    ///
    /// # Arguments
    ///
    /// * `req`: the [`Request`] to be authenticated.
    /// * `data`: authentication data required for the authenticator.
    ///
    /// Returns: `Result<(), IntoHttpError>`
    fn authenticate(
        &self,
        req: &mut Request<BytesMut>,
        data: Self::AuthData,
    ) -> Result<(), IntoHttpError>;
}

impl AuthScheme for () {
    fn scheme(&self) -> &'static str {
        ""
    }
}

impl Authenticator for () {
    type AuthData = ();

    fn authenticate(
        &self,
        _req: &mut Request<BytesMut>,
        _data: Self::AuthData,
    ) -> Result<(), IntoHttpError> {
        Ok(())
    }
}
