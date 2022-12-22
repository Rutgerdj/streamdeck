use std::collections::HashMap;
use actix::{Actor, Handler, Message, Addr};
use rand::Rng;
use streamdeck_interface::{hub::{DeckHub, Broadcast, AddListener, HubMessage}, deckactor::MsgType};

use crate::ws_session::{SessionId, WsSession};

#[derive(Message)]
#[rtype(result = "()")]
pub struct WsMessage {
    pub msg: String
}

#[derive(Message)]
#[rtype(usize)]
pub struct Connect {
    pub addr: Addr<WsSession>,
}


pub struct Server {
    pub sessions: HashMap<SessionId, Addr<WsSession>>,
    rng: rand::rngs::ThreadRng,
    hub: Addr<DeckHub>
}

impl Server {
    pub fn new(hub: Addr<DeckHub>) -> Self {
        Self {
            sessions: HashMap::new(),
            rng: rand::thread_rng(),
            hub
        }
    }
}

impl Actor for Server {
    type Context = actix::Context<Self>;
}

impl Handler<Connect> for Server {
    type Result = usize;

    fn handle(&mut self, msg: Connect, _: &mut actix::Context<Self>) -> Self::Result {
        println!("Someone Connected");
        let id = self.rng.gen::<usize>();
        msg.addr.do_send(WsMessage {
            msg: format!("Your ID is {}", id)
        });
        self.sessions.insert(id, msg.addr.clone());

        let addr = msg.addr.recipient::<HubMessage>();
        self.hub.do_send(AddListener(id, addr));
        id
    }
}

impl Handler<WsMessage> for Server {
    type Result = ();

    fn handle(&mut self, msg: WsMessage, _: &mut actix::Context<Self>) -> Self::Result {
        log::info!("Got a message: {}", msg.msg);
        if let Ok(x) = msg.msg.parse::<u8>() {
            self.hub.do_send(Broadcast(MsgType::BrightnessChange(x)))
        } else {
            self.hub.do_send(Broadcast(MsgType::BrightnessChange(1)))
        }
    }
}