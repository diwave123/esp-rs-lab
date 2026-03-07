#![no_std]
#![no_main]

use esp_hal::clock::CpuClock;
use esp_hal::delay::Delay;
use esp_hal::i2c::master::{Config, I2c};
use esp_hal::main;
use esp_hal::Blocking;
use log::{info, error};

esp_bootloader_esp_idf::esp_app_desc!();

const VEML6040_ADDR: u8 = 0x10;

#[derive(Debug)]
pub struct Measurement {
    pub red: u16,
    pub green: u16,
    pub blue: u16,
    pub white: u16,
}

pub struct Veml6040<'a> {
    i2c: I2c<'a, Blocking>,
}

impl<'a> Veml6040<'a> {
    pub fn new(i2c: I2c<'a, Blocking>) -> Self {
        Self { i2c }
    }

    pub fn init(&mut self) -> Result<(), esp_hal::i2c::master::Error> {
        // IT = 160ms (0x20), SD=0 (normal mode)
        let config: u16 = 0x20; 
        self.write_register(0x00, config)
    }

    fn write_register(&mut self, reg: u8, value: u16) -> Result<(), esp_hal::i2c::master::Error> {
        let bytes = value.to_le_bytes();
        self.i2c.write(VEML6040_ADDR, &[reg, bytes[0], bytes[1]])
    }

    fn read_register(&mut self, reg: u8) -> Result<u16, esp_hal::i2c::master::Error> {
        let mut buffer = [0u8; 2];
        self.i2c.write_read(VEML6040_ADDR, &[reg], &mut buffer)?;
        Ok(u16::from_le_bytes(buffer))
    }

    pub fn read_all_channels(&mut self) -> Result<Measurement, esp_hal::i2c::master::Error> {
        Ok(Measurement {
            red: self.read_register(0x08)?,
            green: self.read_register(0x09)?,
            blue: self.read_register(0x0A)?,
            white: self.read_register(0x0B)?,
        })
    }
}

#[main]
fn main() -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_println::logger::init_logger(log::LevelFilter::Info);

    let delay = Delay::new();

    info!("Initializing I2C...");

    let i2c = I2c::new(peripherals.I2C0, Config::default())
        .expect("Failed to initialize I2c wrapper")
        .with_sda(peripherals.GPIO1)
        .with_scl(peripherals.GPIO0);

    info!("Initializing VEML6040...");
    let mut sensor = Veml6040::new(i2c);

    if let Err(e) = sensor.init() {
        error!("Failed to initialize sensor: {:?}", e);
    }

    delay.delay_millis(200);

    loop {
        match sensor.read_all_channels() {
            Ok(m) => {
                info!("R: {}, G: {}, B: {}, W: {}", m.red, m.green, m.blue, m.white);
            }
            Err(e) => error!("Failed to read sensor: {:?}", e),
        }
        delay.delay_millis(1000);
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
