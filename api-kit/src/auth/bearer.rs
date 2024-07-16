use bytes::BytesMut;
use http::{header::AUTHORIZATION, HeaderValue, Request};

use crate::{
    auth::{AuthScheme, Authenticator},
    error::IntoHttpError,
};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default)]
pub struct BearerAuth;

impl AuthScheme for BearerAuth {
    fn scheme(&self) -> &'static str {
        "bearer"
    }
}

impl Authenticator for BearerAuth {
    type AuthData = String;
    fn authenticate(
        &self,
        req: &mut Request<BytesMut>,
        token: Self::AuthData,
    ) -> Result<(), IntoHttpError> {
        let headers = req.headers_mut();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", token))?,
        );
        Ok(())
    }
}
