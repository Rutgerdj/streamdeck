use std::{collections::HashSet, time::Duration, sync::{Mutex, Arc}};

use actix::Actor;
use hidapi::HidApi;
use streamdeck::StreamDeck;

use crate::hub::{DeckHub, Connect};


pub struct ConnectionManager {
  connected_devices: Arc<Mutex<HashSet<u16>>>,
  hub: actix::Addr<DeckHub>,
}


impl Actor for ConnectionManager {
  type Context = actix::Context<Self>;
}

impl ConnectionManager {
  pub fn new(hub: actix::Addr<DeckHub>, mut api: HidApi) -> Self {
    let connected_devices = Arc::new(Mutex::new(HashSet::new()));

    let h2 = hub.clone();
    let devs = connected_devices.clone();
    actix_rt::spawn(async move {

      loop {
        actix_rt::time::sleep(Duration::from_secs(5)).await;
        println!("[CM] Checking for new devices...");
        let _ = api.refresh_devices();
        for dev in api.device_list() {
          if dev.vendor_id() != crate::info::ELGATO_VID {
            // Skip devices that are not Elgato devices
            continue;
          }
          println!("Found device: {:?}", dev.product_string());
          let pid = dev.product_id();
          let mut devs = devs.lock().unwrap();
          if devs.contains(&pid) {
            continue;
          }
          if let Ok(deck) = StreamDeck::connect_with_hid(&api, crate::info::ELGATO_VID, pid, None) {
            h2.send(Connect::new(pid, h2.clone(), deck)).await.unwrap();
            devs.insert(pid);
          }



        }
      }
    });

    ConnectionManager {
      connected_devices,
      hub,
    }
  }
}
