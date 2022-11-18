use std::collections::HashMap;

use actix::Addr;

use crate::{hub::{DeckHub, Broadcast}, deckactor::{ButtonChange, MsgType, ButtonState}};

pub struct DeckHandler {
  pub deck_states: HashMap<u32, DeckState>,
  pub active_state: u32,
}

impl DeckHandler {
  pub fn handle_btn_press(&mut self, press: ButtonChange, addr: &Addr<DeckHub>) -> Option<bool> {
    if let ButtonState::Pressed = press.state{
        return None;
    }
    println!("Active state = {}", self.active_state);

    let state = self.deck_states.get(&self.active_state)?;
    let btn = state.btns.get(&(press.btn as u16))?;

    btn.action.execute( &addr, 1)(self);

    println!("Active state = {}", self.active_state);
    None
  }

  pub fn new() -> Self {
    DeckHandler {
      deck_states: HashMap::new(),
      active_state: 0,
    }
  }
}

pub struct DeckState {
  pub btns: HashMap<u16, DeckButton>, 
}

impl DeckState {
  pub fn new() -> Self {
    DeckState {
      btns: HashMap::new(),
    }
  }
}

#[derive(Clone)]
pub struct DeckButton {
  pub action: DeckAction,
}

#[derive(Clone)]
pub enum DeckAction {
  NextState,
  PrevState,
  DeckMsg(MsgType),
}

impl DeckAction {
  pub fn execute(&self, addr: &Addr<DeckHub>, v: u32) -> Box<dyn FnMut(&mut DeckHandler) -> ()> {
    let mut f = Box::new(|dh: &mut DeckHandler| {});

    match &self {
        DeckAction::DeckMsg(msg) => {
            let _res = addr.do_send(Broadcast(msg.clone()));
        },
        DeckAction::NextState => {
            return Box::new(|dh: &mut DeckHandler| {
                dh.active_state += 1;
                if dh.active_state > 1 {
                    dh.active_state = 0;
                }
            });
        }
        DeckAction::PrevState => {
        },
    };

    f
  }
}