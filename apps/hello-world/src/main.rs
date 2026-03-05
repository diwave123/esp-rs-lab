#![no_std]
#![no_main]

use esp_hal::clock::CpuClock;
use esp_hal::main;
use log::info;

// 必须添加应用描述符
esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let _peripherals = esp_hal::init(config);

    esp_println::logger::init_logger(log::LevelFilter::Info);

    loop {
        info!("Hello ESP32-S3 from Workspace!");
        esp_hal::delay::Delay::new().delay_millis(1000);
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
