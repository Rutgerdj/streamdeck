use std::{error::Error, time::Duration};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ButtonChange {
    pub btn: usize,
    pub state: ButtonState,
}

impl ButtonChange {
    pub fn new(btn: usize, state: ButtonState) -> Self {
        Self { btn, state }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ButtonState {
    Pressed,
    Released,
}

use streamdeck::StreamDeck;
use tokio::sync::broadcast;

pub struct DeckInterface {
    pub pid: u16,
}

impl DeckInterface {
    pub fn new(
        pid: u16,
        mut deck: StreamDeck,
        tx: broadcast::Sender<u32>,
        mut rx: broadcast::Receiver<u32>,
    ) -> Result<DeckInterface, Box<dyn Error>> {
        tokio::spawn(async move {
            let mut prev_btn_state = vec![0; 16];

            let tx = tx.clone();
            loop {
                // Get button press;
                match deck.read_buttons(Some(Duration::from_millis(10))) {
                    Ok(btns) => {
                        println!("btn press");
                        let change = Self::state_change(&prev_btn_state, &btns);
                        prev_btn_state = btns;
                        println!("{:?}", change);

                        let _ = tx.send(1);
                    }
                    Err(streamdeck::Error::NoData) => {}
                    _ => {
                        tx.send(2).unwrap();
                        break;
                    }
                }

                // Get message;
                if let Ok(msg) = rx.try_recv() {
                    println!("Device got msg: {}", msg);
                }

                tokio::time::sleep(Duration::from_nanos(1)).await;
            }
            println!("Terminating thread...");
        });

        let interface = DeckInterface { pid };
        Ok(interface)
    }

    fn state_change(prev: &Vec<u8>, curr: &Vec<u8>) -> Vec<ButtonChange> {
        let mut changes = vec![];
        for ((i, p), c) in prev.iter().enumerate().zip(curr) {
            if p != c {
                let state = if *c == 1 {
                    ButtonState::Pressed
                } else {
                    ButtonState::Released
                };

                let c = ButtonChange::new(i, state);
                changes.push(c);
            }
        }

        changes
    }
}
