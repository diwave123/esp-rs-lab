# ESP-RS-Lab (ESP32-S3 Rust 实验室)

这是一个基于 ESP32-S3 的 Rust 嵌入式开发实验室，包含了多个传感器驱动及应用案例。本项目针对 ESP32-S3 Xtensa 架构进行了深度优化。

## 📥 快速开始

### 编译并烧录指定应用

在根目录使用 `-p` 参数指定项目（请根据实际串口修改 `COMx`）：

```powershell
# 1. 运行流水灯 (LED Chaser)
cargo run -p led-chaser --release -- --port COMx

# 2. 运行环境光检测 (VEML7700)
cargo run -p veml7700-app --release -- --port COMx

# 3. 运行颜色传感器 (VEML6040)
cargo run -p veml6040-app --release -- --port COMx

# 4. 运行基础 Hello World
cargo run -p hello-world --release -- --port COMx
```

---

## 📖 文档导航

- [**环境搭建全攻略**](./SETUP_S3_GUIDE.md) : 指引你如何在本地 Windows 环境配置 Xtensa 编译器及工具链。
- [**避坑指南**](./TROUBLESHOOTING.md) : 汇总了 S3 开发中常见的链接错误及解决方法。
- [**VEML6040 说明文档**](./apps/veml6040/README.md) : 详细的颜色传感器接线及量程配置说明。

---

## 🔧 项目结构

- **apps/** :
  - `led-chaser`: 基础 GPIO 控制示例。
  - `veml7700-app`: 环境光检测。
  - `veml6040-app`: RGBW 四通道颜色检测（支持自适应量程）。
- **.github/workflows/rust.yml** : 适配 ESP32-S3 的 GitHub Actions 自动构建脚本。

## 🛠️ 技术栈

- **HAL**: `esp-hal v1.0.0`
- **Architecture**: Xtensa (ESP32-S3)
- **Linker**: Xtensa GCC (解决 Windows 下 `rust-lld` 对 Xtensa 支持不佳的问题)
- **CI**: GitHub Actions + `esp-rs/xtensa-toolchain`
