use esp_hal::delay::Delay;
use embedded_hal::i2c::I2c;

/// AHT20 默认 I2C 地址
pub const SENSOR_ADDR: u8 = 0x38;

/// 命令集
mod cmd {
    pub const INITIALIZE: [u8; 3] = [0xBE, 0x08, 0x00];
    pub const MEASURE: [u8; 3] = [0xAC, 0x33, 0x00];
    pub const STATUS: u8 = 0x71;
    pub const SOFT_RESET: u8 = 0xBA;
}

/// 状态位掩码
mod status {
    pub const BUSY: u8 = 0x80;        // Bit 7: 1-忙, 0-空闲
    pub const CALIBRATED: u8 = 0x08;  // Bit 3: 1-已校准, 0-未校准
}

#[derive(Debug, Copy, Clone)]
pub struct SensorReading {
    pub temperature: f32,
    pub humidity: f32,
}

#[derive(Debug)]
pub enum Error<E> {
    I2c(E),
    NotCalibrated,
    Timeout,
    InvalidData,
}

pub struct Aht20<T> {
    i2c: T,
    temp_offset: f32,
    hum_offset: f32,
}

impl<T, E> Aht20<T>
where
    T: I2c<Error = E>,
{
    pub fn new(i2c: T) -> Self {
        Self { 
            i2c,
            temp_offset: 0.0,
            hum_offset: 0.0,
        }
    }

    /// 设置用户自定义偏移量（校准）
    /// t_offset: 温度修正值（例如传感器读数偏高 1.0 度，则设为 -1.0）
    /// h_offset: 湿度修正值
    pub fn set_calibration_offsets(&mut self, t_offset: f32, h_offset: f32) {
        self.temp_offset = t_offset;
        self.hum_offset = h_offset;
    }

    /// 执行软复位
    pub fn reset(&mut self, delay: &mut Delay) -> Result<(), Error<E>> {
        self.i2c.write(SENSOR_ADDR, &[cmd::SOFT_RESET]).map_err(Error::I2c)?;
        delay.delay_millis(20);
        Ok(())
    }

    /// 初始化并校准传感器
    pub fn init(&mut self, delay: &mut Delay) -> Result<(), Error<E>> {
        delay.delay_millis(100);

        let status = self.read_status_byte()?;

        if (status & status::CALIBRATED) == 0 {
            self.i2c.write(SENSOR_ADDR, &cmd::INITIALIZE).map_err(Error::I2c)?;
            delay.delay_millis(10);
            
            if (self.read_status_byte()? & status::CALIBRATED) == 0 {
                return Err(Error::NotCalibrated);
            }
        }
        Ok(())
    }

    /// 触发测量并获取结果（应用校准偏移）
    pub fn measure(&mut self, delay: &mut Delay) -> Result<SensorReading, Error<E>> {
        self.i2c.write(SENSOR_ADDR, &cmd::MEASURE).map_err(Error::I2c)?;
        delay.delay_millis(80);

        let mut buffer = [0u8; 7];
        self.i2c.read(SENSOR_ADDR, &mut buffer).map_err(Error::I2c)?;

        let hum_raw = ((buffer[1] as u32) << 12) | ((buffer[2] as u32) << 4) | (buffer[3] >> 4) as u32;
        let humidity = (hum_raw as f32) * 100.0 / 1048576.0;

        let temp_raw = (((buffer[3] as u32) & 0x0F) << 16) | ((buffer[4] as u32) << 8) | (buffer[5] as u32);
        let temperature = (temp_raw as f32) * 200.0 / 1048576.0 - 50.0;

        if humidity > 105.0 || temperature < -50.0 || temperature > 110.0 {
            return Err(Error::InvalidData);
        }

        // 应用偏移量
        Ok(SensorReading { 
            temperature: temperature + self.temp_offset, 
            humidity: (humidity + self.hum_offset).clamp(0.0, 100.0) 
        })
    }

    fn read_status_byte(&mut self) -> Result<u8, Error<E>> {
        let mut status = [0u8; 1];
        self.i2c.read(SENSOR_ADDR, &mut status).map_err(Error::I2c)?;
        Ok(status[0])
    }
}
