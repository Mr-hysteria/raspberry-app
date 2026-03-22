# 开发 SOP — Raspberry Pi 全屏时钟

目标设备：Raspberry Pi Zero 2W · 512MB · Debian 13 (Trixie) 64-bit · 800×480

---

## 1. 开发 → 编译 → 部署

### 1.1 修改代码

| 文件 | 用途 |
|------|------|
| `src/main.rs` | 时钟逻辑、时间读取、定时刷新 |
| `ui/clock.slint` | 全屏 UI 布局、字体、颜色 |
| `Cargo.toml` | 依赖版本管理 |

### 1.2 交叉编译（在 Mac 本地执行，树莓派无需安装 Rust）

```bash
source ~/.cargo/env
cd ~/Desktop/code/raspberry-app

PKG_CONFIG_ALLOW_CROSS=1 cargo build --release --target aarch64-unknown-linux-gnu
```

产物路径：`target/aarch64-unknown-linux-gnu/release/raspberry-clock`

> 首次编译约 4 分钟（下载并编译全部依赖）；后续增量编译通常在 30 秒内。

### 1.3 部署到树莓派

```bash
sshpass -p 'tang19971226' scp \
  target/aarch64-unknown-linux-gnu/release/raspberry-clock \
  raspberry@192.168.1.12:/home/raspberry/Desktop/code/raspberry-app/target/release/raspberry-clock
```

---

## 2. 在树莓派上运行

### 2.1 SSH 远程启动（Mac 执行）

```bash
sshpass -p 'tang19971226' ssh raspberry@192.168.1.12 \
  "DISPLAY=:0 XAUTHORITY=/home/raspberry/.Xauthority \
   bash /home/raspberry/Desktop/code/raspberry-app/run-clock.sh &"
```

### 2.2 树莓派本机启动

登录到树莓派桌面终端后执行：

```bash
cd /home/raspberry/Desktop/code/raspberry-app
./run-clock.sh
```

`run-clock.sh` 会自动完成：
- 关闭屏保 / DPMS 自动熄屏
- 隐藏鼠标光标（需安装 `unclutter`）
- 杀掉已有的旧进程，避免重复启动
- 全屏运行 `target/release/raspberry-clock`

---

## 3. 关闭程序

### 3.1 SSH 远程关闭（Mac 执行）

```bash
sshpass -p 'tang19971226' ssh raspberry@192.168.1.12 \
  "pkill -f raspberry-clock && echo '已关闭'"
```

### 3.2 树莓派本机关闭

```bash
pkill -f raspberry-clock
```

---

## 4. 一键开发部署脚本（可选）

把编译 + 部署合并成一条命令，方便日常使用：

```bash
source ~/.cargo/env && \
PKG_CONFIG_ALLOW_CROSS=1 cargo build --release --target aarch64-unknown-linux-gnu && \
sshpass -p 'tang19971226' scp \
  target/aarch64-unknown-linux-gnu/release/raspberry-clock \
  raspberry@192.168.1.12:/home/raspberry/Desktop/code/raspberry-app/target/release/raspberry-clock && \
sshpass -p 'tang19971226' ssh raspberry@192.168.1.12 \
  "DISPLAY=:0 XAUTHORITY=/home/raspberry/.Xauthority \
   bash /home/raspberry/Desktop/code/raspberry-app/run-clock.sh &" && \
echo "✅ 部署并启动完成"
```

---

## 5. 环境依赖备忘

| 工具 | 版本 | 安装方式 |
|------|------|---------|
| Rust stable | 1.94+ | `curl https://rsproxy.cn/rustup-init.sh \| sh` |
| aarch64 交叉工具链 | 15.2 | `brew install messense/macos-cross-toolchains/aarch64-unknown-linux-gnu` |
| sshpass | - | `brew install sshpass` |
| aarch64 编译目标 | - | `rustup target add aarch64-unknown-linux-gnu` |
