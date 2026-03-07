本项目从 ESP32-C3 (RISC-V) 成功迁移至 ESP32-S3 (Xtensa)，并解决了 Windows 环境下特定的工具链与链接器冲突。

> [!NOTE]
> **背景提示**: 如果你是参考 Paxon Qiao 的 [ESP32-C3 教程](https://paxonqiao.com/rust-esp32c3/) 搭建的环境，请注意 ESP32-S3 (Xtensa) 的配置要求更高。C3 使用的是原生 LLVM 支持良好的 RISC-V 架构，而 S3 的 Xtensa 架构在 Windows 上需要额外的链接器配置才能稳定运行。

---

## 🚀 快速启动

### 编译并烧录

```powershell
cargo run --release -- --port COM4
```

### 核心环境配置

- **工具链**: `esp` channel (`xtensa-esp32s3-none-elf`)
- **链接器**: Xtensa GCC (处理重定位问题的关键)
- **底层驱动**: `esp-hal` v1.0.0

---

## 🛠️ 深度故障排查 (Troubleshooting)

### A. 链接阶段：重定位溢出 (Relocation out of range)

- **现象**: 编译报错 `R_XTENSA_SLOT0_OP out of range`。
- **根本原因**: `rust-lld` 对 Xtensa 字面量池（literal pools）的管理在某些复杂代码段下不够健壮。
- **对策**:
  1. 强制在 `.cargo/config.toml` 中指定 **Xtensa GCC** 绝对路径。
  2. 禁止使用 `link-arg=-Tlink.x`（由 hal 自动处理）。
  3. 添加 `-C link-arg=-nostartfiles`。

### B. 烧录阶段：应用描述符缺失 (Missing App Descriptor)

- **现象**: `espflash` 报错无法找到 `ESP-IDF App Descriptor`。
- **根本原因**: S3 的引导加载程序要求二进制文件包含描述元数据。
- **对策**:
  - 引入 `esp-bootloader-esp-idf`。
  - 在入口文件调用 `esp_bootloader_esp_idf::esp_app_desc!();`。

### C. 运行阶段：串口死寂 (Empty Serial Output)

- **现象**: 程序运行正常（如 LED 闪烁）但无任何日志。
- **对策**:
  1. **Logger 初始化**: 必须调用 `esp_println::logger::init_logger(...)`。
  2. **Feature 开关**: `esp-println` 需开启 `auto` 特性以自动适配 S3 硬件。
  3. **LogLevel**: 检查 `.cargo/config.toml` 中的 `ESP_LOG` 环境变量。

### D. 编译阶段：API 冲突与特性开关

- **现象**: 找不到 `delay` 模块或 `Output::new` 参数不匹配。
- **根本原因**: `esp-hal` v1.0.0 引入了大量 Breaking Changes。
- **对策**:
  - 开启 `esp-hal` 的 `unstable` 特性。
  - 为 `Output::new` 提供 `OutputConfig::default()`。

### E. 编译阶段：包名歧义 (Ambiguous Package Specification)

- **现象**: `error: There are multiple 'veml7700' packages in your project`.
- **根本原因**: 本地项目名称与依赖的第三方 Crates.io 库名称完全一致，导致 Cargo 无法定位。
- **对策**:
  - 将本地项目在 `Cargo.toml` 中的 `name` 改为 `veml7700-app` 或类似名称，以示区分。
  - 在工作区模式下，统一使用 `cargo run -p <unique-name>` 运行。

### F. CI/CD 阶段：GitHub Actions 编译 S3 失败

- **现象**: 标准的 `cargo build` 在云端无法识别 Xtensa 编译器。
- **根本原因**: 普通 Linux 虚拟机没有 Xtensa 交叉工具链。
- **对策**:
  - 使用 `esp-rs/xtensa-toolchain` 官方 Action。
  - 注意要在 Action 中显式安装 `binutils` 或设置 `build_std: true` 以适配 `no_std` 环境。

### G. 运行阶段：I2C 传感器数据饱和

- **现象**: 环境光传感器在强光下读数恒定为 `65535`。
- **根本原因**: 积分时间（曝光时间）设置过长导致计数器溢出。
- **对策**:
  - 降低积分时间 (Integration Time)。例如 VEML6040 从 160ms 改为 40ms，虽然灵敏度降低，但动态范围增加，能测更强的光。

---

## 📁 文件说明

- `src/bin/main.rs`: 5 路流水灯逻辑实现。
- `.cargo/config.toml`: 核心编译器与链接器配置。
- `Cargo.toml`: 针对 S3 优化的依赖管理。
- `TROUBLESHOOTING.md`: (本文档) 维护与避坑指南。
