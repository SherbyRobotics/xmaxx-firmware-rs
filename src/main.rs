#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

// use core::ops::RangeInclusive;

use arduino_hal::prelude::*;
use arduino_hal::simple_pwm::*;
use embedded_hal::serial::{Read, Write};

use postcard::{from_bytes_cobs, to_slice_cobs};

mod utils;
use utils::panic::panic;
use utils::readbuf::ReadBuf;
use utils::time::{millis, millis_init};

mod messages;
use messages::{Command, Info, XmaxxError};

fn read_command<const N: usize>(read_buf: &mut ReadBuf<N>, serial: impl Read<u8>) -> Command {
    todo!()
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
    let dummy_sensors = Info::Sensors {
        fl_whl_spd: 0.0,
        fr_whl_spd: 1.0,
        rl_whl_spd: 2.0,
        rr_whl_spd: 3.0,
    };

    // steering setup
    //     const STEERING_RANGE_DEG: RangeInclusive<u16> = 35..=135;
    //     const STEERING_RANGE_DUTY: RangeInclusive<u16> = 130..=250;
    let mut timer1 = Timer1Pwm::new(dp.TC1, Prescaler::Prescale64);
    let mut steering = pins.d12.into_output().into_pwm(&mut timer1);
    steering.enable(); // really important

    // motors setup TODO uncomment
    let mut enable_front = pins.d8.into_output();
    let mut enable_rear = pins.d11.into_output();
    //     enable_front.set_high();
    //     enable_rear.set_high();

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
    let speed_fl = pins.a3.into_analog_input(&mut adc); // current a0?
    let speed_fr = pins.a2.into_analog_input(&mut adc); // a1?
    let speed_rl = pins.a7.into_analog_input(&mut adc); // a4?
    let speed_rr = pins.a6.into_analog_input(&mut adc); // a5?

    const ZERO_RPM_mV: u16 = 412; // mV
    const RPM_PER_mV: u16 = 0x70D0; // RPM / mV

    let mut led = pins.d13.into_output();
    led.set_low();
    //     led.enable();

    loop {
        // read from serial
        // NOTE for this to work, the sending end must send each byte individually.
        // TODO uncomment
        while let Ok(byte) = serial.read() {
            read_buf.push(byte).unwrap();

            if byte == 0 {
                command = from_bytes_cobs(read_buf.as_mut_slice()).unwrap();
                read_buf.reset();
            }
        }

        steering.set_duty(190); // 130..250  mid 190
        led.toggle();
        // TODO uncomment
        motor_fl.set_duty(127);
        motor_fr.set_duty(127);
        motor_rl.set_duty(127);
        motor_rr.set_duty(127);

        // write Sensor to serial TODO uncomment
        if let Ok(msg) = to_slice_cobs(&dummy_sensors, &mut write_buf) {
            for b in msg {
                nb::block!(serial.write(*b));
            }
        }

        //         ufmt::uwriteln!(&mut serial, "{}", speed_fr.analog_read(&mut adc));
        arduino_hal::delay_ms(1000);
    }
}
