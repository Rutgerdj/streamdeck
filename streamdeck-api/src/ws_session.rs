use std::time::{Duration, Instant};
use actix::{fut, Running, Actor, AsyncContext, StreamHandler, ActorContext, Handler, Addr, WrapFuture, ActorFutureExt, ContextFutureSpawner};
use actix_web_actors::ws;
use streamdeck_interface::hub::{DeckHub, HubMessage};

use crate::server::{Server, WsMessage};



pub type SessionId = usize;
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);


#[derive(Debug)]
pub struct WsSession {
    /// unique session id
    pub id: SessionId,

    pub hb: Instant,

    pub server: Addr<Server>,

    pub i: Instant
}


impl WsSession {
    /// helper method that sends ping to client every second.
    ///
    /// also this method checks heartbeats from client
    fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            // check client heartbeats
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                // heartbeat timed out
                println!("Websocket Client heartbeat failed, disconnecting!");
                
                // stop actor
                ctx.stop();

                // don't try to send a ping
                return;
            }
            ctx.ping(b"");
        });
    }
}


impl Actor for WsSession {
    type Context = ws::WebsocketContext<Self>;

    /// Method is called on actor start.
    fn started(&mut self, ctx: &mut Self::Context) {
        // we'll start heartbeat process on session start.
        self.hb(ctx);
        let addr = ctx.address();

        self.server
            .send(crate::server::Connect {
                addr: addr.clone(),
            })
            .into_actor(self)
            .then(move |res, act, ctx| {
                match res {
                    Ok(res) => act.id = res,
                    // something is wrong with server
                    _ => ctx.stop(),
                }
                fut::ready(())
            })
            .wait(ctx);
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        log::info!("WS Server Stopping");
        Running::Stop
    }
    
}



/// WebSocket message handler
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        let msg = match msg {
            Err(_) => {
                ctx.stop();
                return;
            }
            Ok(msg) => msg,
        };

        match msg {
            ws::Message::Ping(msg) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            ws::Message::Pong(_) => {
                self.hb = Instant::now();
            }
            ws::Message::Text(text) => {
                let m = text.trim();
                println!("{}", m);
                let parts = m.split_ascii_whitespace().collect::<Vec<&str>>();
                match *parts {
                    ["b", x] => {
                        log::info!("Sending");
                        self.server.do_send(WsMessage {msg: x.to_owned()});
                        log::info!("Sent!");
                    }
                    _ => {}
                }
            }
            ws::Message::Close(reason) => {
                ctx.close(reason);
                ctx.stop();
            }
            ws::Message::Continuation(_) => {
                ctx.stop();
            }
            _ => (),
        }
    }
}

impl Handler<WsMessage> for WsSession {
    type Result = ();

    fn handle(&mut self, msg: WsMessage, ctx: &mut Self::Context) -> Self::Result {
        ctx.text(msg.msg);
    }
}

impl Handler<HubMessage> for WsSession {
    type Result = ();

    fn handle(&mut self, msg: HubMessage, ctx: &mut Self::Context) -> Self::Result {
        ctx.text(msg.0);
    }
}


impl Handler<crate::server::Connect> for DeckHub {
    type Result = SessionId;

    fn handle(&mut self, msg: crate::server::Connect, _: &mut Self::Context) -> Self::Result {

        1
    }
}