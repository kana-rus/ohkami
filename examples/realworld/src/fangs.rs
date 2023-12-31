use ohkami::{Fang, IntoFang, Context, Request, Response};


pub struct Auth {
    condition: Option<fn(&Request)->bool>
}
impl Default for Auth {
    fn default() -> Self {
        Auth {
            condition: None,
        }
    }
}
impl Auth {
    pub fn with_condition(condition: fn(&Request)->bool) -> Self {
        Auth {
            condition: Some(condition),
        }
    }
}
impl IntoFang for Auth {
    fn into_fang(self) -> Fang {
        Fang(move |c: &mut Context, req: &mut Request| {
            if !self.condition.is_some_and(|cond| cond(req)) {
                return Ok(());
            }

            todo!()
        })
    }
}

pub struct LogRequest;
impl IntoFang for LogRequest {
    fn into_fang(self) -> Fang {
        Fang(|req: &mut Request| {
            let method = req.method;
            let path   = req.path();

            tracing::info!("{method:<7} {path}");
        })
    }
}

pub struct LogResponse;
impl IntoFang for LogResponse {
    fn into_fang(self) -> Fang {
        Fang(|res: Response| {
            tracing::info!("{res:?}");

            res
        })
    }
}