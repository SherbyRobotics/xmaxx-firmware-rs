#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

use core::ops::RangeInclusive;

use arduino_hal::prelude::*;
use arduino_hal::simple_pwm::*;
use embedded_hal::pwm::SetDutyCycle;
use embedded_hal_v0::serial::{Read, Write};

mod utils;
use utils::readbuf::ReadBuf;
use utils::time::{millis, millis_init};

use xmaxx_messages::*;

/// Read a command from serial.
///
/// **Note:** for this function to work, the sending end must send each byte individually.
fn read_command<const N: usize>(
    read_buf: &mut ReadBuf<{ N }>,
    serial: &mut impl Read<u8>,
) -> Result<Option<Command>, XmaxxInfo> {
    while let Ok(byte) = serial.read() {
        read_buf.push(byte).or_else(|_| {
            read_buf.reset();
            Err(XmaxxInfo::ReadBufferOverflow)
        })?;

        if byte == 0 {
            // TODO reset buffer on deserialization error or will fail forever after
            let command: Command = deserialize(read_buf.as_mut_slice())
                .or_else(|_| Err(XmaxxInfo::DeserializationError))?;
            read_buf.reset();
            return Ok(Some(command));
        }
    }

    Ok(None)
}

/// Write an event to serial.
fn write_event(
    event: &XmaxxEvent,
    write_buf: &mut [u8],
    serial: &mut impl Write<u8>,
) -> Result<(), XmaxxInfo> {
    let msg = serialize(event, write_buf).or_else(|_| Err(XmaxxInfo::SerializationError))?;
    for b in msg {
        let _ = nb::block!(serial.write(*b)); // should be infallible, cannot .expect() because some trait is not implemented
    }
    Ok(())
}

const DUTY_CYCLE_DENOM: u16 = 10_000;

const STEERING_DUTY_NUM_RANGE: RangeInclusive<i16> = 5098..=9803; // 130..=250 / 255 * 10000
const STEERING_DUTY_NUM_ZERO: i16 = 7451; // 190 / 255 * 10000
const STEERING_RANGE: RangeInclusive<i16> = 35..=135; // deg

/// Compute the duty cycle to achieve the desired angle.
///
/// It assumes that `angle` is in the steering range of motion.
fn angle_to_duty(angle: i16) -> u16 {
    let delta_duty = STEERING_DUTY_NUM_RANGE.end() - STEERING_DUTY_NUM_RANGE.start();
    let delta_angle = STEERING_RANGE.end() - STEERING_RANGE.start();

    // (9803-5098) * 35 / (135-35) + 7451 > 0
    (delta_duty * angle / delta_angle + STEERING_DUTY_NUM_ZERO) as u16
}

const MOTOR_DUTY_NUM_RANGE: RangeInclusive<i16> = 1000..=9000; // 0.1..=0.9
const MOTOR_DUTY_NUM_ZERO: i16 = 5000;
const RPM_RANGE: RangeInclusive<i16> = -4500..=4500; // RPM

/// Computes the duty cycle to achieve the wheel RPM.
///
/// It assumes that `rpm` is in the range.
fn rpm_to_duty(rpm: i16) -> u16 {
    let delta_duty = MOTOR_DUTY_NUM_RANGE.end() - MOTOR_DUTY_NUM_RANGE.start();
    let delta_rpm = RPM_RANGE.end() - RPM_RANGE.start();

    // (9000-1000) * -4500 / (4500--4500) + 5000 > 0
    (delta_duty * rpm / delta_rpm + MOTOR_DUTY_NUM_ZERO) as u16
}

const ZERO_RPM: f32 = 412.; // analog_unit
const RPM_PER_ANALOG: f32 = 4500. / 410.; // RPM / analog_unit
const GEARING: f32 = 10.6; // 10.6 (motor) : 1 (wheel)
const WHEEL_RADIUS: f32 = 0.1; // m
const CURRENT_RANGE: RangeInclusive<f32> = -8.0..=8.0; // A

/// Computes the wheel RPM from the analog reading.
fn analog_to_rpm(analog: f32) -> f32 {
    RPM_PER_ANALOG * (analog - ZERO_RPM) / GEARING
}

