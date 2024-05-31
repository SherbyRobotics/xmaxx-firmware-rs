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

const STEERING_DUTY_RANGE: RangeInclusive<u16> = 130..=250;
const STEERING_DUTY_ZERO: u16 = 190;
const STEERING_RANGE: RangeInclusive<u8> = 35..=135; // deg

/// Compute the duty cycle to achieve the desired angle.
fn angle_to_duty(angle: f32) -> u8 {
    todo!()
}

const MOTOR_DUTY_RANGE: RangeInclusive<f32> = 0.1..=0.9;
const RPM_RANGE: RangeInclusive<f32> = -4500.0..=4500.0; // RPM
const CURRENT_RANGE: RangeInclusive<f32> = -8.0..=8.0; // A

const ZERO_RPM: f32 = 412.; // analog_unit
const RPM_PER_ANALOG: f32 = 4500. / 410.; // RPM / analog_unit
const GEARING: f32 = 10.6; // 10.6 (motor) : 1 (wheel)
const WHEEL_RADIUS: f32 = 0.1; // m

/// Computes the wheel RPM from the analog reading.
fn analog_to_rpm(analog: f32) -> f32 {
    RPM_PER_ANALOG * (analog - ZERO_RPM) / GEARING
}

/// Computes the duty cycle to achieve the wheel RPM.
fn rpm_to_duty(rpm: f32) -> u8 {
    todo!()
}

fn execute<Pwm: SetDutyCycle>(
    command: Command,
    steering: &mut Pwm,
    motor_fl: &mut Pwm,
    motor_fr: &mut Pwm,
    motor_rl: &mut Pwm,
    motor_rr: &mut Pwm,
) {
    steering.set_duty_cycle_percent(angle_to_duty(command.steering));
    motor_fl.set_duty_cycle_percent(rpm_to_duty(command.fl_whl_rpm));
    motor_fr.set_duty_cycle_percent(rpm_to_duty(command.fr_whl_rpm));
    motor_rl.set_duty_cycle_percent(rpm_to_duty(command.rl_whl_rpm));
    motor_rr.set_duty_cycle_percent(rpm_to_duty(command.rr_whl_rpm));
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

    let mut command = Command::default();
    let dummy_sensors = XmaxxEvent::Sensors(Sensors {
        fl_whl_rpm: 0.0,
        fr_whl_rpm: 1.0,
        rl_whl_rpm: 2.0,
        rr_whl_rpm: 3.0,
    });

    // steering setup
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

    let mut led = pins.d13.into_output();

    loop {
        // read from serial
        match read_command(&mut read_buf, &mut serial) {
            Ok(Some(command)) => {
                //                 if is_valid(&command) {
                //                     execute(command, &mut motor_fl, &mut motor_fr, &mut motor_rl, &mut motor_rr);
                //                 } else {
                //                     write_event(&XmaxxEvent::Info(XmaxxInfo::InvalidCommand), &mut write_buf, &mut serial).expect("should work because valid message and big enough buffer");
                //                 }
            }
            Ok(None) => (),
            Err(info) => write_event(&XmaxxEvent::Info(info), &mut write_buf, &mut serial)
                .expect("should work because valid message and big enough buffer"),
        };

        steering.set_duty(190); // 130..250  mid 190
                                //led.toggle();
                                // TODO uncomment
        motor_fl.set_duty(127);
        motor_fr.set_duty(127);
        motor_rl.set_duty(127);
        motor_rr.set_duty(127);

        // write Sensor to serial
        write_event(&dummy_sensors, &mut write_buf, &mut serial);
        //         write_event(&XmaxxEvent::Info(XmaxxInfo::ReadTimeout), &mut write_buf, &mut serial);

        let fl_rpm = analog_to_rpm(speed_fl.analog_read(&mut adc).into());
        let fr_rpm = analog_to_rpm(speed_fr.analog_read(&mut adc).into());
        let rl_rpm = analog_to_rpm(speed_rl.analog_read(&mut adc).into());
        let rr_rpm = analog_to_rpm(speed_rr.analog_read(&mut adc).into());

        //         ufmt::uwriteln!(&mut serial, "{}", fr_rpm);
        //         arduino_hal::delay_ms(1000);
    }
}
