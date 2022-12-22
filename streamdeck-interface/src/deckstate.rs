use std::collections::HashMap;

use actix::Addr;
use serde::{Deserialize, Serialize};

use crate::{
    deckactor::{ButtonChange, ButtonState, MsgType},
    hub::{Broadcast, DeckHub},
};

pub struct DeckHandler {
    pub deck_states: HashMap<u32, DeckState>,
    pub active_state: u32,
}

impl DeckHandler {
    pub fn handle_btn_press(&mut self, press: ButtonChange, addr: &Addr<DeckHub>) -> Option<bool> {
        if let ButtonState::Released = press.state {
            return None;
        }

        let state = self.deck_states.get_mut(&self.active_state)?;
        let btn = state.btns.get(&(press.btn as u16))?.clone();

        btn.action.execute(addr, self);

        None
    }

    pub fn new(deck_states: HashMap<u32, DeckState>) -> Self {
        DeckHandler {
            deck_states,
            active_state: 0,
        }
    }

    pub fn save(&self) {
        if let Ok(s) = serde_json::to_string_pretty(&self.deck_states) {
            // Write to file
            let _ = std::fs::write("deck_states.json", s);
        }
    }

    pub fn load() -> Self {
        if let Ok(s) = std::fs::read_to_string("deck_states.json") {
            let deck_states: HashMap<u32, DeckState> = serde_json::from_str(&s).unwrap();
            Self::new(deck_states)
        } else {
            Self::default()
        }
    }
}

impl Default for DeckHandler {
    fn default() -> Self {
        Self::new(HashMap::new())
    }
}

#[derive(Deserialize, Serialize)]
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

#[derive(Clone, Deserialize, Serialize)]
pub struct DeckButton {
    pub action: DeckAction,
}

#[derive(Clone, Deserialize, Serialize)]
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
            }
            DeckAction::NextState => {
                dh.active_state += 1;
                if dh.active_state > 1 {
                    dh.active_state = 0;
                }
            }
            DeckAction::PrevState => {}
        };
    }
}
