# VEML6040 颜色传感器测试工程 (ESP32-S3)

这是一个基于 `esp-hal` 的 Rust 嵌入式工程，用于在 ESP32-S3 上通过 I2C 读取 VEML6040 RGBW 颜色传感器的原始数据。

## 硬件接线

本工程默认使用以下引脚进行 I2C 通信：

| VEML6040 | ESP32-S3  | 备注           |
| :------: | :-------: | :------------- |
| **VCC**  | **3.3V**  | 传感器电源供电 |
| **GND**  |  **GND**  | 共地           |
| **SDA**  | **GPIO1** | I2C 数据线     |
| **SCL**  | **GPIO0** | I2C 时钟线     |

> **注意：** 如果您的实际接线不同，请在 `src/main.rs` 中修改 `.with_sda()` 和 `.with_scl()` 的引脚配置。

## 快速运行

在工作区根目录 (`esp-rs-lab`) 运行以下命令编译并烧录运行（请根据实际串口修改 `COMx`）：

```bash
cargo run -p veml6040-app --release -- --port COMx
```

运行后，您应该能在串口输出中看到包含当前积分时间 (IT) 的颜色数值：

```text
INFO - Initializing I2C...
INFO - Initializing VEML6040 with Auto-Ranging...
INFO - [IT: 160ms] R: 486, G: 480, B: 227, W: 1545
...
```

## 核心功能：自适应量程 (Auto-Ranging)

本案例实现了自动调整积分时间（IT）的逻辑，以应对不同的光强环境：

- **太亮时**（White > 60,000）：自动缩短积分时间（最低至 40ms），增加测量量程。
- **太暗时**（White < 2,000）：自动增加积分时间（最高至 640ms），提高检测灵敏度。

| 积分时间 (IT) | 配置值 (Hex) | 灵敏度 | 最大勒克斯 (约) | 适用场景      |
| :------------ | :----------- | :----- | :-------------- | :------------ |
| **40 ms**     | **0x00**     | 最低   | **16496**       | **强光/户外** |
| 80 ms         | 0x10         | 较低   | 8248            | 室内高亮      |
| 160 ms        | 0x20         | 中等   | 4124            | 普通室内      |
| 320 ms        | 0x30         | 较高   | 2062            | 较暗环境      |
| 640 ms        | 0x40         | 最高   | 1031            | 极暗环境      |

## 技术文档

在 `docs/` 目录下存放了官方技术参考文件：

- [数据手册 (Datasheet)](./docs/veml6040_datasheet.pdf)
- [设计指南 (Design Guide)](./docs/designing_veml6040.pdf)

## 技术说明

- **本地驱动**：由于社区库版本较旧，本项目直接在 `main.rs` 中实现了一个支持自适应逻辑的轻量级驱动。
- **App Descriptor**：包含 `build.rs` 以确保 ESP-IDF App Descriptor 正确链接，解决烧录识别问题。

## 依赖说明

- [esp-hal](https://github.com/esp-rs/esp-hal): ESP 官方 Rust HAL。
- [esp-println](https://github.com/esp-rs/esp-println): 串口日志输出。
- [esp-bootloader-esp-idf](https://github.com/esp-rs/esp-bootloader-esp-idf): App Descriptor 支持。
