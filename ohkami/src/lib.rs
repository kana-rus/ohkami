#![doc(html_root_url = "https://docs.rs/ohkami")]

/* Execute static tests for sample codes in README */
#![cfg_attr(feature="DEBUG", doc = include_str!("../../README.md"))]

//! <div align="center">
//!     <h1>Ohkami</h1>
//!     Ohkami <em>- [狼] wolf in Japanese -</em> is intuitive and declarative web framework.
//! </div>
//! 
//! <br>
//! 
//! - *macro-less and type-safe* APIs for intuitive and declarative code
//! - *multi runtime* support：`tokio`, `async-std`, `worker` (Cloudflare Workers)


#![cfg_attr(feature="nightly", feature(
    min_specialization,
    try_trait_v2,
))]


#[cfg(any(
    all(feature="rt_tokio",     feature="rt_async-std"),
    all(feature="rt_async-std", feature="rt_worker"),
    all(feature="rt_worker",    feature="rt_tokio"),
))] compile_error!("
    Can't activate multiple `rt_*` features!
");


#[allow(unused)]
mod __rt__ {
    #[cfg(all(feature="rt_tokio", feature="DEBUG"))]
    pub(crate) use tokio::test;
    #[allow(unused)]
    #[cfg(all(feature="rt_async-std", feature="DEBUG"))]
    pub(crate) use async_std::test;

    #[cfg(feature="rt_tokio")]
    pub(crate) use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};
    #[cfg(feature="rt_async-std")]
    pub(crate) use async_std::net::{TcpListener, TcpStream, ToSocketAddrs};

    #[cfg(feature="rt_tokio")]
    pub(crate) use tokio::task;
    #[cfg(feature="rt_async-std")]
    pub(crate) use async_std::task;

    #[cfg(feature="rt_tokio")]
    pub(crate) use tokio::time::sleep;
    #[cfg(feature="rt_async-std")]
    pub(crate) use async_std::task::sleep;

    #[cfg(feature="rt_tokio")]
    pub(crate) use tokio::io::AsyncReadExt as AsyncReader;
    #[cfg(feature="rt_async-std")]
    pub(crate) use async_std::io::ReadExt as AsyncReader;

    #[cfg(feature="rt_tokio")]
    pub(crate) use tokio::io::AsyncWriteExt as AsyncWriter;
    #[cfg(feature="rt_async-std")]
    pub(crate) use async_std::io::WriteExt as AsyncWriter;

    #[cfg(feature="rt_tokio")]
    pub(crate) use tokio::sync::watch::{Sender, Receiver};
    #[cfg(feature="rt_async-std")]
    pub(crate) use async_std::channel::{Sender, Receiver};
    #[cfg(any(feature="rt_tokio",feature="rt_async-std"))]
    pub(crate) fn channel() -> (Sender<()>, Receiver<()>) {
        #[cfg(feature="rt_tokio")] {
            tokio::sync::watch::channel(())
        }
        #[cfg(feature="rt_async-std")] {
            async_std::channel::bounded(1)
        }
    }
}


mod request;
pub use request::{Request, Method, FromRequest, FromParam, Memory};
pub use ::ohkami_macros::FromRequest;

mod response;
pub use response::{Response, Status, IntoResponse};

mod fangs;
pub use fangs::{Fang, FangProc};

mod session;
#[cfg(any(feature="rt_tokio",feature="rt_async-std"))]
use session::Session;

mod ohkami;
#[cfg(any(feature="rt_tokio",feature="rt_async-std",feature="rt_worker"))]
pub use ohkami::{Ohkami, Route};

pub mod builtin;

pub mod typed;

#[cfg(feature="testing")]
#[cfg(any(feature="rt_tokio",feature="rt_async-std",feature="rt_worker"))]
pub mod testing;

pub mod utils {
    #[doc(hidden)]
    #[macro_export]
    macro_rules! warning {
        ( $( $t:tt )* ) => {{
            eprintln!( $( $t )* );

            #[cfg(feature="rt_worker")]
            worker::console_log!( $( $t )* );
        }};
    }

    pub use crate::fangs::util::FangAction;

    #[cfg(feature="sse")]
    pub use ohkami_lib::stream;
    #[cfg(feature="sse")]
    pub use ohkami_lib::stream::{Stream, StreamExt};

