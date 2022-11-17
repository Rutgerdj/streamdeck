use std::collections::HashMap;

pub struct DeckHandler {
  pub deck_states: HashMap<u32, DeckState>,
  pub active_state: u32,

}

impl DeckHandler {
  pub fn handle_btn_press(&mut self, btn_id: u16) -> Option<bool> {
    println!("Active state = {}", self.active_state);
    let state = self.deck_states.get(&self.active_state)?;
    let btn = state.btns.get(&btn_id)?;
    let f = btn.action.execute()(self, 1);
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
}

impl DeckAction {
  pub fn execute(&self) -> impl Fn(&mut DeckHandler, u32) -> i32 {
    return |x, i| {
      println!("NextState");
      x.active_state = i;
      1
    };
  }
}