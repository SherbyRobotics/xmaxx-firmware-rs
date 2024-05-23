#![no_std]

use serde::{Deserialize, Serialize};
use postcard::{from_bytes_cobs, to_slice_cobs, Error};

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
#[derive(Serialize, Deserialize, Debug)]
pub enum XmaxxError {
    SerializationError,
    DeserializationError,
    ReadBufferOverflow,
    ReadTimeout,
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


pub fn serialize<'a, 'b, M>(message: &'b M, buffer: &'a mut [u8]) -> Result<&'a mut [u8], XmaxxError> {
    todo!()
}

pub fn deserialize<M>(buffer: &mut [u8]) -> Result<M, XmaxxError> {
    todo!()
}
