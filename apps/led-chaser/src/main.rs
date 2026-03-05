#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use log::info;
use esp_hal::clock::CpuClock;
use esp_hal::delay::Delay;
use esp_hal::gpio::{Level, Output, OutputConfig};
use esp_hal::main;
use esp_println as _;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}


// This creates a default app-descriptor required by the esp-idf bootloader.
esp_bootloader_esp_idf::esp_app_desc!();

#[allow(
    clippy::large_stack_frames,
    reason = "it's not unusual to allocate larger buffers etc. in main"
)]
#[main]
fn main() -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);
    esp_println::logger::init_logger(log::LevelFilter::Info);

    // 初始化所有 LED
    let mut led1 = Output::new(peripherals.GPIO38, Level::Low, OutputConfig::default());
    let mut led2 = Output::new(peripherals.GPIO39, Level::Low, OutputConfig::default());
    let mut led3 = Output::new(peripherals.GPIO5, Level::Low, OutputConfig::default());
    let mut led4 = Output::new(peripherals.GPIO6, Level::Low, OutputConfig::default());
    let mut led5 = Output::new(peripherals.GPIO7, Level::Low, OutputConfig::default());

    // 初始化延迟
    let delay = Delay::new();

    loop {
        info!("Sequence Start");

        // 简易流水灯效果
        let leds = &mut [
            &mut led1, &mut led2, &mut led3, &mut led4, &mut led5
        ];

        for (i, led) in leds.iter_mut().enumerate() {
            info!("LED {} ON", i + 1);
            led.set_high();
            delay.delay_millis(200);
            led.set_low();
        }
    }
}
