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
}

impl<'a> Veml6040<'a> {
    /// 创建一个新的 VEML6040 驱动实例
    pub fn new(i2c: I2c<'a, Blocking>) -> Self {
        Self { i2c }
    }

    /// 初始化传感器配置
    pub fn init(&mut self) -> Result<(), esp_hal::i2c::master::Error> {
        // VEML6040 积分时间 (IT) 配置表:
        // | 积分时间 (IT) | 配置值 (Hex) | 灵敏度 | 最大勒克斯 (约) |
        // |--------------|--------------|--------|----------------|
        // | 40 ms        | 0x00         | 最低   | 16496          |
        // | 80 ms        | 0x10         | 较低   | 8248           |
        // | 160 ms       | 0x20         | 中等   | 4124           |
        // | 320 ms       | 0x30         | 较高   | 2062           |
        // | 640 ms       | 0x40         | 最高   | 1031           |
        //
        // 注意：直射阳光很容易超过 10,000 lux。
        // 寄存器 00h 配置：Bit 6:4 为 IT，Bit 0 为 SD（0 表示启动性能，1 表示进入省电模式）
        let config: u16 = 0x00; // 设置为 40ms 以获得最大量程（减少强光下的饱和）
        self.write_register(0x00, config)
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
            red: self.read_register(0x08)?,   // 红色通道寄存器 08h
            green: self.read_register(0x09)?, // 绿色通道寄存器 09h
            blue: self.read_register(0x0A)?,  // 蓝色通道寄存器 0Ah
            white: self.read_register(0x0B)?, // 白色通道寄存器 0Bh
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

    info!("Initializing VEML6040...");
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
                info!("R: {}, G: {}, B: {}, W: {}", m.red, m.green, m.blue, m.white);
            }
            Err(e) => error!("Failed to read sensor: {:?}", e),
        }
        
        // 间隔 1 秒
        delay.delay_millis(1000);
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
