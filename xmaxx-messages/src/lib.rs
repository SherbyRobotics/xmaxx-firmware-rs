#![no_std]

use postcard::{from_bytes_cobs, to_slice_cobs};
use serde::{Deserialize, Serialize};

/// Information sent by the firmware.
#[derive(Serialize, Deserialize, Debug)]
pub enum XmaxxEvent {
    Sensors(Sensors),
    Info(XmaxxInfo),
}

impl XmaxxEvent {
    pub const MAX_SERIAL_SIZE: usize = core::mem::size_of::<XmaxxEvent>() + core::mem::size_of::<XmaxxEvent>() / 8 + 1;
}

/// Sensor readings.
#[derive(Serialize, Deserialize, Debug)]
pub struct Sensors {
    /// Front left wheel RPM.
    pub fl_whl_rpm: i32,
    /// Front right wheel RPM.
    pub fr_whl_rpm: i32,
    /// Rear left wheel RPM.
    pub rl_whl_rpm: i32,
    /// Rear right wheel RPM.
    pub rr_whl_rpm: i32,
}

/// Information about what it happening in the firmware.
#[derive(Serialize, Deserialize, Debug)]
pub enum XmaxxInfo {
    SerializationError,
    DeserializationError,
    ReadBufferOverflow,
    ReadTimeout,
    FirmwarePanic,
    InvalidCommand,
    CommandReceived,
    NoCommandReceived,
}

/// Command sent to the firmware.
#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Command {
    /// Angle of the steering (90 deg -> straight).
    pub steering: i32,
    /// Front left wheel RPM.
    pub fl_whl_rpm: i32,
    /// Front right wheel RPM.
    pub fr_whl_rpm: i32,
    /// Rear left wheel RPM.
    pub rl_whl_rpm: i32,
    /// Rear right wheel RPM.
    pub rr_whl_rpm: i32,
}

impl Command {
    pub const MAX_SERIAL_SIZE: usize = core::mem::size_of::<Command>() + core::mem::size_of::<Command>() / 8 + 1;

}

/// Serializes the message.
pub fn serialize<'a, 'b, M>(
    message: &'b M,
    buffer: &'a mut [u8],
) -> Result<&'a mut [u8], postcard::Error>
where
    M: Serialize,
{
    // This function allows to alter the serial format without having to rewrite
    // the caller site.
    to_slice_cobs(message, buffer)
}

/// Deserializes the message.
pub fn deserialize<'a, M>(buffer: &'a mut [u8]) -> Result<M, postcard::Error>
where
    M: Deserialize<'a>,
{
    // This function allows to alter the serial format without having to rewrite
    // the caller site.
    from_bytes_cobs(buffer)
}