    #[cfg(not(feature="rt_worker"))]
    /// ```
    /// # let _ =
    /// {
    ///     std::time::SystemTime::now()
    ///         .duration_since(std::time::UNIX_EPOCH)
    ///         .unwrap()
    ///         .as_secs()
    /// }
    /// # ;
    /// ```
    #[inline] pub fn unix_timestamp() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }
    #[cfg(feature="rt_worker")]
    /// ```ignore
    /// {
    ///     JS's `Date.now() / 1000` as Rust's u64
    /// }
    /// ```
    #[inline] pub fn unix_timestamp() -> u64 {
        (worker::js_sys::Date::now() / 1000.) as _
    }

    pub struct ErrorMessage(pub String);
    const _: () = {
        impl std::fmt::Debug for ErrorMessage {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(&self.0)
            }
        }
        impl std::fmt::Display for ErrorMessage {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(&self.0)
            }
        }
        impl std::error::Error for ErrorMessage {}
        impl super::IntoResponse for ErrorMessage {
            fn into_response(self) -> crate::Response {
                crate::Response::InternalServerError().with_text(self.0)
            }
        }
    };
}

#[cfg(feature="rt_worker")]
pub use ::ohkami_macros::{worker, bindings};

pub mod header {#![allow(non_snake_case)]
    pub(crate) mod private {
        use std::borrow::Cow;

        pub struct Append(pub(crate) Cow<'static, str>);

        #[derive(Debug, PartialEq)]
        pub enum SameSitePolicy {
            Strict,
            Lax,
            None,
        }
        impl SameSitePolicy {
            const fn as_str(&self) -> &'static str {
                match self {
                    Self::Strict => "Strict",
                    Self::Lax    => "Lax",
                    Self::None   => "None",
                }
            }
            const fn from_bytes(bytes: &[u8]) -> Option<Self> {
                match bytes {
                    b"Strict" => Some(Self::Strict),
                    b"Lax"    => Some(Self::Lax),
                    b"None"   => Some(Self::None),
                    _ => None
                }
            }
        }

        #[derive(Debug, PartialEq)]
        pub struct SetCookie<'c> {
            pub(crate) Cookie:   (&'c str, Cow<'c, str>),
            pub(crate) Expires:  Option<Cow<'c, str>>,
            pub(crate) MaxAge:   Option<u64>,
            pub(crate) Domain:   Option<Cow<'c, str>>,
            pub(crate) Path:     Option<Cow<'c, str>>,
            pub(crate) Secure:   Option<bool>,
            pub(crate) HttpOnly: Option<bool>,
            pub(crate) SameSite: Option<SameSitePolicy>,
        }
        impl<'c> SetCookie<'c> {
            pub fn Cookie(&self) -> (&str, &str) {
                let (name, value) = &self.Cookie;
                (name, &value)
            }
            pub fn Expires(&self) -> Option<&str> {
                self.Expires.as_deref()
            }
            pub const fn MaxAge(&self) -> Option<u64> {
                self.MaxAge
            }
            pub fn Domain(&self) -> Option<&str> {
                self.Domain.as_deref()
            }
            pub fn Path(&self) -> Option<&str> {
                self.Path.as_deref()
            }
            pub const fn Secure(&self) -> Option<bool> {
                self.Secure
            }
            pub const fn HttpOnly(&self) -> Option<bool> {
                self.HttpOnly
            }
            /// `Some`: `"Lax" | "None" | "Strict"`
            pub const fn SameSite(&self) -> Option<&'static str> {
                match &self.SameSite {
                    None         => None,
                    Some(policy) => Some(policy.as_str())
                }
            }

