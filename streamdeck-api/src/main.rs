use std::time::Duration;

use actix::Actor;
use hidapi::HidApi;
use streamdeck_interface::connectionmanager::ConnectionManager;
use streamdeck_interface::deckactor::MsgType;
use streamdeck_interface::deckstate::{DeckAction, DeckButton, DeckHandler, DeckState};
use streamdeck_interface::hub::DeckHub;

#[actix_rt::main]
async fn main() {
    dotenv::dotenv().ok();
    env_logger::init();

    let api = HidApi::new().unwrap();

    let mut state = DeckState::new();
    state.btns.insert(
        0,
        DeckButton {
            action: DeckAction::DeckMsg(MsgType::BrightnessChange(0)),
        },
    );
    state.btns.insert(
        1,
        DeckButton {
            action: DeckAction::DeckMsg(MsgType::BrightnessChange(100)),
        },
    );
    state.btns.insert(
        2,
        DeckButton {
            action: DeckAction::NextState,
        },
    );

    let mut state2 = DeckState::new();
    state2.btns.insert(
        0,
        DeckButton {
            action: DeckAction::DeckMsg(MsgType::BrightnessChange(100)),
        },
    );
    state2.btns.insert(
        1,
        DeckButton {
            action: DeckAction::DeckMsg(MsgType::BrightnessChange(0)),
        },
    );
    state2.btns.insert(
        2,
        DeckButton {
            action: DeckAction::NextState,
        },
    );

    let mut handler = DeckHandler::new();
    handler.active_state = 0;
    handler.deck_states.insert(0, state);
    handler.deck_states.insert(1, state2);

    let hub = DeckHub::new(handler).start();

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
        // let _res = hub.send(Broadcast(MsgType::Ping(10))).await;
        actix_rt::time::sleep(Duration::from_secs(5)).await;
    }
}
