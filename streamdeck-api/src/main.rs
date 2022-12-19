use std::time::Duration;

use actix::Actor;
use hidapi::HidApi;
use streamdeck_interface::connectionmanager::ConnectionManager;
use streamdeck_interface::deckstate::DeckHandler;
use streamdeck_interface::hub::DeckHub;

#[actix_rt::main]
async fn main() {
    dotenv::dotenv().ok();
    env_logger::init();

    let api = HidApi::new().unwrap();

    let handler = DeckHandler::load();

    let hub = DeckHub::new(handler).start();

    let cm = ConnectionManager::new(hub.clone(), api);

    cm.start();

    loop {
        // let _res = hub.send(Broadcast(MsgType::Ping(10))).await;
        actix_rt::time::sleep(Duration::from_secs(5)).await;
    }
}
