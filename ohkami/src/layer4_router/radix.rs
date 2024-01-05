use crate::{
    Request,
    Response,
    response as r,
    layer0_lib::{Method, Status, Slice, percent_decode},
    layer3_fang_handler::{Handler, FrontFang, BackFang},
};

#[cfg(feature="websocket")]
use crate::websocket::{
    UpgradeID,
    request_upgrade_id,
};


/*===== defs =====*/
pub(crate) struct RadixRouter {
    pub(super) GET:    Node,
    pub(super) PUT:    Node,
    pub(super) POST:   Node,
    pub(super) PATCH:  Node,
    pub(super) DELETE: Node,
    pub(super) HEADfangs:    (&'static [FrontFang], &'static [BackFang]),
    pub(super) OPTIONSfangs: (&'static [FrontFang], &'static [BackFang]),
}

pub(super) struct Node {
    pub(super) patterns: &'static [Pattern],
    pub(super) front:    &'static [FrontFang],
    pub(super) handler:  Option<Handler>,
    pub(super) back:     &'static [BackFang],
    pub(super) children: Vec<Node>,
}

pub(super) enum Pattern {
    Static(&'static [u8]),
    Param,
} const _: () = {
    #[cfg(test)]
    impl std::fmt::Debug for Pattern {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str(match self {
                Self::Param         => ":Param",
                Self::Static(bytes) => std::str::from_utf8(bytes).unwrap(),
            })
        }
    }
};


/*===== impls =====*/
#[cfg(feature="websocket")] type HandleResult = (Response, Option<UpgradeID>);
#[cfg(feature="websocket")] fn __no_upgrade(res: Response) -> HandleResult {
    (res, None)
}

#[cfg(not(feature="websocket"))] type HandleResult = Response;
#[cfg(not(feature="websocket"))] fn __no_upgrade(res: Response) -> HandleResult {
    res
}


impl RadixRouter {
    pub(crate) async fn handle(
        &self,
        req: &mut Request,
    ) -> HandleResult {
        let search_result = match req.method {
            Method::GET    => self.GET   .search(req/*.path_bytes()*/),
            Method::PUT    => self.PUT   .search(req/*.path_bytes()*/),
            Method::POST   => self.POST  .search(req/*.path_bytes()*/),
            Method::PATCH  => self.PATCH .search(req/*.path_bytes()*/),
            Method::DELETE => self.DELETE.search(req/*.path_bytes()*/),
            
            Method::HEAD => {
                let (front, back) = self.HEADfangs;

                let mut res = 'res: {
                    for ff in front {
                        if let Err(err_res) = ff.0(req) {
                            break 'res err_res
                        }
                    }

                    let target = match self.GET.search(req/*.path_bytes()*/) {
                        Ok(Some(node)) => node,
                        Ok(None)       => break 'res r::Empty::NotFound().into(),
                        Err(err_res)   => break 'res err_res,
                    };

                    let Response { headers, .. } = target.handle_discarding_upgrade(req).await;
                    Response {
                        headers,
                        status:  Status::NoContent,
                        content: None,
                    }
                };

                for bf in back {
                    res = bf.0(req, res)
                }

                return __no_upgrade(res);
            }

            Method::OPTIONS => {
                let (front, back) = self.OPTIONSfangs;

                let mut res = 'res: {
                    for ff in front {
                        if let Err(err_res) = ff.0(req) {
                            break 'res err_res
                        }
                    }
                    r::Empty::NoContent().into()
                };

                for bf in back {
                    res = bf.0(req, res)
                }
                
                return __no_upgrade(res);
            }
        };

        match search_result {
            Ok(Some(node)) => node.handle(req).await,
            Ok(None)       => __no_upgrade(r::Empty::NotFound().into()),
            Err(err_res)   => __no_upgrade(err_res),
        }
    }
}

impl Node {
    #[inline] pub(super) async fn handle(&self, req: &mut Request) -> HandleResult {
        match &self.handler {
            Some(handler) => {
                #[cfg(feature="websocket")]
                let upgrade_id = match (handler.requires_upgrade).then(|| async {
                    let id = request_upgrade_id().await;
                    req.upgrade_id = Some(id);
                    id
                }) {None => None, Some(id) => Some(id.await)};

                let mut res = (handler.proc)(req).await;
                for b in self.back {
                    res = b.0(req, res);
                }

                #[cfg(feature="websocket")]
                {(res, upgrade_id)}
                #[cfg(not(feature="websocket"))]
                {res}
            }
            None => __no_upgrade(r::Empty::NotFound().into()),
        }
    }

    #[inline] pub(super) async fn handle_discarding_upgrade(&self, req: &mut Request) -> Response {
        match &self.handler {
            Some(handler) => {
                let mut res = (handler.proc)(req).await;
                for b in self.back {
                    res = b.0(req, res);
                }
                res
            }
            None => r::Empty::NotFound().into()
        }
    }

    pub(super/* for test */) fn search(&self, req: &mut Request) -> Result<Option<&Node>, Response> {
        let mut target = self;

        // SAFETY:
        // 1. `req` must be alive while `search`
        // 2. `Request` DOESN'T have method that mutates `path`,
        //    So what `path` refers to is NEVER changed by any other process
        //    while `search`
        let path_bytes_maybe_percent_encoded = unsafe {req.path_bytes()};
        // Decode percent encodings in `path_bytes_maybe_percent_encoded`,
        // without checking entire it is valid UTF-8.
        let decoded = percent_decode(path_bytes_maybe_percent_encoded);
        let mut path: &[u8] = &decoded;

        loop {
            for ff in target.front {
                ff.0(req)?
            }

            for pattern in target.patterns {
                if path.is_empty() || unsafe {path.get_unchecked(0)} != &b'/' {
                    // At least one `pattern` to match is remaining
                    // but path doesn't start with '/'
                    return Ok(None)
                }
                path = unsafe {path.get_unchecked(1..)};
                match pattern {
                    Pattern::Static(s)  => path = match path.strip_prefix(*s) {
                        Some(remaining) => remaining,
                        None            => return Ok(None),
                    },
                    Pattern::Param      => {
                        let (param, remaining) = split_next_section(path);
                        req.path.params.push(unsafe {Slice::from_bytes(param)});
                        path = remaining;
                    },
                }
            }

            if path.is_empty() {
                return Ok(Some(target))
            } else {
                target = match target.matchable_child(path) {
                    Some(child) => child,
                    None        => return Ok(None),
                }
            }
        }
    }
}


/*===== utils =====*/
impl Node {
    #[inline] fn matchable_child(&self, path: &[u8]) -> Option<&Node> {
        for child in &self.children {
            if child.patterns.first()?.is_matchable_to(path) {
                return Some(child)
            }
        }
        None
    }
}

impl Pattern {
    #[inline(always)] fn is_matchable_to(&self, path: &[u8]) -> bool {
        match self {
            Self::Param     => true,
            Self::Static(s) => (&path[1..]/* skip initial '/' */).starts_with(s),
        }
    }
}

#[inline] fn split_next_section(path: &[u8]) -> (&[u8], &[u8]) {
    let len = path.len();
    let mut slash = len; for i in 0..len {
        if b'/' == path[i] {slash = i}
    }

    let after_slash = (slash + 1/* skip `/` */).min(len/* considering: `path` ends with `/` */);
    let ptr         = path.as_ptr();

    unsafe {(
        std::slice::from_raw_parts(ptr,                  slash),
        std::slice::from_raw_parts(ptr.add(after_slash), len - after_slash),
    )}
}
