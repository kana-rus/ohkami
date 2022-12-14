use ohkami::prelude::*;

fn main() -> Result<()> {
    Ohkami::default()
        .GET("/sleepy/:time", sleepy_hello)
        .GET("/sleepy/:time/:name", sleepy_hello_with_name)
        .howl(":3000")
}

async fn sleepy_hello(time: u64) -> Result<Response> {
    (time < 30)
        ._else(|| Response::BadRequest("sleeping time (sec) must be less than 30."))?;
    std::thread::sleep(
        std::time::Duration::from_secs(time)
    );
    Response::OK("Hello, I'm sleepy...")
}

async fn sleepy_hello_with_name(time: u64, name: String) -> Result<Response> {
    (time < 30)
        ._else(|| Response::BadRequest("sleeping time (sec) must be less than 30."))?;
    std::thread::sleep(
        std::time::Duration::from_secs(time)
    );
    Response::OK(format!("Hello {name},,, I'm extremely sleepy..."))
}
