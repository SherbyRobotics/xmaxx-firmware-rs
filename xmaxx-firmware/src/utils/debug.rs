use core::cell::RefCell;

pub(crate) type DebugConsole = arduino_hal::hal::usart::Usart1<arduino_hal::DefaultClock>;
pub(crate) static DEBUG: avr_device::interrupt::Mutex<RefCell<Option<DebugConsole>>> =
    avr_device::interrupt::Mutex::new(RefCell::new(None));

#[macro_export]
macro_rules! debug_no_ln {
    ($($t:tt)*) => {
        avr_device::interrupt::free(
            |cs| {
                if let Some(debug) = DEBUG.borrow(cs).borrow_mut().as_mut() {
                    let _ = ufmt::uwrite!(debug, $($t)*);
                }
            },
        )
    };
}

#[macro_export]
macro_rules! debug {
    ($($t:tt)*) => {
        avr_device::interrupt::free(
            |cs| {
                if let Some(debug) = DEBUG.borrow(cs).borrow_mut().as_mut() {
                    let _ = ufmt::uwriteln!(debug, $($t)*);
                }
            },
        )
    };
}

pub(crate) fn init_debug(debug: DebugConsole) {
    avr_device::interrupt::free(|cs| {
        *DEBUG.borrow(cs).borrow_mut() = Some(debug);
    })
}
