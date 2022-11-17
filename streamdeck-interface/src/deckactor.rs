use std::time::Duration;

use actix::{Actor, Handler, Message};
use streamdeck::StreamDeck;

use crate::{
    hub::{DeckHub, Disconnect},
};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Message)]
#[rtype(result = "usize")]
pub struct ButtonChange {
    pub btn: usize,
    pub state: ButtonState,
}

impl ButtonChange {
    pub fn new(btn: usize, state: ButtonState) -> Self {
        Self { btn, state }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ButtonState {
    Pressed,
    Released,
}


pub struct WriterActor {
    action: Vec<MsgType>,
}

impl Actor for WriterActor {
    type Context = actix::Context<Self>;
}

#[derive(Message, Clone)]
#[rtype(result = "{}")]
pub struct WriteMsg(MsgType);


#[derive(Clone, Debug, Message)]
#[rtype(result = "()")]
pub enum MsgType {
    Ping(usize),
    BrightnessChange(u8)
}

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

impl Handler<MsgType> for DeckActor {
    type Result = ();

    fn handle(&mut self, msg: MsgType, _ctx: &mut Self::Context) -> Self::Result {
        println!("[{}]: received msg({:?})", self.devid, msg);
        self.wa.do_send(WriteMsg(msg));
    }
}

impl DeckActor {
    pub fn new(devid: u16, hub: actix::Addr<DeckHub>, mut deck: StreamDeck) -> Self {
        let h2 = hub.clone();
        let wa = WriterActor { action: vec![] }.start();
        let wa2 = wa.clone();
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

                // Check if there are any tasks sent to the deck
                if let Ok(tasks) = wa2.send(GetTasks()).await {
                    if tasks.is_empty() { continue; }

                    println!("Tasks: {:?}", tasks);
                    for t in tasks {
                        match t {
                            MsgType::Ping(i) => {
                                println!("Ping: {}", i);
                            }
                            MsgType::BrightnessChange(i) => {
                                println!("Brightness: {}", i);
                                deck.set_brightness(i);
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
