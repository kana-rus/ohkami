use std::{future::Future, pin::Pin};
use crate::{response::Response, result::Result, context::Context, utils::range::RangeList};


pub(crate) type HandleFunc = Box<dyn Fn(Context, RangeList) -> Pin<Box<dyn Future<Output=Result<Response>> + Send >> + Send + Sync>;

pub trait Param {}
pub trait Handler<P: Param> {
    fn into_handlefunc(self) -> (HandleFunc, u8/*path param num*/);
}

impl Param for () {}
impl<F, Fut> Handler<()> for F
where
F:   Fn() -> Fut + Send + Sync + 'static,
Fut: Future<Output=Result<Response>> + Send + 'static
{
    fn into_handlefunc(self) -> (HandleFunc, u8) {
        (
            Box::new(move |_, _| Box::pin(self())),
            0
        )
    }
}
impl Param for Context {}
impl<F, Fut> Handler<Context> for F
where
F:   Fn(Context) -> Fut + Send + Sync + 'static,
Fut: Future<Output=Result<Response>> + Send + 'static
{
    fn into_handlefunc(self) -> (HandleFunc, u8) {
        (
            Box::new(move |ctx, _| Box::pin(self(ctx))),
            0
        )
    }
}

impl Param for String {}
impl<F, Fut> Handler<String> for F
where
F:   Fn(String) -> Fut + Send + Sync + 'static,
Fut: Future<Output=Result<Response>> + Send + 'static
{
    fn into_handlefunc(self) -> (HandleFunc, u8) {
        (Box::new(move |ctx, params|
            match params.get1() {
                Some(range) => {
                    let param = ctx.buffer.read_str(&range).to_owned();
                    Box::pin(self(param))
                },
                None => unreachable!(/* already validated in Server::add_handler */),
            }
        ), 1)
    }
}
impl Param for (Context, String) {}
impl<F, Fut> Handler<(Context, String)> for F
where
F:   Fn(Context, String) -> Fut + Send + Sync + 'static,
Fut: Future<Output=Result<Response>> + Send + 'static
{
    fn into_handlefunc(self) -> (HandleFunc, u8) {
        (Box::new(move |ctx, params|
            match params.get1() {
                Some(range) => {
                    let param = ctx.buffer.read_str(&range).to_owned();
                    Box::pin(self(ctx, param))
                },
                None => unreachable!(/* already validated in Server::add_handler */),
            }
        ), 1)
    }
}

macro_rules! impl_handler_with_int {
    ( $( $int_type:ty )* ) => {
        $(
            impl Param for $int_type {}
            impl<F, Fut> Handler<$int_type> for F
            where
                F:   Fn($int_type) -> Fut + Send + Sync + 'static,
                Fut: Future<Output=Result<Response>> + Send + 'static
            {
                fn into_handlefunc(self) -> (HandleFunc, u8) {
                    (Box::new(move |ctx, params|
                        match params.get1() {
                            Some(range) => {
                                let parsed = ctx.buffer.read_str(&range).parse::<$int_type>();
                                match parsed {
                                    Ok(param) => Box::pin(self(param)),
                                    _ => Box::pin(async {Err(Response::BadRequest("format of path param is wrong"))})
                                }
                            },
                            None => unreachable!(/* already validated in Server::add_handler */),
                        }
                    ), 1)
                }
            }
            impl Param for (Context, $int_type) {}
            impl<F, Fut> Handler<(Context, $int_type)> for F
            where
                F:   Fn(Context, $int_type) -> Fut + Send + Sync + 'static,
                Fut: Future<Output=Result<Response>> + Send + 'static
            {
                fn into_handlefunc(self) -> (HandleFunc, u8) {
                    (Box::new(move |ctx, params|
                        match params.get1() {
                            Some(range) => {
                                let parsed = ctx.buffer.read_str(&range).parse::<$int_type>();
                                match parsed {
                                    Ok(param) => Box::pin(self(ctx, param)),
                                    _ => Box::pin(async {Err(Response::BadRequest("format of path param is wrong"))})
                                }
                            },
                            None => unreachable!(/* already validated in Server::add_handler */),
                        }
                    ), 1)
                }
            }
        )*
    };
} impl_handler_with_int!(u64 usize i64);

