use std::sync::{Arc, Mutex};
use std::time::Duration;

use hidapi::HidApi;
use streamdeck_interface::deckmanager::DeckManager;
use tokio;

#[tokio::main]
async fn main() {
    let api = Arc::new(Mutex::new(HidApi::new().unwrap()));

    let _dm = DeckManager::new(api);

    loop {
        tokio::time::sleep(Duration::from_secs(5)).await;
        println!("Getting devices..");
        let f = _dm.get_connected_devices();
        println!("Connected devices: {:?}", f);
    }
}
