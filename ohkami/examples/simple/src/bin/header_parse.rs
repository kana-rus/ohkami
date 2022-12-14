use ohkami::prelude::*;

fn main() -> Result<()> {
    Ohkami::default()
        .GET("/", reflect_header_host)
        .GET("/custom", reflect_header_custom)
        .howl(":3000")
}

async fn reflect_header_host(c: Context) -> Result<Response> {
    let host = c.req.header(Header::Host)?;
    c.OK(format!("requested from {host}"))
}

async fn reflect_header_custom(c: Context) -> Result<Response> {
    let custom_header_value = c.req.header("X-Custom")?;
    c.OK(format!("`X-Custom`'s value is {custom_header_value}"))
}