            pub(crate) fn from_raw(str: &'c str) -> Result<Self, String> {
                let mut r = byte_reader::Reader::new(str.as_bytes());

                let mut this = {
                    let name  = std::str::from_utf8(r.read_until(b"=")).map_err(|e| format!("Invalid Cookie name: {e}"))?;
                    r.consume("=").ok_or_else(|| format!("No `=` found in a `Set-Cookie` header value"))?;
                    let value =  ohkami_lib::percent_decode_utf8({
                        let mut bytes = r.read_until(b"; ");
                        let len = bytes.len();
                        if len >= 2 && bytes[0] == b'"' && bytes[len-1] == b'"' {
                            bytes = &bytes[1..(len-1)]
                        }
                        bytes
                    }).map_err(|e| format!("Invalid Cookie value: {e}"))?;

                    Self {
                        Cookie: (name, value),
                        Expires:  None,
                        MaxAge:   None,
                        Domain:   None,
                        Path:     None,
                        SameSite: None,
                        Secure:   None,
                        HttpOnly: None,
                    }
                };

                while r.consume("; ").is_some() {
                    let directive = r.read_until(b"; ");
                    let mut r = byte_reader::Reader::new(directive);
                    match r.consume_oneof([
                        "Expires", "Max-Age", "Domain", "Path", "SameSite", "Secure", "HttpOnly"
                    ]) {
                        Some(0) => {
                            r.consume("=").ok_or_else(|| format!("Invalid `Expires`: No `=` found"))?;
                            let value = std::str::from_utf8(r.read_until(b"; ")).map_err(|e| format!("Invalid `Expires`: {e}"))?;
                            this.Expires = Some(Cow::Borrowed(value))
                        },
                        Some(1) => {
                            r.consume("=").ok_or_else(|| format!("Invalid `Max-Age`: No `=` found"))?;
                            let value = r.read_until(b"; ").iter().fold(0, |secs, d| 10*secs + (*d - b'0') as u64);
                            this.MaxAge = Some(value)
                        }
                        Some(2) => {
                            r.consume("=").ok_or_else(|| format!("Invalid `Domain`: No `=` found"))?;
                            let value = std::str::from_utf8(r.read_until(b"; ")).map_err(|e| format!("Invalid `Domain`: {e}"))?;
                            this.Domain = Some(Cow::Borrowed(value))
                        },
                        Some(3) => {
                            r.consume("=").ok_or_else(|| format!("Invalid `Path`: No `=` found"))?;
                            let value = std::str::from_utf8(r.read_until(b"; ")).map_err(|e| format!("Invalid `Path`: {e}"))?;
                            this.Path = Some(Cow::Borrowed(value))
                        }
                        Some(4) => {
                            r.consume("=").ok_or_else(|| format!("Invalid `SameSite`: No `=` found"))?;
                            this.SameSite = SameSitePolicy::from_bytes(r.read_until(b"; "));
                        }
                        Some(5) => this.Secure = Some(true),
                        Some(6) => this.HttpOnly = Some(true),
                        _ => return Err((|| format!("Unkown directive: `{}`", r.remaining().escape_ascii()))())
                    }
                }

                Ok(this)
            }
        }

        pub struct SetCookieBuilder(SetCookie<'static>);
        impl SetCookieBuilder {
            #[inline]
            pub(crate) fn new(cookie_name: &'static str, cookie_value: impl Into<Cow<'static, str>>) -> Self {
                Self(SetCookie {
                    Cookie: (cookie_name, cookie_value.into()),
                    Expires: None, MaxAge: None, Domain: None, Path: None, Secure: None, HttpOnly: None, SameSite: None,
                })
            }
            pub(crate) fn build(self) -> String {
                let mut bytes = Vec::new();

                let (name, value) = self.0.Cookie; {
                    bytes.extend_from_slice(name.as_bytes());
                    bytes.push(b'=');
                    bytes.extend_from_slice(ohkami_lib::percent_encode(&value).as_bytes());
                }
                if let Some(Expires) = self.0.Expires {
                    bytes.extend_from_slice(b"; Expires=");
                    bytes.extend_from_slice(Expires.as_bytes());
                }
                if let Some(MaxAge) = self.0.MaxAge {
                    bytes.extend_from_slice(b"; Max-Age=");
                    bytes.extend_from_slice(MaxAge.to_string().as_bytes());
                }
                if let Some(Domain) = self.0.Domain {
                    bytes.extend_from_slice(b"; Domain=");
                    bytes.extend_from_slice(Domain.as_bytes());
                }
                if let Some(Path) = self.0.Path {
                    bytes.extend_from_slice(b"; Path=");
                    bytes.extend_from_slice(Path.as_bytes());
                }
                if let Some(true) = self.0.Secure {
                    bytes.extend_from_slice(b"; Secure");
                }
                if let Some(true) = self.0.HttpOnly {
                    bytes.extend_from_slice(b"; HttpOnly");
                }
                if let Some(SameSite) = self.0.SameSite {
                    bytes.extend_from_slice(b"; SameSite=");
                    bytes.extend_from_slice(SameSite.as_str().as_bytes());
                }

                unsafe {// SAFETY: All fields and punctuaters is UTF-8
                    String::from_utf8_unchecked(bytes)
                }
            }

            #[inline]
            pub fn Expires(mut self, Expires: impl Into<Cow<'static, str>>) -> Self {
                self.0.Expires = Some(Expires.into());
                self
            }
            #[inline]
            pub const fn MaxAge(mut self, MaxAge: u64) -> Self {
                self.0.MaxAge = Some(MaxAge);
                self
            }
            #[inline]
            pub fn Domain(mut self, Domain: impl Into<Cow<'static, str>>) -> Self {
                self.0.Domain = Some(Domain.into());
                self
            }
            #[inline]
            pub fn Path(mut self, Path: impl Into<Cow<'static, str>>) -> Self {
                self.0.Path = Some(Path.into());
                self
            }
            #[inline]
            pub const fn Secure(mut self) -> Self {
                self.0.Secure = Some(true);
                self
            }
            #[inline]
            pub const fn HttpOnly(mut self) -> Self {
                self.0.HttpOnly = Some(true);
                self
            }
            #[inline]
            pub const fn SameSiteLax(mut self) -> Self {
                self.0.SameSite = Some(SameSitePolicy::Lax);
                self
            }
            #[inline]
            pub const fn SameSiteNone(mut self) -> Self {
                self.0.SameSite = Some(SameSitePolicy::None);
                self
            }
            #[inline]
            pub const fn SameSiteStrict(mut self) -> Self {
                self.0.SameSite = Some(SameSitePolicy::Strict);
                self
            }
        }
    }

    /// Passed to `{Request/Response}.headers.set().Name( 〜 )` and
    /// append `value` to the header.
    /// 
    /// Here appended values are combined by `,`.
    /// 
    /// ---
    /// *example.rs*
    /// ```no_run
    /// use ohkami::prelude::*;
    /// use ohkami::header::append;
    /// 
    /// #[derive(Clone)]
    /// struct AppendServer(&'static str);
    /// impl FangAction for AppendServer {
    ///     async fn back<'b>(&'b self, res: &'b mut Response) {
    ///         res.headers.set()
    ///             .Server(append(self.0));
    ///     }
    /// }
    /// 
    /// #[tokio::main]
    /// async fn main() {
    ///     Ohkami::with(AppendServer("ohkami"),
    ///         "/".GET(|| async {"Hello, append!"})
    ///     ).howl("localhost:3000").await
    /// }
    /// ```
    pub fn append(value: impl Into<std::borrow::Cow<'static, str>>) -> private::Append {
        private::Append(value.into())
    }
}

