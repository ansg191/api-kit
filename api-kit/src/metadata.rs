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

#[derive(Debug, Clone)]
pub struct VersionHistory<'a, V: Version> {
    /// A list of unstable endpoints.
    ///
    /// For endpoint querying purposes, the last item is used.
    pub unstable_paths: &'a [Metadata<'a>],
    /// A list of metadata versions, mapped to the version they were introduced in.
    pub stable_paths: &'a [(V, Metadata<'a>)],
    /// The version the endpoint was deprecated in, if any.
    pub deprecated: Option<V>,
    /// The version the endpoint was removed in, if any.
    pub removed: Option<V>,
}

impl<'a, V: Version> VersionHistory<'a, V> {
    /// Creates a new [`VersionHistory`].
    pub const fn new(
        unstable_paths: &'a [Metadata<'a>],
        stable_paths: &'a [(V, Metadata<'a>)],
        deprecated: Option<V>,
        removed: Option<V>,
    ) -> Self {
        Self {
            unstable_paths,
            stable_paths,
            deprecated,
            removed,
        }
    }

    /// Constructs an endpoint URL based on the provided arguments.
    ///
    /// # Arguments
    ///
    /// * `versions` - A slice of `Version`s representing the accepted API versions.
    /// * `base_url` - The base URL of the endpoint.
    /// * `path_args` - The path arguments to filled into the URL placeholders.
    /// * `query_string` - The query arguments to be appended to the URL.
    ///
    /// # Returns
    ///
    /// A `Result` with the constructed endpoint URL as a `String` if successful,
    /// or an `IntoHttpError` if an error occurs.
    pub fn make_endpoint_url(
        &self,
        versions: &[V],
        base_url: &str,
        path_args: &impl Serialize,
        query_string: &impl Serialize,
    ) -> Result<Uri, IntoHttpError> {
        self.select_endpoint(versions)?
            .make_url(base_url, path_args, query_string)
    }

    /// Determines how a particular set of versions sees this endpoint.
    ///
    /// Only returns `Deprecated` or `Removed` if all versions denote it.
    ///
    /// In other words, if in any version it tells it supports the endpoint in a stable fashion,
    /// this returns `Stable`, even if some versions in this set denote deprecation or
    /// removal.
    ///
    /// If the resulting [`VersioningDecision`] is `Stable`, it also details if any version
    /// denoted deprecation or removal.
    #[must_use]
    pub fn versioning_decision_for(&self, versions: &[V]) -> VersioningDecision {
        let greater_or_equal_any = |version: &V| versions.iter().any(|v| v >= version);
        let greater_or_equal_all = |version: &V| versions.iter().all(|v| v >= version);

        // Check if all versions removed this endpoint.
        if self.removed.as_ref().is_some_and(greater_or_equal_all) {
            return VersioningDecision::Removed;
        }

        // Check if *any* version marks this endpoint as stable.
        if self.added_in().is_some_and(greater_or_equal_any) {
            let all_deprecated = self.deprecated.as_ref().is_some_and(greater_or_equal_all);

            return VersioningDecision::Stable {
                any_deprecated: all_deprecated
                    || self.deprecated.as_ref().is_some_and(greater_or_equal_any),
                all_deprecated,
                any_removed: self.removed.as_ref().is_some_and(greater_or_equal_any),
            };
        }

        VersioningDecision::Unstable
    }

    /// The metadata that should be used to query the endpoint given a series of versions.
    ///
    /// This picks the latest metadata that the version accepts.
    ///
    /// Note: this does not keep in mind endpoint removals, check with
    /// [`versioning_decision_for`](VersionHistory::versioning_decision_for) to see if this endpoint
    /// is still available.
    #[must_use]
    pub fn stable_endpoint_for(&self, versions: &[V]) -> Option<&Metadata> {
        // Go reverse to check the "latest" version first.
        for (ver, meta) in self.stable_paths.iter().rev() {
            // Check if any of the versions are equal or greater than the version the path needs.
            if versions.iter().any(|v| v >= ver) {
                return Some(meta);
            }
        }

        None
    }

    /// Returns the *first* version this endpoint was introduced in.
    ///
    /// If the endpoint is unstable, returns [`None`]
    #[must_use]
    pub fn added_in(&self) -> Option<&V> {
        self.stable_paths.first().map(|(version, _)| version)
    }

    /// Returns the version this endpoint was deprecated in, if any.
    #[must_use]
    pub const fn deprecated_in(&self) -> Option<&V> {
        self.deprecated.as_ref()
    }

    /// Returns the version this endpoint was removed in, if any.
    #[must_use]
    pub const fn removed_in(&self) -> Option<&V> {
        self.removed.as_ref()
    }

    /// Picks the last unstable metadata if it exists.
    #[must_use]
    pub const fn unstable(&self) -> Option<&Metadata> {
        self.unstable_paths.last()
    }

    /// Returns all metadata variants
    pub fn all_paths(&self) -> impl Iterator<Item = &Metadata> + '_ {
        self.unstable_paths()
            .chain(self.stable_paths().map(|(_, meta)| meta))
    }

    /// Returns all unstable path variants in canon form.
    pub fn unstable_paths(&self) -> impl Iterator<Item = &Metadata> {
        self.unstable_paths.iter()
    }

    /// Returns all stable path variants in canon form, with a corresponding version.
    pub fn stable_paths(&self) -> impl Iterator<Item = (&V, &Metadata)> {
        self.stable_paths
            .iter()
            .map(|(version, data)| (version, data))
    }

    pub fn select_endpoint(&self, versions: &[V]) -> Result<&Metadata, IntoHttpError> {
        match self.versioning_decision_for(versions) {
            VersioningDecision::Unstable => self.unstable().ok_or(IntoHttpError::NoUnstablePath),
            VersioningDecision::Stable { .. } => Ok(self
                .stable_endpoint_for(versions)
                .expect("stable_endpoint_for should return Some if VersioningDecision is Stable")),
            VersioningDecision::Removed => Err(IntoHttpError::EndpointRemoved),
        }
    }
}

