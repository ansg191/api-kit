use std::fmt::Debug;
use bytes::BytesMut;
use http::Request;

use crate::error::IntoHttpError;

#[cfg(feature = "basic-auth")]
pub mod basic;
pub mod bearer;

pub trait AuthScheme: Debug {
    fn scheme(&self) -> &'static str;
}

pub trait Authenticator: AuthScheme {
    type AuthData;

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
