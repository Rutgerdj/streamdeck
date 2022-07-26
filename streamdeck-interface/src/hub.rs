use std::collections::{HashSet, HashMap};

use actix::{Actor, Message, Handler};
use streamdeck::StreamDeck;

use crate::{deckactor::DeckActor, deckinterface::ButtonChange};


#[derive(Message)]
#[rtype(result = "bool")]
pub struct Disconnect(pub u16);

#[derive(Message)]
#[rtype(result = "bool")]
pub struct Connect {
  devid: u16,
  hub: actix::Addr<DeckHub>,
  deck: StreamDeck
}

impl Connect {
  pub fn new(devid: u16, hub: actix::Addr<DeckHub>, deck: StreamDeck) -> Self {
    Connect {
      devid,
      hub,
      deck,
    }
  }
}

pub struct DeckHub {
  connected_devices: HashMap<u16, actix::Addr<DeckActor>>,
}

impl Actor for DeckHub {
  type Context = actix::Context<Self>;
}

impl DeckHub {
  pub fn new() -> Self {
    DeckHub {
      connected_devices: HashMap::new(),
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

impl Handler<ButtonChange> for DeckHub {
  type Result = usize;

  fn handle(&mut self, msg: ButtonChange, _ctx: &mut Self::Context) -> Self::Result {
    println!("[DeckHub]: received ButtonChange({}, {:?})", msg.btn, msg.state);

    1
  }
}

impl Handler<Disconnect> for DeckHub {
  type Result = bool;

  fn handle(&mut self, msg: Disconnect, ctx: &mut Self::Context) -> Self::Result {
    if let Some(_) = self.connected_devices.get(&msg.0) {
      self.connected_devices.remove(&msg.0);
      return true;      
    }
    false
  } 

}