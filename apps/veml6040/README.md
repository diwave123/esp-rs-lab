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

运行后，您应该能在串口输出中看到四通道（红、绿、蓝、白）的原始数值：

```text
INFO - Initializing I2C...
INFO - Initializing VEML6040...
INFO - R: 486, G: 480, B: 227, W: 1545
...
```

## 核心配置：积分时间 (Integration Time)

VEML6040 的测量范围由积分时间决定。若数值显示为 `65535`，说明传感器已饱和，需减小积分时间。

| 积分时间 (IT) | 配置值 (Hex) | 灵敏度 | 最大勒克斯 (约) | 适用场景      |
| :------------ | :----------- | :----- | :-------------- | :------------ |
| **40 ms**     | **0x00**     | 最低   | **16496**       | **强光/户外** |
| 80 ms         | 0x10         | 较低   | 8248            | 室内高亮      |
| 160 ms        | 0x20         | 中等   | 4124            | 普通室内      |
| 320 ms        | 0x30         | 较高   | 2062            | 较暗环境      |
| 640 ms        | 0x40         | 最高   | 1031            | 极暗环境      |

> **当前配置：** 默认为 `40ms` (0x00)，以获得最大的测量动态范围。

## 技术说明

- **本地驱动**：由于社区的 `veml6040` (v0.1.1) 库版本较旧，无法直接兼容 `esp-hal` v1.0.0 使用的 `embedded-hal` v1.0 接口，因此本项目在 `main.rs` 中直接实现了一个轻量级的 I2C 驱动。
- **App Descriptor**：项目中包含了 `build.rs` 以确保 ESP-IDF App Descriptor 被正确链接，保证 `espflash` 能够识别并烧录。

## 依赖说明

- [esp-hal](https://github.com/esp-rs/esp-hal): ESP 官方 Rust 硬件抽象层。
- [esp-println](https://github.com/esp-rs/esp-println): 用于串口日志输出。
- [esp-bootloader-esp-idf](https://github.com/esp-rs/esp-bootloader-esp-idf): 提供 App Descriptor 兼容支持。
