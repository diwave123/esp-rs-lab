# ESP32-S3 Rust 环境搭建全攻略 (esp-rs-lab)

本指南参考 Paxon Qiao 的 C3 教程，并针对 ESP32-S3 (Xtensa) 架构进行了深度优化和问题修复。

---

## 1. 基础环境安装

### 安装 Rust

前往 [rust-lang.org](https://www.rust-lang.org/tools/install) 下载并安装 `rustup`。

### 安装 Espressif 专用工具链

ESP32-S3 是 Xtensa 架构，需要专门的编译器。

```powershell
# 安装工具链管理器
cargo install espup
# 执行安装 (这会安装 xtensa-esp32s3-elf-gcc 等)
espup install
```

*安装完成后，请按照提示执行生成的 `export-esp.ps1`（或重启终端）。*

### 安装核心工具

```powershell
cargo install esp-generate espflash ldproxy
```

---

## 2. 项目初始化

建议使用官方模板生成器：

```powershell
esp-generate --chip esp32s3 my-project
cd my-project
```

---

## 3. 针对 S3 的核心配置 (关键！防止报错)

由于 S3 在 Windows 上使用默认配置极易报错，请务必执行以下修改：

### 1) 修改 `.cargo/config.toml` (解决链接重定位错误)

不要使用默认的 `rust-lld`，强制使用刚才安装的 GCC：

```toml
[target.xtensa-esp32s3-none-elf]
runner = "espflash flash --monitor --chip esp32s3"
# 路径通常在 rustup 目录下，请根据实际安装位置修改
linker = "C:\\Users\\你的用户名\\.rustup\\toolchains\\esp\\xtensa-esp-elf\\bin\\xtensa-esp32s3-elf-gcc.exe"

[build]
rustflags = [
  "-C", "force-frame-pointers",
  "-C", "link-arg=-nostartfiles", # 必须添加
]
target = "xtensa-esp32s3-none-elf"
```

### 2) 修改 `Cargo.toml` (适配最新 v1.0.0)

```toml
[dependencies]
# 开启 unstable 以支持 delay 等基础模块
esp-hal = { version = "1.0.0", features = ["esp32s3", "unstable"] }
# 必须带上此依赖，否则无法启动
esp-bootloader-esp-idf = { version = "0.4.0", features = ["esp32s3"] }
esp-println = { version = "0.16.1", features = ["log-04", "esp32s3", "auto"] }
log = "0.4"
```

### 3) 修改 `src/bin/main.rs` (核心初始化)

```rust
#![no_std]
#![no_main]

// 关键 1: 添加应用描述符
esp_bootloader_esp_idf::esp_app_desc!();

#[esp_hal::main]
fn main() -> ! {
    let config = esp_hal::Config::default();
    let peripherals = esp_hal::init(config);

    // 关键 2: 初始化日志输出
    esp_println::logger::init_logger(log::LevelFilter::Info);

    loop {
        log::info!("Hello ESP32-S3!");
        esp_hal::delay::Delay::new().delay_millis(1000);
    }
}
```

---

## 4. 编译与烧录

连接开发板到电脑（假设串口为 `COMx`），运行：

```powershell
cargo run -p led-chaser --release -- --port COMx
```

---

## 💡 进阶避坑总结

- **为什么不用 rust-lld?** 在 Windows 上它对 Xtensa 的字面量地址计算有 bug，会导致烧录失败。
- **为什么要 `unstable`?** `esp-hal` v1.0.0 正在快速迭代，很多稳定功能（如 Delay）暂时放在 unstable 下。
- **烧录完没反应?** 大概率是忘了写 `esp_app_desc!()` 或没初始化 `logger`。