macro_rules! impl_handler_with_2ints {
    ( $( ($int1:ty, $int2:ty) )* ) => {
        $(
            impl Param for ($int1, $int2) {}
            impl<F, Fut> Handler<($int1, $int2)> for F
            where
                F:   Fn($int1, $int2) -> Fut + Send + Sync + 'static,
                Fut: Future<Output=Result<Response>> + Send + 'static
            {
                fn into_handlefunc(self) -> (HandleFunc, u8) {
                    (Box::new(move |ctx, params|
                        match params.get2() {
                            Some((range1, range2)) => {
                                let parsed1 = ctx.buffer.read_str(&range1).parse::<$int1>();
                                let parsed2 = ctx.buffer.read_str(&range2).parse::<$int2>();
                                match (parsed1, parsed2) {
                                    (Ok(param1), Ok(param2)) => Box::pin(self(param1, param2)),
                                    _ => Box::pin(async {Err(Response::BadRequest("format of path param is wrong"))})
                                }
                            },
                            None => unreachable!(/* already validated in Server::add_handler */),
                        }
                    ), 2)
                }
            }
            impl Param for (Context, $int1, $int2) {}
            impl<F, Fut> Handler<(Context, $int1, $int2)> for F
            where
                F:   Fn(Context, $int1, $int2) -> Fut + Send + Sync + 'static,
                Fut: Future<Output=Result<Response>> + Send + 'static
            {
                fn into_handlefunc(self) -> (HandleFunc, u8) {
                    (Box::new(move |ctx, params|
                        match params.get2() {
                            Some((range1, range2)) => {
                                let parsed1 = ctx.buffer.read_str(&range1).parse::<$int1>();
                                let parsed2 = ctx.buffer.read_str(&range2).parse::<$int2>();
                                match (parsed1, parsed2) {
                                    (Ok(param1), Ok(param2)) => Box::pin(self(ctx, param1, param2)),
                                    _ => Box::pin(async {Err(Response::BadRequest("format of path param is wrong"))})
                                }
                            },
                            None => unreachable!(/* already validated in Server::add_handler */),
                        }
                    ), 2)
                }
            }
        )*
    };
} impl_handler_with_2ints!(
    (u64,  u64) (u64,  usize) (u64,  i64)
    (usize,u64) (usize,usize) (usize,i64)
    (i64,  u64) (i64,  usize) (i64,  i64) 
);

/*
    HAVE TO COME UP WITH MORE EFFICIENT WAY for
    - 3ints
    - 4ints
    - string_2ints
    - string_3ints
*/

// macro_rules! impl_handler_with_3ints {
//     ( $( ($int1:ty, $int2:ty, $int3:ty) )* ) => {
//         $(
//             impl Param for ($int1, $int2, $int3) {}
//             impl<F, Fut> Handler<($int1, $int2, $int3)> for F
//             where
//                 F:   Fn(Context, $int1, $int2, $int3) -> Fut + Send + Sync + 'static,
//                 Fut: Future<Output=Result<Response>> + Send + 'static
//             {
//                 fn into_handlefunc(self) -> (HandleFunc, u8) {
//                     Box::new(move |ctx, params|
//                         match params.get3() {
//                             Some((range1, range2, range3)) => {
//                                 let parsed1 = ctx.buffer.read_str(&range1).parse::<$int1>();
//                                 let parsed2 = ctx.buffer.read_str(&range2).parse::<$int2>();
//                                 let parsed2 = ctx.buffer.read_str(&range3).parse::<$int3>();
//                                 match (parsed1, parsed2, parsed3) {
//                                     (Ok(param1), Ok(param2), Ok(param3)) => Box::pin(self(ctx, param1, param2, param3)),
//                                     _ => Box::pin(async {Err(Response::BadRequest("format of path param is wrong"))})
//                                 }
//                             },
//                             None => unreachable!(/* already validated in Server::add_handler */),
//                         }
//                     )
//                 }
//             }
//         )*
//     };
// }

