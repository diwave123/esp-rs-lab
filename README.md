## 📥 快速开始

### 编译并烧录指定应用
进入相应目录或在根目录使用 `-p` 参数：
```powershell
# 运行流水灯
cargo run -p led-chaser --release -- --port COM4

# 运行 Hello World
cargo run -p hello-world --release -- --port COM4

# 运行 VEML7700 光强检测
cargo run -p veml7700 --release -- --port COM4
```

---

## 📖 文档导航
- [**SETUP_S3_GUIDE.md**](./SETUP_S3_GUIDE.md) : **环境搭建全攻略**。
- [**TROUBLESHOOTING.md**](./TROUBLESHOOTING.md) : **避坑指南**。
- [**apps/**](./apps/) : 存放各个独立的可选应用工程。

## 🔧 技术选型
- **HAL**: `esp-hal v1.0.0` (开启 `unstable` 特性)
- **Architecture**: Xtensa (ESP32-S3)
- **Linker**: Xtensa GCC (解决 Windows 重定位问题的最优解)
- **Logger**: `log` + `esp-println`
