use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
    time::Duration,
};

use crate::deckinterface::DeckInterface;
use hidapi::HidApi;
use streamdeck::StreamDeck;
use tokio::sync::broadcast::{self, Receiver, Sender};

pub struct DeckManager {
    connected_devices: Arc<Mutex<HashSet<u16>>>,
    pub to_manager: broadcast::Sender<u32>,
    pub from_manager: broadcast::Sender<u32>,
}

impl DeckManager {
    pub fn new(api: Arc<Mutex<HidApi>>) -> Self {
        println!("Initializing Deck Manager...");
        let connected_devices = Arc::new(Mutex::new(HashSet::new()));

        let (to_manager, _) = broadcast::channel(16);
        let (from_manager, _) = broadcast::channel(16);

        Self::handle_messages(to_manager.subscribe(), &connected_devices);
        Self::handle_connections(&api, &from_manager, &to_manager, &connected_devices);

        DeckManager {
            connected_devices,
            to_manager,
            from_manager,
        }
    }

    pub fn handle_messages(mut rx: Receiver<u32>, connected_devices: &Arc<Mutex<HashSet<u16>>>) {
        let cd = connected_devices.clone();
        println!("Spawning deck manager thread to handle messages...");
        tokio::spawn(async move {
            loop {
                if let Ok(x) = rx.recv().await {
                    println!("[DM] Got message: {}", x);
                    if x == 2 {
                        println!("[DM] Device error");
                        let mut cd = cd.lock().unwrap();
                        cd.clear();
                    }
                } else {
                    println!("err ret mes");
                    break;
                }
            }
        });
    }

    pub fn handle_connections(
        api: &Arc<Mutex<HidApi>>,
        from_manager: &Sender<u32>,
        to_manager: &Sender<u32>,
        connected_devices: &Arc<Mutex<HashSet<u16>>>,
    ) {
        let api = api.clone();
        let tx = to_manager.clone();

        let fm = from_manager.clone();
        let cd = connected_devices.clone();

        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(5)).await;
                println!("Checking for available devices...");
                let mut ap = api.lock().unwrap();
                let _ = ap.refresh_devices();
                for dev in ap.device_list() {
                    if dev.vendor_id() != crate::info::ELGATO_VID {
                        continue;
                    }

                    println!("Found Elgato device: {:?}", dev.product_string());
                    let pid = dev.product_id();
                    let mut c = cd.lock().unwrap();

                    if c.contains(&pid) {
                        continue;
                    }

                    if let Ok(deck) =
                        StreamDeck::connect_with_hid(&ap, crate::info::ELGATO_VID, pid, None)
                    {
                        println!("Connected device: {:?}", dev.product_string());
                        let _conn =
                            DeckInterface::new(pid, deck, tx.clone(), fm.subscribe()).unwrap();
                        c.insert(pid);
                        println!("Inserted device: {:?}", dev.product_string());
                    };
                }
            }
        });
    }

    pub fn get_connected_devices(&self) -> Vec<u16> {
        let c = self.connected_devices.lock().unwrap();
        c.iter().cloned().collect()
    }
}
