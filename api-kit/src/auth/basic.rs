//! Basic authentication scheme.

use base64::{engine::general_purpose::STANDARD, Engine};
use bytes::BytesMut;
use http::{header::AUTHORIZATION, HeaderValue, Request};

use crate::{
    auth::{AuthScheme, Authenticator},
    error::IntoHttpError,
};

/// Basic authentication scheme.
///
/// This authenticator adds a [`Basic`] authentication header to the request.
///
/// [`Basic`]: https://datatracker.ietf.org/doc/html/rfc7617
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default)]
pub struct BasicAuth;

impl AuthScheme for BasicAuth {
    fn scheme(&self) -> &'static str {
        "basic"
    }
}

impl Authenticator for BasicAuth {
    type AuthData = BasicAuthData;

    fn authenticate(
        &self,
        req: &mut Request<BytesMut>,
        data: Self::AuthData,
    ) -> Result<(), IntoHttpError> {
        let auth = STANDARD.encode(format!("{}:{}", data.username, data.password));
        let header_val = HeaderValue::from_str(&format!("Basic {auth}"))?;

        let headers = req.headers_mut();
        headers.insert(AUTHORIZATION, header_val);

        Ok(())
    }
}

/// Basic authentication data.
///
/// This data (username and password) is required for basic authentication.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct BasicAuthData {
    pub username: String,
    pub password: String,
}

impl BasicAuthData {
    #[inline]
    #[must_use]
    pub const fn new(username: String, password: String) -> Self {
        Self { username, password }
    }
}
