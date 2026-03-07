#![no_std]
#![no_main]

/*
AHT20 + BMP280 高精度温湿度气压工程 (GPIO4/GPIO3)
已集成软件校准接口 (Calibration Interface)
*/

mod aht20;
mod bmp280;

use esp_hal::clock::CpuClock;
use esp_hal::delay::Delay;
use esp_hal::i2c::master::{Config, I2c};
use esp_hal::main;
use log::{error, info};
use esp_bootloader_esp_idf as _;
use core::cell::RefCell;
use embedded_hal_bus::i2c::RefCellDevice;

use crate::aht20::Aht20;
use crate::bmp280::Bmp280;

esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);
    esp_println::logger::init_logger(log::LevelFilter::Info);

    let mut delay = Delay::new();
    delay.delay_millis(1000);

    info!("Initializing I2C Bus (SDA: GPIO4, SCL: GPIO3)...");
    let i2c = I2c::new(peripherals.I2C0, Config::default())
        .expect("Failed to init I2C")
        .with_sda(peripherals.GPIO4)
        .with_scl(peripherals.GPIO3);

    let i2c_bus = RefCell::new(i2c);

    // 实例化驱动
    let mut aht = Aht20::new(RefCellDevice::new(&i2c_bus));
    let mut bmp = Bmp280::new(RefCellDevice::new(&i2c_bus));

    // ==========================================
    // 软件校准配置 (资深工程师建议)
    // ==========================================
    // 假设 AHT20 读数偏高 0.8°C，湿度偏低 2%
    aht.set_calibration_offsets(-0.8, 2.0);
    
    // 假设 BMP280 压强读数比标准值偏低 50Pa
    bmp.set_calibration_offsets(0.0, 50.0);

    info!("Sensor Initialization...");
    if let Err(e) = aht.init(&mut delay) {
        error!("AHT20 Init Error: {:?}", e);
    } else {
        info!("AHT20 Initialized with Offsets.");
    }

    if let Err(e) = bmp.init() {
        error!("BMP280 Init Error: {:?}", e);
    } else {
        info!("BMP280 Initialized with Offsets.");
    }

    loop {
        // 读取 AHT20
        match aht.measure(&mut delay) {
            Ok(data) => info!("AHT20  | Temp: {:.2} C, Humidity: {:.2} %", data.temperature, data.humidity),
            Err(e) => error!("AHT20 Error: {:?}", e),
        }

        // 读取 BMP280
        match bmp.measure() {
            Ok(data) => {
                info!("BMP280 | Temp: {:.2} C, Pressure: {:.1} Pa", data.temperature, data.pressure);
                // 使用优化后的全局函数计算海拔
                let alt = crate::bmp280::calculate_altitude(data.pressure, 101325.0);
                info!("BMP280 | Approx Altitude: {:.1} m", alt);
            }
            Err(e) => error!("BMP280 Error: {:?}", e),
        }

        delay.delay_millis(2000);
    }
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    esp_println::println!("Panic: {:?}", info);
    loop {}
}
