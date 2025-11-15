//! Stores certain hardware configurations

pub mod stepper_motor;

use crate::errors::Result;
use esp_hal::{clock::CpuClock, peripherals::Peripherals};

/// Initialize the ESP32 logger capbilities
pub fn initalize_logger() -> Result<()> {
    esp_println::logger::init_logger(log::LevelFilter::Info);
    Ok(())
}

/// Initialize the peripherals
pub fn initialize_peripherals() -> Result<Peripherals> {
    let peripherals = esp_hal::init({
        let mut config = esp_hal::Config::default();
        config.cpu_clock = CpuClock::max();
        config
    });
    Ok(peripherals)
}

#[macro_export]
/// Makes an object static even after the start of the program.
/// When you are okay with using a nightly compiler it's better to use https://docs.rs/static_cell/2.1.0/static_cell/macro.make_static.html
macro_rules! mk_static {
    ($t:ty,$val:expr) => {{
        static STATIC_CELL: static_cell::StaticCell<$t> = static_cell::StaticCell::new();
        #[deny(unused_attributes)]
        let x = STATIC_CELL.uninit().write(($val));
        x
    }};
}
