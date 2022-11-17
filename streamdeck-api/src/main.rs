use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use actix::Actor;
use actix_rt;
use hidapi::HidApi;
use streamdeck_interface::connectionmanager::ConnectionManager;
use streamdeck_interface::deckactor::{Ping, MsgType};
// use streamdeck_interface::deckmanager::DeckManager;
use streamdeck_interface::deckstate::{self, DeckAction, DeckButton, DeckHandler, DeckState};
use streamdeck_interface::hub::{DeckHub, Broadcast};
use tokio;

#[actix_rt::main]
async fn main() {
    let api = HidApi::new().unwrap();

    let hub = DeckHub::new().start();

    // let mut btns = HashMap::new();
    // btns.insert(
    //     0,
    //     DeckButton {
    //         action: DeckAction::NextState,
    //     },
    // );

    // let mut dh = DeckHandler::new();
    // for i in 0..4 {
    //     let mut s = DeckState::new();
    //     s.btns = btns.clone();
    //     dh.deck_states.insert(i, s);
    // }
    // println!("Pressing btn");
    // dh.handle_btn_press(0);

    let cm = ConnectionManager::new(hub.clone(), api);

    cm.start();

    loop {
        let _res = hub.send(Broadcast(MsgType::Ping(10))).await;
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}