fn execute(
    command: Command,
    steering: &mut impl SetDutyCycle,
    motor_fl: &mut impl SetDutyCycle,
    motor_fr: &mut impl SetDutyCycle,
    motor_rl: &mut impl SetDutyCycle,
    motor_rr: &mut impl SetDutyCycle,
) -> Result<(), XmaxxInfo> {
    if !(STEERING_RANGE.contains(&command.steering))
        || !(RPM_RANGE.contains(&command.fl_whl_rpm))
        || !(RPM_RANGE.contains(&command.fr_whl_rpm))
        || !(RPM_RANGE.contains(&command.rl_whl_rpm))
        || !(RPM_RANGE.contains(&command.rr_whl_rpm))
    {
        return Err(XmaxxInfo::InvalidCommand);
    }

    steering
        .set_duty_cycle_fraction(angle_to_duty(command.steering), DUTY_CYCLE_DENOM)
        .expect("duty cycle should not be too large");
    motor_fl
        .set_duty_cycle_fraction(rpm_to_duty(command.fl_whl_rpm), DUTY_CYCLE_DENOM)
        .expect("duty cycle should not be too large");
    motor_fr
        .set_duty_cycle_fraction(rpm_to_duty(command.fr_whl_rpm), DUTY_CYCLE_DENOM)
        .expect("duty cycle should not be too large");
    motor_rl
        .set_duty_cycle_fraction(rpm_to_duty(command.rl_whl_rpm), DUTY_CYCLE_DENOM)
        .expect("duty cycle should not be too large");
    motor_rr
        .set_duty_cycle_fraction(rpm_to_duty(command.rr_whl_rpm), DUTY_CYCLE_DENOM)
        .expect("duty cycle should not be too large");

    Ok(())
}

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    millis_init(dp.TC0);
    unsafe { avr_device::interrupt::enable() };

    // communication setup
    let mut serial = arduino_hal::default_serial!(dp, pins, 57600);
    let mut read_buf = ReadBuf::<128>::new();
    let mut write_buf = [0u8; 192];

    // TODO remove
//     let mut command = Command::default();
//     let dummy_sensors = XmaxxEvent::Sensors(Sensors {
//         fl_whl_rpm: 0.0,
//         fr_whl_rpm: 1.0,
//         rl_whl_rpm: 2.0,
//         rr_whl_rpm: 3.0,
//     });

    // steering setup
    let mut timer1 = Timer1Pwm::new(dp.TC1, Prescaler::Prescale64);
    let mut steering = pins.d12.into_output().into_pwm(&mut timer1);
    steering.enable(); // really important

    // motors setup TODO uncomment
    let mut enable_front = pins.d8.into_output();
    let mut enable_rear = pins.d11.into_output();
    enable_front.set_high();
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
    let speed_fl = pins.a3.into_analog_input(&mut adc); // current a0?
    let speed_fr = pins.a2.into_analog_input(&mut adc); // a1?
    let speed_rl = pins.a7.into_analog_input(&mut adc); // a4?
    let speed_rr = pins.a6.into_analog_input(&mut adc); // a5?

    let _led = pins.d13.into_output();

    loop {
        // read from serial
        match read_command(&mut read_buf, &mut serial) {
            Ok(Some(command)) => {
                // execute the command
                if let Err(info) = execute(
                    command,
                    &mut steering,
                    &mut motor_fl,
                    &mut motor_fr,
                    &mut motor_rl,
                    &mut motor_rr,
                ) {
                    write_event(&XmaxxEvent::Info(info), &mut write_buf, &mut serial)
                        .expect("should work because valid message and big enough buffer");
                }
            }
            // there was no command
            Ok(None) => (),
            // could not read a command
            Err(info) => write_event(&XmaxxEvent::Info(info), &mut write_buf, &mut serial)
                .expect("should work because valid message and big enough buffer"),
        };

        // write Sensors to serial
        let fl_whl_rpm = analog_to_rpm(speed_fl.analog_read(&mut adc) as f32);
        let fr_whl_rpm = analog_to_rpm(speed_fr.analog_read(&mut adc) as f32);
        let rl_whl_rpm = analog_to_rpm(speed_rl.analog_read(&mut adc) as f32);
        let rr_whl_rpm = analog_to_rpm(speed_rr.analog_read(&mut adc) as f32);
        let sensors = Sensors {
            fl_whl_rpm,
            fr_whl_rpm,
            rl_whl_rpm,
            rr_whl_rpm,
        };
        write_event(&XmaxxEvent::Sensors(sensors), &mut write_buf, &mut serial)
            .expect("should work because valid message and big enough buffer");
    }
}
