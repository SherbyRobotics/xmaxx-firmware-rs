#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

// use core::ops::RangeInclusive;

use arduino_hal::prelude::*;
use arduino_hal::simple_pwm::*;

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

    // necessary to make millis() work
//     millis_init(dp.TC0);
//     unsafe { avr_device::interrupt::enable() };  // enable interrupts globally


    // communication setup
    let mut serial = arduino_hal::default_serial!(dp, pins, 57600);
    let mut read_buf = ReadBuf::<128>::new();
    let mut write_buf = [0u8; 192];

    let mut command = Command::default();
    let dummy_sensors = Sensors {
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

    // steering setup
//     const STEERING_RANGE_DEG: RangeInclusive<u16> = 35..=135;
//     const STEERING_RANGE_DUTY: RangeInclusive<u16> = 130..=250;
    let mut timer1 = Timer1Pwm::new(dp.TC1, Prescaler::Prescale64);
    let mut steering = pins.d12.into_output().into_pwm(&mut timer1);
    steering.enable();  // really important


    // motors setup TODO uncomment
    let mut enable_front = pins.d8.into_output();
    enable_front.set_high();
    let mut enable_rear = pins.d11.into_output();
    enable_rear.set_high();

    let timer3 = Timer3Pwm::new(dp.TC3, Prescaler::Prescale64);
    let timer4 = Timer4Pwm::new(dp.TC4, Prescaler::Prescale64);
    let mut motor_fl = pins.d2.into_output().into_pwm(&timer3);
    let mut motor_fr = pins.d3.into_output().into_pwm(&timer3);
    let mut motor_rl = pins.d5.into_output().into_pwm(&timer3);
    let mut motor_rr = pins.d6.into_output().into_pwm(&timer4);
    motor_fl.enable();
    motor_fr.enable();
    motor_rl.enable();
    motor_rr.enable();

    // motor speed sensor setup
    let mut adc = arduino_hal::Adc::new(dp.ADC, Default::default());
    let speed_fl = pins.a2.into_analog_input(&mut adc);  // current a0
    let speed_fr = pins.a3.into_analog_input(&mut adc);  // a1
    let speed_rl = pins.a6.into_analog_input(&mut adc);  // a4
    let speed_rr = pins.a7.into_analog_input(&mut adc);  // a5

    let mut led = pins.d13.into_output();
    led.set_low();
//     led.enable();

    loop {

        // read from serial
        // NOTE for this to work, the sending end must send each byte individually.
        // TODO uncomment
//         while let Ok(byte) = serial.read() {
//             read_buf.push(byte).unwrap();
//
//             if byte == 0 {
//                 command = from_bytes_cobs(read_buf.as_mut_slice()).unwrap();
//                 read_buf.reset();
//             }
//         }

        steering.set_duty(250);  // 130..250  mid 190
        led.toggle();
        // TODO uncomment
        motor_fl.set_duty(130);
        motor_fr.set_duty(130);
        motor_rl.set_duty(130);
        motor_rr.set_duty(130);

        // write Sensor to serial TODO uncomment
//         if let Ok(msg) = to_slice_cobs(&dummy_sensors, &mut write_buf) {
//             for b in msg {
//                 nb::block!(serial.write(*b));
//             }
//         }

       ufmt::uwriteln!(&mut serial, "{}", speed_fr.analog_read(&mut adc));
        arduino_hal::delay_ms(1000);
    }
}
