//! Uses a PCF8574 connected to pins PB6 and PB7 of the STM23F3Discovery
//! board to read the pins P0-P3 and output the values to the LEDs
//! connected to P4-P7.

#![deny(unsafe_code)]
#![deny(warnings)]
#![no_std]
#![no_main]

extern crate cortex_m;
#[macro_use(entry, exception)]
extern crate cortex_m_rt as rt;
extern crate f3;
extern crate panic_semihosting;
extern crate pcf857x;

use f3::hal::delay::Delay;
use f3::hal::prelude::*;
use f3::hal::stm32f30x;
use rt::ExceptionFrame;
use f3::hal::i2c::I2c;
pub use f3::hal::stm32f30x::i2c1;
use pcf857x::{PCF8574, SlaveAddr, PinFlag};

entry!(main);

fn main() -> ! {
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = stm32f30x::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.freeze(&mut flash.acr);
    let mut delay = Delay::new(cp.SYST, clocks);

    let mut gpiob = dp.GPIOB.split(&mut rcc.ahb);
    let scl = gpiob.pb6.into_af4(&mut gpiob.moder, &mut gpiob.afrl);
    let sda = gpiob.pb7.into_af4(&mut gpiob.moder, &mut gpiob.afrl);

    let i2c = I2c::i2c1(dp.I2C1, (scl, sda), 400.khz(), clocks, &mut rcc.apb1);
    let mut expander = PCF8574::new(i2c, SlaveAddr::default());

    loop {
        // instead of havin a busy-wait loop like this one, one could use the INT output
        // of the PCF8574 which notifies of changes on the input pins (see datasheet).

        let input_mask = PinFlag::P0 | PinFlag::P1
                         | PinFlag::P2 | PinFlag::P3;
        let input = expander.get(input_mask).unwrap();
        // inputs are set to `1` (see PCF8574 datasheet).
        // The status needs to be kept so we `or` the input mask.
        expander.set(input << 4 | input_mask).unwrap();
        delay.delay_ms(20_u16);
    }
}

exception!(HardFault, hard_fault);

fn hard_fault(ef: &ExceptionFrame) -> ! {
    panic!("{:#?}", ef);
}

exception!(*, default_handler);

fn default_handler(irqn: i16) {
    panic!("Unhandled exception (IRQn = {})", irqn);
}
