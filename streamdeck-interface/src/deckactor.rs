use std::time::Duration;

use actix::{Actor, Handler, Message};
use streamdeck::StreamDeck;

use crate::{
    deckinterface::{ButtonChange, ButtonState},
    hub::{DeckHub, Disconnect},
};

pub struct WriterActor {
    action: Vec<MsgType>,
}

impl Actor for WriterActor {
    type Context = actix::Context<Self>;
}

#[derive(Message, Clone)]
#[rtype(result = "{}")]
pub struct WriteMsg(MsgType);


#[derive(Clone, Debug)]
pub enum MsgType {
    Ping2(usize),
    BrightnessChange(u8)
}

#[derive(Message, Clone)]
#[rtype(result = "{}")]
pub struct ChangeBrightness(pub u8);


#[derive(Message, Clone)]
#[rtype(result = " Vec<MsgType>")]
pub struct GetTasks();

impl Handler<WriteMsg> for WriterActor {
    type Result = ();

    fn handle(&mut self, msg: WriteMsg, _ctx: &mut Self::Context) -> Self::Result {
        self.action.push(msg.0);
    }
}

impl Handler<GetTasks> for WriterActor {
    type Result = Vec<MsgType>;

    fn handle(&mut self, msg: GetTasks, _ctx: &mut Self::Context) -> Self::Result {
        let res = self.action.clone();
        self.action = Vec::new();
        res
    }
}

// Main struct that handles communication with the StreamDeck.
pub struct DeckActor {
    devid: u16,
    hub: actix::Addr<DeckHub>,
    wa: actix::Addr<WriterActor>,
}

#[derive(Message, Clone)]
#[rtype(result = "usize")]
pub struct Ping(pub usize);

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

impl Handler<ChangeBrightness> for DeckActor {
    type Result = ();

    fn handle(&mut self, msg: ChangeBrightness, _ctx: &mut Self::Context)  {
        println!("[{}]: received Brightness change({})", self.devid, msg.0);
        let f = self.wa.do_send(WriteMsg(MsgType::BrightnessChange(msg.0.into())));
    }
}

impl Handler<Ping> for DeckActor {
    type Result = usize;

    fn handle(&mut self, msg: Ping, _ctx: &mut Self::Context) -> Self::Result {
        println!("[{}]: received Ping({})", self.devid, msg.0);
        let f = self.wa.do_send(WriteMsg(MsgType::Ping2(msg.0.into())));
        msg.0
    }
}

impl DeckActor {
    pub fn new(devid: u16, hub: actix::Addr<DeckHub>, mut deck: StreamDeck) -> Self {
        let h2 = hub.clone();
        let wa = WriterActor { action: vec![] }.start();
        let wa2 = wa.clone();
        actix_rt::spawn(async move {
            let mut prev_btn_state = vec![0; 16];
            let mut wa = wa2;
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

                if let Ok(tasks) = wa.send(GetTasks()).await {
                    if tasks.len() > 0 {
                        println!("Tasks: {:?}", tasks);
                        for t in tasks {
                            match t {
                                MsgType::Ping2(i) => {
                                    println!("Ping: {}", i);
                                }
                                MsgType::BrightnessChange(i) => {
                                    println!("Brightness: {}", i);
                                    deck.set_brightness(i);
                                }
                            }
                        }
                    }
                }
                
                actix_rt::time::sleep(Duration::from_nanos(10)).await;
            }
        });

        DeckActor { devid, hub, wa }
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
