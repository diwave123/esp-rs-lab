#![no_std]
#![no_main]

use esp_hal::clock::CpuClock;
use esp_hal::delay::Delay;
use esp_hal::i2c::master::{Config, I2c};
use esp_hal::main;
use log::{info, error};
use veml7700::{Veml7700, IntegrationTime, Gain};

esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_println::logger::init_logger(log::LevelFilter::Info);

    let delay = Delay::new();

    info!("Initializing I2C...");

    // Default pins for I2C (can be changed based on actual board wiring)
    // using IO1 as SDA and IO0 as SCL.
    let mut i2c = I2c::new(peripherals.I2C0, Config::default())
        .expect("Failed to initialize I2c wrapper")
        .with_sda(peripherals.GPIO1)
        .with_scl(peripherals.GPIO0);

    info!("Initializing VEML7700...");
    let mut sensor = Veml7700::new(i2c);

    // Configure sensor
    let _ = sensor.set_integration_time(IntegrationTime::_100ms);
    let _ = sensor.set_gain(Gain::OneQuarter);
    let _ = sensor.enable();

    delay.delay_millis(200); // wait for sensor to be ready

    loop {
        match sensor.read_lux() {
            Ok(lux) => info!("Illuminance: {:.2} lux", lux),
            Err(_) => error!("Failed to read from sensor"),
        };
        delay.delay_millis(1000);
    }
}


#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
