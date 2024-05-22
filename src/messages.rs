use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum Info {
    Sensors {
        fl_whl_spd: f32,
        fr_whl_spd: f32,
        rl_whl_spd: f32,
        rr_whl_spd: f32,
    },
    Error(XmaxxError),
}

#[derive(Serialize, Deserialize)]
pub enum XmaxxError {
    SerializationError,
    DeserializationError,
    ReadError,
    ReadBufferOverflow,
    ReadTimeout,
    WriteError,
}

#[derive(Serialize, Deserialize, Default)]
pub struct Command {
    steering: i8,
    fl_whl_spd: i16,
    fr_whl_spd: i16,
    rl_whl_spd: i16,
    rr_whl_spd: i16,
}
