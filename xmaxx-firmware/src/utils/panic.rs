use arduino_hal::prelude::*;
use xmaxx_messages::*;

#[panic_handler]
pub fn panic(info: &core::panic::PanicInfo) -> ! {
    // disable interrupts - firmware has panicked so no ISRs should continue running
    avr_device::interrupt::disable();

    // get the peripherals so we can access serial and the LED.
    //
    // SAFETY: Because main() already has references to the peripherals this is an unsafe
    // operation - but because no other code can run after the panic handler was called,
    // we know it is okay.
    let dp = unsafe { arduino_hal::Peripherals::steal() };
    let pins = arduino_hal::pins!(dp);
    let mut serial = arduino_hal::default_serial!(dp, pins, 57600);

    // disable the motors
    let mut enable_front = pins.d8.into_output();
    let mut enable_rear = pins.d11.into_output();
    enable_front.set_low();
    enable_rear.set_low();

    // print out panic location
    ufmt::uwriteln!(&mut serial, "Firmware panic!\r").unwrap_infallible();
    if let Some(loc) = info.location() {
        ufmt::uwriteln!(
            &mut serial,
            "  At {}:{}:{}\r",
            loc.file(),
            loc.line(),
            loc.column(),
        )
        .unwrap_infallible();
    }

    let mut led = pins.d13.into_output();

    let mut write_buf = [0u8; 128];
    let msg = serialize(&XmaxxEvent::Info(XmaxxInfo::FirmwarePanic), &mut write_buf).unwrap();

    loop {
        // blink LED rapidly
        led.toggle();
        arduino_hal::delay_ms(100);

        // spam that the firmware panicked
        for b in &(*msg) {
            let _ = nb::block!(serial.write(*b)); // should be infallible, cannot .expect() because some trait is not implemented
        }
    }
}