/// The `history!` macro defines a version history for API paths and their corresponding metadata.
///
/// # Usage
///
/// The macro takes a version type, an optional list of unstable [`Metadata`]s, a optional list of
/// stable [`Metadata`]s with their corresponding versions, and optionally a list of deprecated and
/// removed versions.
///
/// For example:
///
/// The following is a version history for a hypothetical API. It uses `i32`s as to specify the
/// version.
/// The API consists of an unstable endpoint at `GET`
/// `/v1alpha1/endpoint` with Bearer authentication.
/// The API stabilizes at version `1`, with the endpoint at `GET` `/v1/endpoint` with Bearer auth.
/// Version `2` introduces Basic auth,
/// the endpoint is moved to `/v2/endpoint`, and the endpoint is deprecated.
/// Finally, version `3` removes the endpoint.
///
/// ```rust
/// use api_kit::{auth::bearer::BearerAuth, history, metadata::VersionHistory};
/// const HISTORY: VersionHistory<i32> = history! {
///     i32,
///     @unstable => {
///         method: GET,
///         auth: [BearerAuth],
///         path: "/v1alpha1/endpoint",
///     },
///     1 => {
///         method: GET,
///         auth: [BearerAuth],
///         path: "/v1/endpoint",
///     },
///     2 => {
///         method: GET,
///         auth: [BearerAuth],
///         path: "/v2/endpoint",
///     },
///     2 => deprecated,
///     3 => removed,
/// };
/// ```
///
/// expands to:
/// ```rust
/// const HISTORY: api_kit::metadata::VersionHistory<i32> = {
///         const UNSTABLE_PATHS: &[api_kit::metadata::Metadata] = &[
///             api_kit::metadata::Metadata {
///                 method: api_kit::http::Method::GET,
///                 auth: &[&api_kit::auth::bearer::BearerAuth],
///                 path: "/v1alpha1/endpoint",
///             },
///         ];
///         const STABLE_PATHS: &[(i32, api_kit::metadata::Metadata)] = &[
///             (
///                 1,
///                 api_kit::metadata::Metadata {
///                     method: api_kit::http::Method::GET,
///                     auth: &[&api_kit::auth::bearer::BearerAuth],
///                     path: "/v1/endpoint",
///                 },
///             ),
///             (
///                 2,
///                 api_kit::metadata::Metadata {
///                     method: api_kit::http::Method::GET,
///                     auth: &[
///                         &api_kit::auth::bearer::BearerAuth,
///                     ],
///                     path: "/v2/endpoint",
///                 },
///             ),
///         ];
///         api_kit::metadata::VersionHistory::new(
///             UNSTABLE_PATHS,
///             STABLE_PATHS,
///             Some(2),
///             Some(3),
///         )
///     };
/// ```
#[macro_export]
macro_rules! history {
    (
        $vty:ty,
        $( @unstable => $rhs:tt, )*
        $( $( $version:expr => $rhs2:tt, )+ )?
        $(,)?
    ) => {{
        const UNSTABLE_PATHS: &[$crate::metadata::Metadata] = &[
            $( $crate::history!( @metadata $rhs ) ),*
        ];
        $crate::history! {
            @inner
            $vty,
            UNSTABLE_PATHS,
            $( $( $rhs2 => $version ),+ )?
        }
    }};

    (
        @inner
        $vty:ty,
        $unstable_paths: ident,
        $(
            $( { $( $field:ident: $val:tt ),+ $(,)? } => $version:expr ),+
            $(, deprecated => $deprecated:expr )?
            $(, removed => $removed:expr )?
        )?
    ) => {
        const STABLE_PATHS: &[($vty, $crate::metadata::Metadata)] = &[
            $( $( ($version, $crate::history!( @metadata { $( $field: $val ),+ } )) ),+ )?
        ];
        $crate::metadata::VersionHistory::new(
            $unstable_paths,
            STABLE_PATHS,
            $crate::history!( @opt_version $( $( $deprecated )? )? ),
            $crate::history!( @opt_version $( $( $removed )? )? ),
        )
    };

    (
        @metadata
        {
            $( $field:ident: $val:tt ),+ $(,)?
        }
    ) => {
        $crate::metadata::Metadata {
            $( $field: $crate::history!(@field $field: $val) ),+
        }
    };

    ( @field method: $method:ident ) => { $crate::http::Method::$method };
    ( @field auth: [ $($scheme:expr),* ]) => { &[$( &$scheme ),*] };
    ( @field path: $path:expr ) => { $path };

    ( @opt_version ) => { None };
    ( @opt_version $version:expr ) => { Some($version) };
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum VersioningDecision {
    /// The unstable endpoint should be used.
    Unstable,
    /// The stable endpoint should be used.
    Stable {
        /// If any version is deprecated.
        any_deprecated: bool,
        /// If *all* versions denoted deprecation
        all_deprecated: bool,
        /// If any version is removed.
        any_removed: bool,
    },
    /// This endpoint was removed in all versions, it should not be used.
    Removed,
}

pub trait Version: Ord + 'static {}

impl<T: Ord + 'static> Version for T {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn versioning_decision_unstable() {
        let history = history! {
            i32,
            @unstable => {
                method: GET,
                auth: [],
                path: "/v1alpha1/endpoint",
            },
        };

        assert_eq!(
            history.versioning_decision_for(&[1]),
            VersioningDecision::Unstable
        );
    }

    #[test]
    fn versioning_decision_removed() {
        let history = history! {
            i32,
            @unstable => {
                method: GET,
                auth: [],
                path: "/v1alpha1/endpoint",
            },
            1 => {
                method: GET,
                auth: [],
                path: "/v1/endpoint",
            },
            2 => removed,
        };

        assert_eq!(
            history.versioning_decision_for(&[2]),
            VersioningDecision::Removed
        );
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn versioning_decision_stable() {
        let history = history! {
            i32,
            @unstable => {
                method: GET,
                auth: [],
                path: "/v1alpha1/endpoint",
            },
            1 => {
                method: GET,
                auth: [],
                path: "/v1/endpoint",
            },
        };

        assert_eq!(
            history.versioning_decision_for(&[1]),
            VersioningDecision::Stable {
                any_deprecated: false,
                all_deprecated: false,
                any_removed: false
            }
        );

        let history = history! {
            i32,
            1 => {
                method: GET,
                auth: [],
                path: "/v1/endpoint",
            },
            2 => {
                method: GET,
                auth: [],
                path: "/v2/endpoint",
            },
        };

        assert_eq!(
            history.versioning_decision_for(&[1, 2]),
            VersioningDecision::Stable {
                any_deprecated: false,
                all_deprecated: false,
                any_removed: false
            }
        );

        let history = history! {
            i32,
            1 => {
                method: GET,
                auth: [],
                path: "/v1/endpoint",
            },
            2 => {
                method: GET,
                auth: [],
                path: "/v2/endpoint",
            },
            2 => deprecated,
        };

        assert_eq!(
            history.versioning_decision_for(&[1, 2]),
            VersioningDecision::Stable {
                any_deprecated: true,
                all_deprecated: false,
                any_removed: false
            }
        );

        let history = history! {
            i32,
            1 => {
                method: GET,
                auth: [],
                path: "/v1/endpoint",
            },
            2 => {
                method: GET,
                auth: [],
                path: "/v2/endpoint",
            },
            2 => deprecated,
        };

        assert_eq!(
            history.versioning_decision_for(&[2]),
            VersioningDecision::Stable {
                any_deprecated: true,
                all_deprecated: true,
                any_removed: false
            }
        );

        let history = history! {
            i32,
            1 => {
                method: GET,
                auth: [],
                path: "/v1/endpoint",
            },
            2 => {
                method: GET,
                auth: [],
                path: "/v2/endpoint",
            },
            2 => deprecated,
            3 => removed,
        };

        assert_eq!(
            history.versioning_decision_for(&[1, 2, 3]),
            VersioningDecision::Stable {
                any_deprecated: true,
                all_deprecated: false,
                any_removed: true
            }
        );
    }
}
