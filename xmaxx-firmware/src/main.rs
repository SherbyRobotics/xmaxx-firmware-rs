#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

use core::ops::RangeInclusive;

use arduino_hal::simple_pwm::*;
use embedded_hal::pwm::SetDutyCycle;
use embedded_hal_v0::serial::{Read, Write};

use xmaxx_messages::*;

mod utils;
use utils::debug::*;
use utils::readbuf::ReadBuf;
use utils::time::{init_millis, millis};

/// Read a command from serial.
///
/// **Note:** for this function to work properly, each byte must be sent slowly
/// enough for the microcontroller to read them on time.
///
fn read_command<const N: usize>(
    read_buf: &mut ReadBuf<{ N }>,
    serial: &mut impl Read<u8>,
) -> Result<Option<Command>, Log> {
    while let Ok(byte) = serial.read() {
        // reset on overflow or it will always fail
        read_buf.push(byte).or_else(|_| {
            debug!("overflow");
            read_buf.reset();
            Err(Log::ReadBufferOverflow)
        })?;

        debug!("read");
        debug!("{:?}", read_buf.as_mut_slice());

        // null char is the separator in cobs encoding
        if byte == '\0' as u8 {
            // reset buffer on deserialization error or will fail forever after
            let command: Command = deserialize(read_buf.as_mut_slice()).or_else(|_| {
                debug!("deser err");
                read_buf.reset();
                Err(Log::DeserializationError)
            })?;

            debug!("command!");
            read_buf.reset();
            return Ok(Some(command));
        }
    }

    Ok(None)
}

/// Write information to serial.
fn write_event(
    info: &Info,
    write_buf: &mut [u8],
    serial: &mut impl Write<u8>,
) -> Result<(), Log> {
    let msg = serialize(info, write_buf).or_else(|_| Err(Log::SerializationError))?;
    for b in msg {
        let _ = nb::block!(serial.write(*b)); // should be infallible, cannot .expect() because some trait is not implemented
    }
    Ok(())
}

const SCALE: i32 = 100;
const DUTY_CYCLE_DENOM: u16 = 1000;

const STEERING_DUTY_MIN: i32 = 510; // 130 / 255 * 1000
const STEERING_DUTY_ZERO: i32 = 745; // 190 / 255 * 1000
const STEERING_DUTY_MAX: i32 = 980; // 250 / 255 * 1000
const STEERING_ANGLE_MIN: i32 = 35 * SCALE; // SCALE-deg
const STEERING_ANGLE_MAX: i32 = 135 * SCALE; // SCALE-deg
const STEERING_ANGLE_RANGE: RangeInclusive<i32> = STEERING_ANGLE_MIN..=STEERING_ANGLE_MAX; // SCALE-deg

/// Compute the duty cycle to achieve the desired angle (SCALE-degrees).
///
/// It assumes that `angle` is in the steering range of motion.
fn angle_to_duty(angle: i32) -> u16 {
    let delta_duty = STEERING_DUTY_MAX - STEERING_DUTY_MIN;
    let delta_angle = STEERING_ANGLE_MAX - STEERING_ANGLE_MIN;

    // safe to cast: all positive and in range of u16
    (delta_duty * (angle - STEERING_ANGLE_MIN) / delta_angle + STEERING_DUTY_MIN) as u16
}

const MOTOR_DUTY_NUM_MIN: i32 = 100;
const MOTOR_DUTY_NUM_ZERO: i32 = 500;
const MOTOR_DUTY_NUM_MAX: i32 = 900;
const RPM_MIN: i32 = -4500 * SCALE; // SCALE-RPM
const RPM_MAX: i32 = 4500 * SCALE; // SCALE-RPM
const RPM_RANGE: RangeInclusive<i32> = RPM_MIN..=RPM_MAX; // SCALE-RPM

/// Computes the duty cycle to achieve the wheel RPM (SCALE-RPM).
///
/// It assumes that `rpm` is in the range.
fn rpm_to_duty(rpm: i32) -> u16 {
    let delta_duty = MOTOR_DUTY_NUM_MAX - MOTOR_DUTY_NUM_MIN;
    let delta_rpm = RPM_MAX - RPM_MIN;

    (delta_duty * rpm / delta_rpm + MOTOR_DUTY_NUM_ZERO) as u16
}

const ANALOG_ZERO_RPM: i32 = 412; // analog_unit
                                  // const MAX_RPM: u16 = 4500;
const ANALOG: i32 = 410; // half analog range
const GEARING_10: i32 = 106; // 10.6 (motor) : 1 (wheel)
                             // const WHEEL_RADIUS: f32 = 0.1; // m
                             // const CURRENT_RANGE: RangeInclusive<f32> = -8.0..=8.0; // A

