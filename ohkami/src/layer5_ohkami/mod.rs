mod with_fangs;
mod build;
mod howl;

use crate::{
    layer3_fang_handler::Fang,
    layer4_router::TrieRouter,
};


/// <br/>
/// 
/// ```ignore
/// async fn main() {
///     let api_ohkami = Ohkami::new()(
///         "/users".
///             POST(create_user),
///         "/users/:id".
///             GET(get_user_by_id).
///             PATCH(update_user),
///     );
/// 
///     // I'd like to use `auth` and `log` fang...
///     
///     let api_ohkami = Ohkami::with((auth, log))(
///         "/users".
///             POST(create_user),
///         "/users/:id".
///             GET(get_user_by_id).
///             PATCH(update_user),
///     );
/// 
///     // (Actually, this `log` fang of api_ohkami is duplicated with
///     // `log` fang of the root ohkami below, but it's no problem
///     // because they are merged internally.)
/// 
///     Ohkami::with((log,))(
///         "/hc" .GET(health_check),
///         "/api".By(api_ohkami),
///     ).howl(3000).await
/// }
/// ```
/// 
/// <br/>
/// 
/// ## fang schema
/// - front
///   - `(&mut Context, Request) -> Request`
///   - `(&mut Context, Request) -> Result<Request, Response>` (for early returning response)
/// - back
///   - `(Response) -> Response`
/// 
/// ## handler schema
/// - async (`Context`) -> `Response`
/// - async (`Context`, {path_params}) -> `Response`
/// - async (`Context`, some {impl `FromRequest`}s) -> `Response`
/// - async (`Context`, {path_params}, some {impl `FromRequest`}s) -> `Response`
/// 
/// path_params :
///   - `String`
///   - `u8` ~ `u128`, `usize`
///   - and tuple of them
pub struct Ohkami {
    pub(crate) routes: TrieRouter,

    /// apply just before `howl`
    pub(crate) fangs:  Vec<Fang>,
}

impl Ohkami {
    pub fn new() -> Self {
        Self {
            routes: TrieRouter::new(),
            fangs:  Vec::new(),
        }
    }

    pub fn with<G>(fangs: impl with_fangs::Fangs<G>) -> Self {
        Self {
            routes: TrieRouter::new(),
            fangs:  fangs.collect(),
        }
    }
}

impl Ohkami {
    pub(crate) fn into_router(self) -> TrieRouter {
        let Self { mut routes, fangs } = self;
        for fang in fangs {
            routes = routes.apply_fang(fang)
        }
        routes
    }
}
