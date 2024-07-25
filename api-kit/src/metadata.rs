use http::{HeaderName, HeaderValue, Uri};
use serde::Serialize;

use crate::{auth::AuthScheme, error::IntoHttpError, url::construct_url};

#[derive(Debug, Clone, Default)]
pub struct Metadata<'a> {
    pub method: http::Method,
    pub auth: &'a [&'a dyn AuthScheme],
    pub path: &'a str,
    pub headers: &'a [(HeaderName, HeaderValue)],
}

impl Metadata<'_> {
    pub fn make_url(
        &self,
        base_url: &str,
        path_args: &impl Serialize,
        query_string: &impl Serialize,
    ) -> Result<Uri, IntoHttpError> {
        let base_url = base_url.strip_suffix('/').unwrap_or(base_url);
        Ok(Uri::try_from(construct_url(
            base_url,
            self.path,
            path_args,
            query_string,
        )?)?)
    }

    pub fn contains_auth(&self, scheme: &impl AuthScheme) -> bool {
        let scheme_str = scheme.scheme();
        self.auth.iter().any(|auth| auth.scheme() == scheme_str)
    }
}
