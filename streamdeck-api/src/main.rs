use std::sync::{Arc, Mutex};
use std::time::Duration;

use actix::Actor;
use hidapi::HidApi;
use streamdeck_interface::deckmanager::DeckManager;
use streamdeck_interface::connectionmanager::ConnectionManager;
use streamdeck_interface::hub::DeckHub;
use tokio;
use actix_rt;

#[actix_rt::main]
async fn main() {
    let api = HidApi::new().unwrap();

    let hub = DeckHub::new().start();

    let cm = ConnectionManager::new(hub.clone(), api);
    
    cm.start();
    
    loop {
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}
