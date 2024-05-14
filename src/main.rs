#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

use arduino_hal::prelude::*;

use postcard::{to_slice_cobs, from_bytes_cobs};
use serde::{Deserialize, Serialize};

mod utils;
use utils::panic::panic;
use utils::readbuf::ReadBuf;
use utils::time::{millis_init, millis};

#[derive(Serialize, Deserialize)]
struct Sensors {
    fl_whl_spd: f32,
    fr_whl_spd: f32,
    rl_whl_spd: f32,
    rr_whl_spd: f32,
    accel_x: f32,
    accel_y: f32,
    accel_z: f32,
    gyro_x: f32,
    gyro_y: f32,
    gyro_z: f32,
}

#[derive(Serialize, Deserialize, Default)]
struct Command {
    steering: i8,
    fl_whl_spd: i16,
    fr_whl_spd: i16,
    rl_whl_spd: i16,
    rr_whl_spd: i16,
}

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    /*
     * For examples (and inspiration), head to
     *
     *     https://github.com/Rahix/avr-hal/tree/main/examples
     *
     * NOTE: Not all examples were ported to all boards!  There is a good chance though, that code
     * for a different board can be adapted for yours.  The Arduino Uno currently has the most
     * examples available.
     */

    let mut serial = arduino_hal::default_serial!(dp, pins, 57600);
    let mut read_buf = ReadBuf::<128>::new();
    let mut write_buf = [0u8; 192];
    let mut led = pins.d13.into_output();
    let mut command = Command::default();

    // necessary to make millis() work
    millis_init(dp.TC0);
    // Enable interrupts globally
    unsafe { avr_device::interrupt::enable() };

    loop {

        // read from serial
        // NOTE for this to work, the send end must send each byte individually.
        while let Ok(byte) = serial.read() {
            read_buf.push(byte).unwrap();

            if byte == 0 {
                command = from_bytes_cobs(read_buf.as_mut_slice()).unwrap();
                read_buf.reset();
            }
        }


        // write Sensor to serial
        let sensors = Sensors {
            fl_whl_spd: 0.0,
            fr_whl_spd: 1.0,
            rl_whl_spd: 2.0,
            rr_whl_spd: 3.0,
            accel_x: 0.1,
            accel_y: 0.2,
            accel_z: 0.3,
            gyro_x: 0.01,
            gyro_y: 0.02,
            gyro_z: 0.03,
        };
        if let Ok(msg) = to_slice_cobs(&sensors, &mut write_buf); {
            for b in msg {
                nb::block!(serial.write(*b));
            }
        }

        ufmt::uwriteln!(&mut serial, "{}", millis());
        arduino_hal::delay_ms(1000);
    }
}
