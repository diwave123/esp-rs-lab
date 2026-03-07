#![no_std]
#![no_main]

use esp_hal::clock::CpuClock;
use esp_hal::delay::Delay;
use esp_hal::main;
use esp_hal::analog::adc::{Adc, AdcConfig, Attenuation};
use log::info;

esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_println::logger::init_logger(log::LevelFilter::Info);

    let delay = Delay::new();

    info!("Initializing Soil Moisture Sensor on ADC (GPIO3)...");

    let mut adc1_config = AdcConfig::new();
    let mut adc_pin = adc1_config.enable_pin(peripherals.GPIO3, Attenuation::_11dB);
    let mut adc1 = Adc::new(peripherals.ADC1, adc1_config);

    loop {
        // Read the analog value (12-bit max on ESP32-S3 default: 0-4095)
        let pin_value: u16 = nb::block!(adc1.read_oneshot(&mut adc_pin)).unwrap();
        
        let voltage_mv = (pin_value as u32 * 3300) / 4095;
        info!("ADC Value: {} | Approx Voltage: {} mV", pin_value, voltage_mv);
        
        delay.delay_millis(1000);
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
