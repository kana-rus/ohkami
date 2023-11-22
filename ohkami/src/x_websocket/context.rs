use std::{future::Future, borrow::Cow};
use super::websocket::Config;
use super::{WebSocket, sign, assume_upgradable};
use crate::{Response, Context, Request};
use crate::__rt__::{task};
use crate::http::{Method};


pub struct WebSocketContext<UFH: UpgradeFailureHandler = DefaultUpgradeFailureHandler> {
    c:                      Context,

    config:                 Config,

    on_failed_upgrade:      UFH,

    sec_websocket_key:      Cow<'static, str>,
    selected_protocol:      Option<Cow<'static, str>>,
    sec_websocket_protocol: Option<Cow<'static, str>>,
}

pub trait UpgradeFailureHandler {
    fn handle(self, error: UpgradeError);
}
pub enum UpgradeError {
    NotRequestedUpgrade,
}
pub struct DefaultUpgradeFailureHandler;
impl UpgradeFailureHandler for DefaultUpgradeFailureHandler {
    fn handle(self, _: UpgradeError) {/* discard error */}
}

impl WebSocketContext {
    pub(crate) fn new(c: Context, req: &mut Request) -> Result<Self, Response> {
        if req.method() != Method::GET {
            return Err((|| c.BadRequest().text("Method is not `GET`"))())
        }
        if req.header("Connection") != Some("upgrade") {
            return Err((|| c.BadRequest().text("Connection header is not `upgrade`"))())
        }
        if req.header("Upgrade") != Some("websocket") {
            return Err((|| c.BadRequest().text("Upgrade header is not `websocket`"))())
        }
        if req.header("Sec-WebSocket-Version") != Some("13") {
            return Err((|| c.BadRequest().text("Sec-WebSocket-Version header is not `13`"))())
        }

        let sec_websocket_key = Cow::Owned(req.header("Sec-WebSocket-Key")
            .ok_or_else(|| c.BadRequest().text("Sec-WebSocket-Key header is missing"))?
            .to_string());

        let sec_websocket_protocol = req.header("Sec-WebSocket-Protocol")
            .map(|swp| Cow::Owned(swp.to_string()));

        Ok(Self {c,
            config:            Config::default(),
            on_failed_upgrade: DefaultUpgradeFailureHandler,
            selected_protocol: None,
            sec_websocket_key,
            sec_websocket_protocol,
        })
    }
}

impl WebSocketContext {
    pub fn write_buffer_size(mut self, size: usize) -> Self {
        self.config.write_buffer_size = size;
        self
    }
    pub fn max_write_buffer_size(mut self, size: usize) -> Self {
        self.config.max_write_buffer_size = size;
        self
    }
    pub fn max_message_size(mut self, size: usize) -> Self {
        self.config.max_message_size = Some(size);
        self
    }
    pub fn max_frame_size(mut self, size: usize) -> Self {
        self.config.max_frame_size = Some(size);
        self
    }
    pub fn accept_unmasked_frames(mut self) -> Self {
        self.config.accept_unmasked_frames = true;
        self
    }

    pub fn protocols<S: Into<Cow<'static, str>>>(mut self, protocols: impl Iterator<Item = S>) -> Self {
        if let Some(req_protocols) = &self.sec_websocket_protocol {
            self.selected_protocol = protocols.map(Into::into)
                .find(|p| req_protocols.split(',').any(|req_p| req_p.trim() == p))
        }
        self
    }
}

impl WebSocketContext {
    pub fn on_upgrade<
        Fut: Future<Output = ()> + Send + 'static,
    >(
        self,
        handler: impl Fn(WebSocket) -> Fut + Send + Sync + 'static
    ) -> Response {
        fn sign(sec_websocket_key: &str) -> String {
            let mut sha1 = sign::Sha1::new();
            sha1.write(sec_websocket_key.as_bytes());
            sha1.write(b"258EAFA5-E914-47DA-95CA-C5AB0DC85B11");
            sign::Base64::<{sign::SHA1_SIZE}>::encode(sha1.sum())
        }

        let Self {
            mut c,
            config,
            on_failed_upgrade,
            selected_protocol,
            sec_websocket_key,
            ..
        } = self;

        task::spawn({
            async move {
                let stream = match c.upgrade_id {
                    Some(id) => assume_upgradable(id).await,
                    None     => return on_failed_upgrade.handle(UpgradeError::NotRequestedUpgrade),
                };

                let ws = WebSocket::new(stream, config);
                handler(ws).await
            }
        });

        c.headers
            .custom("Connection", "Upgrade")
            .custom("Upgrade", "websocket")
            .custom("Sec-WebSocket-Accept", sign(&sec_websocket_key));
        if let Some(protocol) = selected_protocol {
            c.headers
                .custom("Sec-WebSocket-Protocol", protocol);
        }
        c.SwitchingProtocols()
    }
}