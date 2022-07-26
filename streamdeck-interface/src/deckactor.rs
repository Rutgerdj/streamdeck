use std::time::Duration;

use actix::{Actor, Handler, Message};
use streamdeck::StreamDeck;

use crate::{
    deckinterface::{ButtonChange, ButtonState},
    hub::{DeckHub, Disconnect},
};

// Main struct that handles communication with the StreamDeck.
pub struct DeckActor {
    devid: u16,
    hub: actix::Addr<DeckHub>,
}

#[derive(Message)]
#[rtype(result = "usize")]
struct Ping(usize);

impl Actor for DeckActor {
    type Context = actix::Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        println!("[DeckActor::{}] Started", self.devid);
    }

    fn stopping(&mut self, ctx: &mut Self::Context) -> actix::Running {
        println!("Stopping");
        actix::Running::Stop
    }
}

impl Handler<Ping> for DeckActor {
    type Result = usize;

    fn handle(&mut self, msg: Ping, _ctx: &mut Self::Context) -> Self::Result {
        println!("[{}]: received Ping({})", self.devid, msg.0);
        msg.0
    }
}

impl DeckActor {
    pub fn new(devid: u16, hub: actix::Addr<DeckHub>, mut deck: StreamDeck) -> Self {
        let h2 = hub.clone();
        actix_rt::spawn(async move {
            let mut prev_btn_state = vec![0; 16];
            loop {
                match deck.read_buttons(Some(Duration::from_millis(10))) {
                    Ok(btns) => {
                        let change = Self::state_change(&prev_btn_state, &btns);

                        for c in change {
                            h2.send(c).await.unwrap();
                        }
                        prev_btn_state = btns;
                    }
                    Err(streamdeck::Error::NoData) => {}
                    _ => {
                        h2.send(Disconnect(devid)).await.unwrap();
                        break;
                    }
                }
                actix_rt::time::sleep(Duration::from_nanos(10)).await;
            }
        });

        DeckActor { devid, hub }
    }

    fn state_change(prev: &Vec<u8>, curr: &Vec<u8>) -> Vec<ButtonChange> {
        let mut changes = vec![];
        for ((i, p), c) in prev.iter().enumerate().zip(curr) {
            if p != c {
                let state = if *c == 1 {
                    ButtonState::Pressed
                } else {
                    ButtonState::Released
                };

                let c = ButtonChange::new(i, state);
                changes.push(c);
            }
        }

        changes
    }
}
