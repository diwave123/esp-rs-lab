use embedded_hal::i2c::I2c;

/// BMP280 默认 I2C 地址
pub const SENSOR_ADDR: u8 = 0x77;

/// 寄存器映射
mod reg {
    pub const ID: u8 = 0xD0;
    pub const RESET: u8 = 0xE0;
    pub const STATUS: u8 = 0xF3;
    pub const CTRL_MEAS: u8 = 0xF4;
    pub const CONFIG: u8 = 0xF5;
    pub const PRESS_MSB: u8 = 0xF7;
    pub const CALIB_START: u8 = 0x88;
}

/// 芯片 ID
pub const CHIP_ID: u8 = 0x58;

#[derive(Debug, Copy, Clone)]
pub struct SensorReading {
    pub temperature: f32,
    pub pressure: f32,
}

#[derive(Debug)]
pub enum Error<E> {
    I2c(E),
    InvalidChipId(u8),
    InvalidData,
}

#[allow(non_snake_case)]
#[derive(Default)]
struct CalibrationData {
    dig_T1: u16, dig_T2: i16, dig_T3: i16,
    dig_P1: u16, dig_P2: i16, dig_P3: i16, dig_P4: i16, dig_P5: i16,
    dig_P6: i16, dig_P7: i16, dig_P8: i16, dig_P9: i16,
    t_fine: i32,
}

pub struct Bmp280<T> {
    i2c: T,
    calib: CalibrationData,
    temp_offset: f32,
    press_offset: f32,
}

/// 由于海拔计算不依赖 I2C 硬件类型，定义为独立函数或不带泛型的 impl
pub fn calculate_altitude(pressure: f32, sea_level_pa: f32) -> f32 {
    // 海拔简化公式: (1 - (P/P0)^0.190284) * 145366.45 * 0.3048
    // 线性近似：每下降 100Pa 约上升 8.5 米
    (1.0 - (pressure / sea_level_pa)) * 8500.0
}

impl<T, E> Bmp280<T>
where
    T: I2c<Error = E>,
{
    pub fn new(i2c: T) -> Self {
        Self {
            i2c,
            calib: CalibrationData::default(),
            temp_offset: 0.0,
            press_offset: 0.0,
        }
    }

    /// 设置用户自定义偏移量（校准）
    pub fn set_calibration_offsets(&mut self, t_offset: f32, p_offset: f32) {
        self.temp_offset = t_offset;
        self.press_offset = p_offset;
    }

    /// 初始化 BMP280
    pub fn init(&mut self) -> Result<(), Error<E>> {
        let mut id = [0u8; 1];
        self.i2c.write_read(SENSOR_ADDR, &[reg::ID], &mut id).map_err(Error::I2c)?;
        if id[0] != CHIP_ID {
            return Err(Error::InvalidChipId(id[0]));
        }

        let mut data = [0u8; 24];
        self.i2c.write_read(SENSOR_ADDR, &[reg::CALIB_START], &mut data).map_err(Error::I2c)?;
        
        self.calib.dig_T1 = u16::from_le_bytes([data[0], data[1]]);
        self.calib.dig_T2 = i16::from_le_bytes([data[2], data[3]]);
        self.calib.dig_T3 = i16::from_le_bytes([data[4], data[5]]);
        self.calib.dig_P1 = u16::from_le_bytes([data[6], data[7]]);
        self.calib.dig_P2 = i16::from_le_bytes([data[8], data[9]]);
        self.calib.dig_P3 = i16::from_le_bytes([data[10], data[11]]);
        self.calib.dig_P4 = i16::from_le_bytes([data[12], data[13]]);
        self.calib.dig_P5 = i16::from_le_bytes([data[14], data[15]]);
        self.calib.dig_P6 = i16::from_le_bytes([data[16], data[17]]);
        self.calib.dig_P7 = i16::from_le_bytes([data[18], data[19]]);
        self.calib.dig_P8 = i16::from_le_bytes([data[20], data[21]]);
        self.calib.dig_P9 = i16::from_le_bytes([data[22], data[23]]);

        self.i2c.write(SENSOR_ADDR, &[reg::CTRL_MEAS, 0xB7]).map_err(Error::I2c)?;
        self.i2c.write(SENSOR_ADDR, &[reg::CONFIG, 0x10]).map_err(Error::I2c)?;

        Ok(())
    }

    /// 执行测量
    pub fn measure(&mut self) -> Result<SensorReading, Error<E>> {
        let mut buffer = [0u8; 6];
        self.i2c.write_read(SENSOR_ADDR, &[reg::PRESS_MSB], &mut buffer).map_err(Error::I2c)?;

        let p_raw = ((buffer[0] as i32) << 12) | ((buffer[1] as i32) << 4) | ((buffer[2] as i32) >> 4);
        let t_raw = ((buffer[3] as i32) << 12) | ((buffer[4] as i32) << 4) | ((buffer[5] as i32) >> 4);

        if p_raw == 0 || t_raw == 0 {
            return Err(Error::InvalidData);
        }

        let temperature = self.compensate_temp(t_raw);
        let pressure = self.compensate_press(p_raw);

        Ok(SensorReading { 
            temperature: temperature + self.temp_offset, 
            pressure: pressure + self.press_offset 
        })
    }

    fn compensate_temp(&mut self, adc_t: i32) -> f32 {
        let var1 = (((adc_t >> 3) - ((self.calib.dig_T1 as i32) << 1)) * (self.calib.dig_T2 as i32)) >> 11;
        let var2 = (((((adc_t >> 4) - (self.calib.dig_T1 as i32)) * ((adc_t >> 4) - (self.calib.dig_T1 as i32))) >> 12) * (self.calib.dig_T3 as i32)) >> 14;
        self.calib.t_fine = var1 + var2;
        ((self.calib.t_fine * 5 + 128) >> 8) as f32 / 100.0
    }

    fn compensate_press(&self, adc_p: i32) -> f32 {
        let mut v1 = (self.calib.t_fine as i64) - 128000;
        let mut v2 = v1 * v1 * (self.calib.dig_P6 as i64);
        v2 = v2 + ((v1 * (self.calib.dig_P5 as i64)) << 17);
        v2 = v2 + ((self.calib.dig_P4 as i64) << 35);
        v1 = ((v1 * v1 * (self.calib.dig_P3 as i64)) >> 8) + ((v1 * (self.calib.dig_P2 as i64)) << 12);
        v1 = (((1i64 << 47) + v1) * (self.calib.dig_P1 as i64)) >> 33;

        if v1 == 0 { return 0.0; }

        let mut p: i64 = 1048576 - adc_p as i64;
        p = (((p << 31) - v2) * 3125) / v1;
        v1 = ((self.calib.dig_P9 as i64) * (p >> 13) * (p >> 13)) >> 25;
        v2 = ((self.calib.dig_P8 as i64) * p) >> 19;
        p = ((p + v1 + v2) >> 8) + ((self.calib.dig_P7 as i64) << 4);
        (p as f32) / 256.0
    }
}
