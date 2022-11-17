use actix::Message;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Message)]
#[rtype(result = "usize")]
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
