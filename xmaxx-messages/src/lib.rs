#![no_std]

use postcard::{from_bytes_cobs, to_slice_cobs};
use serde::{Deserialize, Serialize};

/// Information sent by the firmware.
#[derive(Serialize, Deserialize, Debug)]
pub enum XmaxxEvent {
    Sensors(Sensors),
    Info(XmaxxInfo),
}

/// Sensor readings.
#[derive(Serialize, Deserialize, Debug)]
pub struct Sensors {
    /// Front left wheel RPM.
    pub fl_whl_rpm: f32,
    /// Front right wheel RPM.
    pub fr_whl_rpm: f32,
    /// Rear left wheel RPM.
    pub rl_whl_rpm: f32,
    /// Rear right wheel RPM.
    pub rr_whl_rpm: f32,
}

/// Possible errors in the firmware.
#[derive(Serialize, Deserialize, Debug)]
pub enum XmaxxInfo {
    SerializationError,
    DeserializationError,
    ReadBufferOverflow,
    ReadTimeout,
}

/// Command sent to the firmware.
#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Command {
    steering: f32,
    fl_whl_rpm: f32,
    fr_whl_rpm: f32,
    rl_whl_rpm: f32,
    rr_whl_rpm: f32,
}

impl Command {
    /// Returns a new valid command.
    pub fn new(
        steering: f32,
        fl_whl_rpm: f32,
        fr_whl_rpm: f32,
        rl_whl_rpm: f32,
        rr_whl_rpm: f32,
    ) -> Self {
        Self {
            steering,
            fl_whl_rpm,
            fr_whl_rpm,
            rl_whl_rpm,
            rr_whl_rpm,
        }
    }
}

/// Serializes the message.
///
/// This function allows to alter the serial format without having to rewrite
/// the caller site.
pub fn serialize<'a, 'b, M>(
    message: &'b M,
    buffer: &'a mut [u8],
) -> Result<&'a mut [u8], postcard::Error>
where
    M: Serialize,
{
    to_slice_cobs(message, buffer)
}

/// Deserializes the message.
///
/// This function allows to alter the serial format without having to rewrite
/// the caller site.
pub fn deserialize<'a, M>(buffer: &'a mut [u8]) -> Result<M, postcard::Error>
where
    M: Deserialize<'a>,
{
    from_bytes_cobs(buffer)
}
