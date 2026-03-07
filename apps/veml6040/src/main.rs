#![no_std]
#![no_main]

use esp_hal::clock::CpuClock;
use esp_hal::delay::Delay;
use esp_hal::i2c::master::{Config, I2c};
use esp_hal::main;
use esp_hal::Blocking;
use log::{info, error};
use esp_bootloader_esp_idf as _;

// 必须调用该宏来生成 ESP-IDF 兼容的 App Descriptor，否则 espflash 无法烧录
esp_bootloader_esp_idf::esp_app_desc!();

/// VEML6040 传感器的默认 I2C 7位地址
const VEML6040_ADDR: u8 = 0x10;

/// VEML6040 积分时间枚举
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IntegrationTime {
    _40ms = 0x00,
    _80ms = 0x10,
    _160ms = 0x20,
    _320ms = 0x30,
    _640ms = 0x40,
}

impl IntegrationTime {
    /// 获取下一个更短的积分时间（降低灵敏度，增大量程）
    pub fn shorter(&self) -> Option<Self> {
        match self {
            Self::_40ms => None,
            Self::_80ms => Some(Self::_40ms),
            Self::_160ms => Some(Self::_80ms),
            Self::_320ms => Some(Self::_160ms),
            Self::_640ms => Some(Self::_320ms),
        }
    }

    /// 获取下一个更长的积分时间（增加灵敏度，减小量程）
    pub fn longer(&self) -> Option<Self> {
        match self {
            Self::_40ms => Some(Self::_80ms),
            Self::_80ms => Some(Self::_160ms),
            Self::_160ms => Some(Self::_320ms),
            Self::_320ms => Some(Self::_640ms),
            Self::_640ms => None,
        }
    }

    /// 转换为可读的字符串（毫秒）
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::_40ms => "40ms",
            Self::_80ms => "80ms",
            Self::_160ms => "160ms",
            Self::_320ms => "320ms",
            Self::_640ms => "640ms",
        }
    }
}

/// 存储传感器的原始颜色测量数据
#[derive(Debug)]
pub struct Measurement {
    pub red: u16,
    pub green: u16,
    pub blue: u16,
    pub white: u16,
}

/// VEML6040 驱动结构体
pub struct Veml6040<'a> {
    i2c: I2c<'a, Blocking>,
    pub integration_time: IntegrationTime,
}

impl<'a> Veml6040<'a> {
    /// 创建一个新的 VEML6040 驱动实例
    pub fn new(i2c: I2c<'a, Blocking>) -> Self {
        Self { 
            i2c,
            integration_time: IntegrationTime::_160ms, // 默认中等
        }
    }

    /// 初始化传感器配置
    pub fn init(&mut self) -> Result<(), esp_hal::i2c::master::Error> {
        self.set_integration_time(self.integration_time)
    }

    /// 设置积分时间
    pub fn set_integration_time(&mut self, it: IntegrationTime) -> Result<(), esp_hal::i2c::master::Error> {
        // 寄存器 00h 配置：Bit 6:4 为 IT，Bit 0 为 SD（0 表示正常工作）
        let config = it as u16;
        self.write_register(0x00, config)?;
        self.integration_time = it;
        Ok(())
    }

    /// 向指定的 8位寄存器写入 16位数据（小端序）
    fn write_register(&mut self, reg: u8, value: u16) -> Result<(), esp_hal::i2c::master::Error> {
        let bytes = value.to_le_bytes();
        self.i2c.write(VEML6040_ADDR, &[reg, bytes[0], bytes[1]])
    }

    /// 从指定的 8位寄存器读取 16位数据（小端序）
    fn read_register(&mut self, reg: u8) -> Result<u16, esp_hal::i2c::master::Error> {
        let mut buffer = [0u8; 2];
        self.i2c.write_read(VEML6040_ADDR, &[reg], &mut buffer)?;
        Ok(u16::from_le_bytes(buffer))
    }

    /// 读取所有颜色通道的数据
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
    // 1. 初始化系统配置：设置 CPU 时钟为最大（240MHz）
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    // 2. 初始化日志：通过串口打印信息
    esp_println::logger::init_logger(log::LevelFilter::Info);

    let delay = Delay::new();

    info!("Initializing I2C...");

    // 3. 配置 I2C 控制器
    // 默认引脚：IO1 为 SDA，IO0 为 SCL
    let i2c = I2c::new(peripherals.I2C0, Config::default())
        .expect("Failed to initialize I2c wrapper")
        .with_sda(peripherals.GPIO1)
        .with_scl(peripherals.GPIO0);

    info!("Initializing VEML6040 with Auto-Ranging...");
    let mut sensor = Veml6040::new(i2c);

    // 执行传感器初始化（配置曝光时间等）
    if let Err(e) = sensor.init() {
        error!("Failed to initialize sensor: {:?}", e);
    }

    // 等待传感器准备就绪
    delay.delay_millis(200);

    loop {
        // 4. 定时读取并处理数据
        match sensor.read_all_channels() {
            Ok(m) => {
                info!("[IT: {}] R: {}, G: {}, B: {}, W: {}", 
                    sensor.integration_time.as_str(), 
                    m.red, m.green, m.blue, m.white
                );

                // 自适应逻辑 (Auto-Ranging)
                if m.white > 60000 {
                    // 太亮了，尝试缩短积分时间
                    if let Some(shorter_it) = sensor.integration_time.shorter() {
                        info!("Too bright! Switching IT to {}", shorter_it.as_str());
                        let _ = sensor.set_integration_time(shorter_it);
                        delay.delay_millis(100); // 等待传感器稳定
                    }
                } else if m.white < 2000 {
                    // 太暗了，尝试增加积分时间
                    if let Some(longer_it) = sensor.integration_time.longer() {
                        info!("Too dim! Switching IT to {}", longer_it.as_str());
                        let _ = sensor.set_integration_time(longer_it);
                        delay.delay_millis(100); // 等待传感器稳定
                    }
                }
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
