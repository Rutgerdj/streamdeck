use actix::{Actor, Message, Handler};



// Main struct that handles communication with the StreamDeck.
pub struct DeckActor {
  devid: u16,
}


#[derive(Message)]
#[rtype(result = "usize")]
struct Ping(usize);

impl Actor for DeckActor {
  type Context = actix::Context<Self>;
}

impl Handler<Ping> for DeckActor {
  type Result = usize;

  fn handle(&mut self, msg: Ping, _ctx: &mut Self::Context) -> Self::Result {
    println!("[{}]: received Ping({})", self.devid, msg.0);
    msg.0
  }
}


impl DeckActor {
  pub fn new(devid: u16) -> Self {
    DeckActor {
      devid,
    }
  }
}