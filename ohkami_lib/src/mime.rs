/// Based on mimes lined up in <https://github.com/hyperium/mime/blob/master/mime-parse/src/constants.rs>
/// 
/// In current version, this **ONLY** support **UTF-8** as charset of `text/〜` files !
pub const fn get_by_extension(extension: &str) -> Option<&'static str> {
    match extension.as_bytes() {
        b"txt"   => Some("text/plain; charset=UTF-8"),
        b"html"  => Some("text/html; charset=UTF-8"),
        b"css"   => Some("text/css; charset=UTF-8"),
        b"js"    => Some("text/javascript"),
        b"xml"   => Some("text/xml"),
        b"csv"   => Some("text/csv; charset=UTF-8"),
        b"tsv"   => Some("text/tab-separated-values; charset=UTF-8"),
        b"vcard" => Some("text/vcard"),

        b"jpeg"  => Some("image/jpeg"),
        b"gif"   => Some("image/gif"),
        b"png"   => Some("image/png"),
        b"svg"   => Some("image/svg+xml"),

        b"woff"  => Some("font/woff"),
        b"woff2" => Some("font/woff2"),

        b"json"  => Some("application/json"),
        b"pdf"   => Some("application/pdf"),

        _ => None
    }
}