/// Computes the wheel RPM from the analog reading.
fn analog_to_rpm(analog: i32) -> i32 {
    //     (Fxp::from_num(MAX_RPM * (analog - ANALOG_ZERO_RPM))
    //         / Fxp::from_num(GEARING)
    //         / Fxp::from_num(ANALOG))
    //     .to_num::<f32>()
    RPM_MAX / SCALE * (analog - ANALOG_ZERO_RPM) / (ANALOG * GEARING_10 / 10)
}

fn execute(
    command: Command,
    steering: &mut impl SetDutyCycle,
    motor_fl: &mut impl SetDutyCycle,
    motor_fr: &mut impl SetDutyCycle,
    motor_rl: &mut impl SetDutyCycle,
    motor_rr: &mut impl SetDutyCycle,
) -> Result<(), Log> {
    if !(STEERING_ANGLE_RANGE.contains(&command.steering))
        || !(RPM_RANGE.contains(&command.fl_whl_rpm))
        || !(RPM_RANGE.contains(&command.fr_whl_rpm))
        || !(RPM_RANGE.contains(&command.rl_whl_rpm))
        || !(RPM_RANGE.contains(&command.rr_whl_rpm))
    {
        return Err(Log::InvalidCommand);
    }

    // duty cycle should not be too large for all
    steering
        .set_duty_cycle_fraction(angle_to_duty(command.steering), DUTY_CYCLE_DENOM)
        .unwrap();
    motor_fl
        .set_duty_cycle_fraction(rpm_to_duty(command.fl_whl_rpm), DUTY_CYCLE_DENOM)
        .unwrap();
    motor_fr
        .set_duty_cycle_fraction(rpm_to_duty(command.fr_whl_rpm), DUTY_CYCLE_DENOM)
        .unwrap();
    motor_rl
        .set_duty_cycle_fraction(rpm_to_duty(command.rl_whl_rpm), DUTY_CYCLE_DENOM)
        .unwrap();
    motor_rr
        .set_duty_cycle_fraction(rpm_to_duty(command.rr_whl_rpm), DUTY_CYCLE_DENOM)
        .unwrap();

    Ok(())
}

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    let tx1 = pins.d18.into_output();
    let rx1 = pins.d19.into_floating_input();
    let debug = arduino_hal::usart::Usart::new(dp.USART1, rx1, tx1, 57600.into());
    init_debug(debug);

    init_millis(dp.TC0);

    unsafe { avr_device::interrupt::enable() };

    // communication setup
    let mut serial = arduino_hal::default_serial!(dp, pins, 57600);
    let mut read_buf = ReadBuf::<{ Command::MAX_SERIAL_SIZE }>::new();
    let mut write_buf = [0u8; Info::MAX_SERIAL_SIZE];

    // steering setup
    let mut timer1 = Timer1Pwm::new(dp.TC1, Prescaler::Prescale64);
    let mut steering = pins.d12.into_output().into_pwm(&mut timer1);
    steering.enable(); // really important

    // motors setup
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
                write_event(
                    &Info::Log(Log::CommandReceived),
                    &mut write_buf,
                    &mut serial,
                )
                .unwrap(); // should work because valid message and big enough buffer
                // execute the command
                if let Err(log) = execute(
                    command,
                    &mut steering,
                    &mut motor_fl,
                    &mut motor_fr,
                    &mut motor_rl,
                    &mut motor_rr,
                ) {
                    write_event(&Info::Log(log), &mut write_buf, &mut serial)
                        .unwrap(); // should work because valid message and big enough buffer
                }
            }
            // there was no command
            Ok(None) => write_event(
                &Info::Log(Log::NoCommandReceived),
                &mut write_buf,
                &mut serial,
            )
            .unwrap(), // should work because valid message and big enough buffer
            // could not read a command
            Err(log) => write_event(&Info::Log(log), &mut write_buf, &mut serial)
                .unwrap(), // should work because valid message and big enough buffer
        };

        // write Sensors to serial
        let fl_whl_rpm = analog_to_rpm(speed_fl.analog_read(&mut adc).into());
        let fr_whl_rpm = analog_to_rpm(speed_fr.analog_read(&mut adc).into());
        let rl_whl_rpm = analog_to_rpm(speed_rl.analog_read(&mut adc).into());
        let rr_whl_rpm = analog_to_rpm(speed_rr.analog_read(&mut adc).into());
        let sensors = Sensors {
            fl_whl_rpm,
            fr_whl_rpm,
            rl_whl_rpm,
            rr_whl_rpm,
        };
        write_event(&Info::Sensors(sensors), &mut write_buf, &mut serial)
            .unwrap(); // should work because valid message and big enough buffer
    }
}
