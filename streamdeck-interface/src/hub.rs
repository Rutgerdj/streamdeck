use std::collections::HashMap;

use actix::{Actor, Handler, Message, AsyncContext};
use streamdeck::StreamDeck;

use crate::{deckactor::DeckActor, deckactor::{ButtonChange, MsgType}, deckstate::DeckHandler};

#[derive(Message)]
#[rtype(result = "bool")]
pub struct Disconnect(pub u16);

#[derive(Message)]
#[rtype(result = "bool")]
pub struct Connect {
    devid: u16,
    hub: actix::Addr<DeckHub>,
    deck: StreamDeck,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Broadcast(pub MsgType);

impl Connect {
    pub fn new(devid: u16, hub: actix::Addr<DeckHub>, deck: StreamDeck) -> Self {
        Connect { devid, hub, deck }
    }
}

pub struct DeckHub {
    connected_devices: HashMap<u16, actix::Addr<DeckActor>>,
    state: DeckHandler,
}

impl Actor for DeckHub {
    type Context = actix::Context<Self>;
}

impl DeckHub {
    pub fn new(state: DeckHandler) -> Self {
        DeckHub {
            connected_devices: HashMap::new(),
            state
        }
    }
}

impl Handler<Connect> for DeckHub {
    type Result = bool;

    fn handle(&mut self, msg: Connect, _ctx: &mut Self::Context) -> Self::Result {
        // Check if there is already a device connected with this ID
        // Should only happen in really rare race conditions, but just to be sure.
        if self.connected_devices.contains_key(&msg.devid) {
            return false;
        }

        // Create a new actor and start it
        let addr = DeckActor::new(msg.devid, msg.hub.clone(), msg.deck).start();
        self.connected_devices.insert(msg.devid, addr);

        true
    }
}

impl Handler<Broadcast> for DeckHub {
    type Result = ();

    fn handle(&mut self, msg: Broadcast, _ctx: &mut Self::Context) -> Self::Result {

        for addr in self.connected_devices.values() {
            // send the message to the actor
            addr.do_send(msg.0.clone());
        }
    }
}


impl Handler<ButtonChange> for DeckHub {
    type Result = usize;

    fn handle(&mut self, msg: ButtonChange, _ctx: &mut Self::Context) -> Self::Result {
        log::info!(
            "[DeckHub]: received ButtonChange({}, {:?})",
            msg.btn, msg.state
        );

        let addr = _ctx.address();
        self.state.handle_btn_press(msg, &addr);

        1
    }
}

impl Handler<Disconnect> for DeckHub {
    type Result = bool;

    fn handle(&mut self, msg: Disconnect, _ctx: &mut Self::Context) -> Self::Result {
        log::info!(
            "[DeckHub]: received Disconnect({})",
            msg.0
        );
        if self.connected_devices.get(&msg.0).is_some() {
            self.connected_devices.remove(&msg.0);
            return true;
        }
        false
    }
}