macro_rules! impl_handler_with_string_int {
    ( $($int:ty)* ) => {
        $(
            impl Param for (String, $int) {}
            impl<F, Fut> Handler<(String, $int)> for F
            where
                F:   Fn(String, $int) -> Fut + Send + Sync + 'static,
                Fut: Future<Output=Result<Response>> + Send + 'static
            {
                fn into_handlefunc(self) -> (HandleFunc, u8) {
                    (Box::new(move |ctx, params|
                        match params.get2() {
                            Some((range_string, range_int)) => {
                                let parsed = ctx.buffer.read_str(&range_int).parse::<$int>();
                                match parsed {
                                    Ok(param) => {
                                        let string = ctx.buffer.read_str(&range_string).to_owned();
                                        Box::pin(self(string, param))
                                    },
                                    _ => Box::pin(async {Err(Response::BadRequest("format of path param is wrong"))})
                                }
                            },
                            None => unreachable!(/* already validated in Server::add_handler */),
                        }
                    ), 2)
                }
            }
            impl Param for ($int, String) {}
            impl<F, Fut> Handler<($int, String)> for F
            where
                F:   Fn($int, String) -> Fut + Send + Sync + 'static,
                Fut: Future<Output=Result<Response>> + Send + 'static
            {
                fn into_handlefunc(self) -> (HandleFunc, u8) {
                    (Box::new(move |ctx, params|
                        match params.get2() {
                            Some((range_int, range_string)) => {
                                let parsed = ctx.buffer.read_str(&range_int).parse::<$int>();
                                match parsed {
                                    Ok(int) => {
                                        let string = ctx.buffer.read_str(&range_string).to_owned();
                                        Box::pin(self(int, string))
                                    },
                                    _ => Box::pin(async {Err(Response::BadRequest("format of path param is wrong"))})
                                }
                            },
                            None => unreachable!(/* already validated in Server::add_handler */),
                        }
                    ), 2)
                }
            }
            impl Param for (Context, String, $int) {}
            impl<F, Fut> Handler<(Context, String, $int)> for F
            where
                F:   Fn(Context, String, $int) -> Fut + Send + Sync + 'static,
                Fut: Future<Output=Result<Response>> + Send + 'static
            {
                fn into_handlefunc(self) -> (HandleFunc, u8) {
                    (Box::new(move |ctx, params|
                        match params.get2() {
                            Some((range_string, range_int)) => {
                                let parsed = ctx.buffer.read_str(&range_int).parse::<$int>();
                                match parsed {
                                    Ok(param) => {
                                        let string = ctx.buffer.read_str(&range_string).to_owned();
                                        Box::pin(self(ctx, string, param))
                                    },
                                    _ => Box::pin(async {Err(Response::BadRequest("format of path param is wrong"))})
                                }
                            },
                            None => unreachable!(/* already validated in Server::add_handler */),
                        }
                    ), 2)
                }
            }
            impl Param for (Context, $int, String) {}
            impl<F, Fut> Handler<(Context, $int, String)> for F
            where
                F:   Fn(Context, $int, String) -> Fut + Send + Sync + 'static,
                Fut: Future<Output=Result<Response>> + Send + 'static
            {
                fn into_handlefunc(self) -> (HandleFunc, u8) {
                    (Box::new(move |ctx, params|
                        match params.get2() {
                            Some((range_int, range_string)) => {
                                let parsed = ctx.buffer.read_str(&range_int).parse::<$int>();
                                match parsed {
                                    Ok(int) => {
                                        let string = ctx.buffer.read_str(&range_string).to_owned();
                                        Box::pin(self(ctx, int, string))
                                    },
                                    _ => Box::pin(async {Err(Response::BadRequest("format of path param is wrong"))})
                                }
                            },
                            None => unreachable!(/* already validated in Server::add_handler */),
                        }
                    ), 2)
                }
            }
        )*
    };
} impl_handler_with_string_int!(u64 usize i64);


#[cfg(test)]
mod test {
    use crate::{context::Context, response::Response, result::Result, components::json::JSON, json};
    use super::{Handler, Param, HandleFunc};

    async fn a(_: Context) -> Result<Response> {
        Response::OK(None::<JSON>)
    }
    async fn b(_: Context, id: usize) -> Result<Response> {
        Response::OK(json!("id": id))
    }

    struct Handlers(
        Vec<HandleFunc>
    );impl Handlers {
        fn new() -> Self {
            Self(Vec::new())
        }
        fn push<H: Handler<P> + 'static, P: Param>(&mut self, handler: H) {
            self.0.push(handler.into_handlefunc().0)
        }
    }

    #[test]
    fn different_signature_handlers() {
        let mut handlers = Handlers::new();

        handlers.push(a);
        handlers.push(b);
    }
}