use serde::{Serialize, Deserialize};
use ohkami_lib::serde_urlencoded;
use crate::typed::PayloadType;


pub struct URLEncoded;

impl PayloadType for URLEncoded {
    const MIME_TYPE: &'static str = "application/x-www-form-urlencoded";

    #[inline]
    fn parse<'req, T: Deserialize<'req>>(bytes: &'req [u8]) -> Result<T, impl crate::serde::de::Error> {
        serde_urlencoded::from_bytes(bytes)
    }

    fn bytes<T: Serialize>(value: &T) -> Result<Vec<u8>, impl crate::serde::ser::Error> {
        serde_urlencoded::to_string(value).map(String::into_bytes)
    }
}
