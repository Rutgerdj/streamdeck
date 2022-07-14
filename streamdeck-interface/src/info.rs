use streamdeck::pids;
use streamdeck::Kind;

pub const ELGATO_VID: u16 = 0x0fd9;

pub fn pid_to_kind(pid: u16) -> Option<Kind> {
    match pid {
        pids::MINI => Some(Kind::Mini),
        pids::ORIGINAL => Some(Kind::Original),
        pids::ORIGINAL_V2 => Some(Kind::OriginalV2),
        pids::MK2 => Some(Kind::Mk2),
        pids::XL => Some(Kind::Xl),
        _ => None,
    }
}
