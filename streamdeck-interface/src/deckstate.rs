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

    let state = self.deck_states.get_mut(&self.active_state)?;
    let btn = state.btns.get(&(press.btn as u16))?.clone();

    btn.action.execute(addr, self);

    None
  }

  pub fn new() -> Self {
    DeckHandler {
      deck_states: HashMap::new(),
      active_state: 0,
    }
  }
}

impl Default for DeckHandler {
    fn default() -> Self {
        Self::new()
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

impl Default for DeckState {
    fn default() -> Self {
        Self::new()
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
  pub fn execute(&self, addr: &Addr<DeckHub>, dh: &mut DeckHandler) {
    match &self {
        DeckAction::DeckMsg(msg) => {
            addr.do_send(Broadcast(msg.clone()));
        },
        DeckAction::NextState => {
          dh.active_state+=1 ;
          if dh.active_state > 1 {
            dh.active_state = 0;
          }
        }
        DeckAction::PrevState => {
        },
    };
  }
}