use std::{collections::HashSet, time::Duration};

use actix::Actor;
use hidapi::HidApi;
use streamdeck::StreamDeck;

use crate::hub::{Connect, DeckHub};

#[allow(dead_code)]
pub struct ConnectionManager {
    hub: actix::Addr<DeckHub>,
}

impl Actor for ConnectionManager {
    type Context = actix::Context<Self>;
}

impl ConnectionManager {
    pub fn new(hub: actix::Addr<DeckHub>, mut api: HidApi) -> Self {
        let h2 = hub.clone();
        actix_rt::spawn(async move {
            let mut prev_connected = HashSet::new();

            loop {
                let mut now_connected: HashSet<u16> = HashSet::new();
                log::info!("Checking for new devices...");

                let _ = api.refresh_devices();
                for dev in api.device_list() {
                    if dev.vendor_id() != crate::info::ELGATO_VID {
                        // Skip devices that are not Elgato devices
                        continue;
                    }
                    let pid = dev.product_id();

                    if prev_connected.contains(&pid) {
                        // If the device was already connected, skip it
                        // TODO: multi (same-type) device support
                        continue;
                    }

                    if let Ok(deck) =
                        StreamDeck::connect_with_hid(&api, crate::info::ELGATO_VID, pid, None)
                    {
                        // Notify the hub that a new device has been discovered
                        h2.send(Connect::new(pid, h2.clone(), deck)).await.unwrap();
                        now_connected.insert(pid);
                    }
                }
                prev_connected = now_connected;

                // Check every 5 seconds for devices
                actix_rt::time::sleep(Duration::from_secs(5)).await;
            }
        });

        ConnectionManager { hub }
    }
}