pub mod prelude {
    pub use crate::{Request, Response, IntoResponse, Method, Status};
    pub use crate::utils::FangAction;

    #[cfg(any(feature="rt_tokio",feature="rt_async-std",feature="rt_worker"))]
    pub use crate::{Route, Ohkami};
}

/// Somthing almost [serde](https://crates.io/crates/serde).
/// 
/// ---
/// *not_need_serde_in_your_dependencies.rs*
/// ```
/// use ohkami::serde::Serialize;
/// 
/// #[derive(Serialize)]
/// struct User {
///     #[serde(rename = "username")]
///     name: String,
///     age:  u8,
/// }
/// ```
/// ---
pub mod serde {
    pub use ::ohkami_macros::{Serialize, Deserialize};
    pub use ::serde::ser::{self, Serialize, Serializer};
    pub use ::serde::de::{self, Deserialize, Deserializer};
    pub use ::serde_json as json;
}

// #[cfg(feature="websocket")]
// pub mod websocket {
//     pub use crate::x_websocket::*;
// }

#[doc(hidden)]
pub mod __internal__ {
    pub use ::serde;

    pub use ohkami_macros::consume_struct;

    pub use crate::fangs::Fangs;

    /* for benchmarks */
    #[cfg(feature="DEBUG")]
    #[cfg(any(feature="rt_tokio",feature="rt_async-std",feature="rt_worker"))]
    pub use crate::{
        request::{RequestHeader, RequestHeaders},
        response::{ResponseHeader, ResponseHeaders},
    };
}
