#![no_std]
#![no_main]

use embassy_executor::Spawner;

use esp32c3_example::{
    config::{
        initalize_logger, initialize_peripherals,
        stepper_motor::{configure_stepper, FREQUENCY},
    },
    mk_static,
};

use esp_backtrace as _;

use esp_hal::{
    ledc::{
        timer::{self, Timer as LEDC_TIMER, TimerIFace as _},
        LSGlobalClkSource, Ledc, LowSpeed,
    },
    time::RateExtU32,
    timer::systimer::SystemTimer,
};

use esp_hal_embassy::main;

#[main]
async fn main(_spawner: Spawner) {
    initalize_logger().unwrap();

    let peripherals = initialize_peripherals().unwrap();
    // Increase this to allocate more heap memory
    esp_alloc::heap_allocator!(150 * 1024);

    let systimer = SystemTimer::new(peripherals.SYSTIMER);
    esp_hal_embassy::init(systimer.alarm0);

    let mut ledc = Ledc::new(peripherals.LEDC);

    ledc.set_global_slow_clock(LSGlobalClkSource::APBClk);
    let lstimer0 = &mut *mk_static!(
        LEDC_TIMER<'static, LowSpeed>,
        ledc.timer::<LowSpeed>(timer::Number::Timer0)
    );

    lstimer0
        .configure(timer::config::Config {
            duty: timer::config::Duty::Duty14Bit,
            clock_source: timer::LSClockSource::APBClk,
            frequency: FREQUENCY.Hz(),
        })
        .unwrap();

    log::info!("Timer frequency configured");

    let _stepper = configure_stepper(
        ledc,
        lstimer0,
        peripherals.GPIO4,
        peripherals.GPIO5,
        peripherals.GPIO20,
        peripherals.GPIO21,
    )
    .unwrap();

    log::info!("Stepper motor configured");
}
