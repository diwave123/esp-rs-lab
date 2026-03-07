#![no_std]
#![no_main]

/*
AHT20 + BMP280 
AHT20: 0x38
BMP280: 0x76/0x77
*/

use bme280::i2c::BME280;
use embedded_aht20::Aht20;
use esp_hal::clock::CpuClock;
use esp_hal::delay::Delay;
use esp_hal::i2c::master::{Config, I2c};
use esp_hal::main;
use log::{error, info};
use esp_bootloader_esp_idf as _;
use core::cell::RefCell;
use embedded_hal_bus::i2c::RefCellDevice;

// 生成 ESP-IDF 兼容的 App Descriptor
esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_println::logger::init_logger(log::LevelFilter::Info);

    let mut main_delay = Delay::new();

    info!("Initializing I2C bus...");

    let i2c = I2c::new(peripherals.I2C0, Config::default())
        .expect("Failed to initialize I2c wrapper")
        .with_sda(peripherals.GPIO1)
        .with_scl(peripherals.GPIO0);

    // 使用 RefCell 包装 I2C 总线
    let i2c_ref_cell = RefCell::new(i2c);

    info!("Initializing AHT20...");
    let i2c_aht20 = RefCellDevice::new(&i2c_ref_cell);
    // AHT20 需要拥有一个 delay 实例。因为 Delay 是 Copy/Clone 的，我们可以直接新建一个。
    let mut aht20 = Aht20::new(i2c_aht20, 0x38, Delay::new()).expect("Failed to create AHT20");

    info!("Initializing BMP280...");
    let i2c_bmp280 = RefCellDevice::new(&i2c_ref_cell);
    let mut bmp280 = BME280::new_primary(i2c_bmp280);
    // BMP280 init 需要 &mut delay
    if let Err(e) = bmp280.init(&mut main_delay) {
        error!("Failed to initialize BMP280: {:?}", e);
    }

    loop {
        // 读取 AHT20
        // 在 0.2.0 版本中，如果 new 时传入了 delay，measure 方法可能不需要再传入
        match aht20.measure() {
            Ok(reading) => {
                info!("AHT20 -> Temp: {:?}, Hum: {:?}", 
                    reading.temperature, 
                    reading.relative_humidity
                );
            }
            Err(e) => error!("AHT20 read error: {:?}", e),
        }

        // 读取 BMP280
        match bmp280.measure(&mut main_delay) {
            Ok(measurement) => {
                info!("BMP280 -> Pressure: {:.2} Pa, Temp: {:.2} °C", 
                    measurement.pressure, 
                    measurement.temperature
                );
            }
            Err(e) => error!("BMP280 read error: {:?}", e),
        }

        main_delay.delay_millis(2000);
    }
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    error!("Panic: {:?}", info);
    loop {}
}
