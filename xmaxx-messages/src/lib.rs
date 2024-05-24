#![no_std]

use postcard::{from_bytes_cobs, to_slice_cobs};
use serde::{Deserialize, Serialize};

/// Information sent by the firmware.
#[derive(Serialize, Deserialize)]
pub enum Info {
    Sensors {
        fl_whl_rpm: f32,
        fr_whl_rpm: f32,
        rl_whl_rpm: f32,
        rr_whl_rpm: f32,
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
    steering: f32,
    fl_whl_rpm: f32,
    fr_whl_rpm: f32,
    rl_whl_rpm: f32,
    rr_whl_rpm: f32,
}

/// Serializes the message.
///
/// This function allows to alter the serial format without having to rewrite
/// the caller site.
pub fn serialize<'a, 'b, M>(
    message: &'b M,
    buffer: &'a mut [u8],
) -> Result<&'a mut [u8], XmaxxError>
where
    M: Serialize,
{
    to_slice_cobs(message, buffer).or_else(|_| Err(XmaxxError::SerializationError))
}

/// Deserializes the message.
///
/// This function allows to alter the serial format without having to rewrite
/// the caller site.
pub fn deserialize<'a, M>(buffer: &'a mut [u8]) -> Result<M, XmaxxError>
where
    M: Deserialize<'a>,
{
    from_bytes_cobs(buffer).or_else(|_| Err(XmaxxError::DeserializationError))
}
