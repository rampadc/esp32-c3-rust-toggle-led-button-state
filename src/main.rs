#![no_std]
#![no_main]

use core::cell::{Cell, RefCell};

use critical_section::Mutex;
use esp_backtrace as _;
use esp_hal::{clock::ClockControl, gpio::{Event, Gpio9, Input, PullUp}, interrupt, peripherals::{Interrupt, Peripherals}, prelude::*, riscv, Delay, IO};
use esp_println::println;
// use esp_println::println;

static BUTTON: Mutex<RefCell<Option<Gpio9<Input<PullUp>>>>> = Mutex::new(RefCell::new(None));
static TOGGLE_LED: Mutex<Cell<bool>> = Mutex::new(Cell::new(false));

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();

    let clocks = ClockControl::max(system.clock_control).freeze();
    let mut delay = Delay::new(&clocks);

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);
    let mut led = io.pins.gpio7.into_push_pull_output();

    led.set_low().unwrap();

    let mut button = io.pins.gpio9.into_pull_up_input();
    button.listen(Event::FallingEdge);

    critical_section::with(|cs| {
        BUTTON.borrow_ref_mut(cs).replace(button);
    });
    interrupt::enable(Interrupt::GPIO, interrupt::Priority::Priority3).unwrap();

    unsafe {
        riscv::interrupt::enable();
    }


    loop {
        delay.delay_ms(200_u32);

        critical_section::with(|cs| {
            let is_toggling = TOGGLE_LED.borrow(cs).get();
            if is_toggling {
                let _ = led.toggle();
            } else {
                led.set_low().unwrap();
            }
        });
    }
}

#[interrupt]
fn GPIO() {
    critical_section::with(|cs| {
        if BUTTON.borrow_ref_mut(cs).as_mut().unwrap().is_low().unwrap() {
            let is_toggling = !TOGGLE_LED.borrow(cs).get();
            println!("Button pressed, toggling: {}", is_toggling);
            TOGGLE_LED.borrow(cs).set(is_toggling);
        }
        BUTTON.borrow_ref_mut(cs).as_mut().unwrap().clear_interrupt();
    });
}