#![no_std]

use serde::{Deserialize, Serialize};

/// Information sent by the firmware.
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

/// Possible errors in the firmware.
#[derive(Serialize, Deserialize)]
pub enum XmaxxError {
    SerializationError,
    DeserializationError,
    ReadError,
    ReadBufferOverflow,
    ReadTimeout,
    WriteError,
}

/// Command sent to the firmware.
#[derive(Serialize, Deserialize, Default)]
pub struct Command {
    steering: i8,
    fl_whl_spd: i16,
    fr_whl_spd: i16,
    rl_whl_spd: i16,
    rr_whl_spd: i16,
}
