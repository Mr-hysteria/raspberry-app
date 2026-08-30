# 开发 SOP — Raspberry Pi 每日阅读屏

目标设备：`Raspberry Pi Zero 2 W` · `512MB` · `Debian 13 (Trixie) 64-bit` · `800×480` · `X11`

## 0. 适用范围

这份 SOP 只覆盖当前每日阅读屏路线：

- `Rust + Slint` 原生应用
- 五条固定今日诗词接口
- `daily-reading.json` 双槽缓存
- 程序化背景
- X11 DPMS 夜间息屏

如果你在排查历史版本遗留问题，请直接跳到文末“历史兼容与缓存排障”。

## 1. 本地开发与构建

### 1.1 常用入口文件

- `src/main.rs`：应用入口、后台线程、状态绑定
- `src/daily_reading.rs`：阅读请求、过滤、缓存
- `src/background.rs`：程序化背景色板
- `src/domain.rs`：开始仪式与夜间窗口
- `src/display_power.rs`：DPMS 协调
- `ui/clock.slint`：`800×480` 固定布局

### 1.2 本地构建

```bash
cargo build
cargo build --release
```

### 1.3 交叉编译

```bash
PKG_CONFIG_ALLOW_CROSS=1 cargo build --release --target aarch64-unknown-linux-gnu
```

产物路径：

```text
target/aarch64-unknown-linux-gnu/release/raspberry-clock
```

## 2. 预览与视觉检查

预览链路复用生产组件，而不是单独维护一套 mock UI。

先检查示例能编译：

```bash
cargo check --example render-preview
```

再生成三张 `800×480` PNG：

```bash
./scripts/render-previews.sh ./tmp/previews
```

输出文件：

- `reading-day.png`
- `reading-focus.png`
- `reading-night.png`

这一步适合在改动 `ui/clock.slint`、`src/background.rs`、`src/main.rs` 的阅读内容绑定后执行。

## 3. 部署到树莓派

### 3.1 推荐方式

```bash
./scripts/deploy-and-run-pi.sh
```

常用覆盖参数：

- `PI_SSH_HOST=<ssh-alias> ./scripts/deploy-and-run-pi.sh`
- `PI_HOST=<device-host-or-ip> ./scripts/deploy-and-run-pi.sh`
- `PI_REMOTE_GIT_PULL=1 ./scripts/deploy-and-run-pi.sh`

建议优先使用 SSH key 或 SSH alias，不在文档中传播口令式流程。

### 3.2 远程运行前检查

登录设备后建议先确认：

```bash
whoami
pwd
uname -a
echo "$DISPLAY"
```

如果只是维护文件或执行构建，`DISPLAY` 为空是正常的。需要远程启动桌面全屏程序时，再显式提供：

```bash
DISPLAY=:0 XAUTHORITY="${HOME}/.Xauthority"
```

### 3.3 设备本机启动

在树莓派桌面终端进入仓库根目录后执行：

```bash
./run-clock.sh
```

`run-clock.sh` 会自动完成：

- 关闭屏保与自动 DPMS 超时
- 启动 `unclutter`
- 清理旧的 `raspberry-clock` 进程
- 优先运行 `target/release/raspberry-clock`
- 找不到 release 时回退到 `target/debug/raspberry-clock`

当前 Debian 13 使用的是经典版 `unclutter` 8.x，启动参数必须保持：

```bash
unclutter -idle 1 -jitter 1 -root
```

可通过下面命令确认鼠标隐藏进程仍然存活：

```bash
pgrep -af unclutter
```

## 4. 运行时缓存与离线排障

### 4.1 缓存位置

当前缓存目录固定为：

```text
~/.cache/raspberry-clock
```

当前活动缓存文件为：

```text
~/.cache/raspberry-clock/daily-reading.json
```

### 4.2 安全查看缓存

```bash
cache_dir="${HOME}/.cache/raspberry-clock"
ls -l "$cache_dir"
python -m json.tool "$cache_dir/daily-reading.json"
```

如果设备没有 `python`，可以改用：

```bash
jq . "$cache_dir/daily-reading.json"
```

### 4.3 安全清理当前缓存

只删除当前 JSON 文件，不删除整个缓存目录：

```bash
cache_dir="${HOME}/.cache/raspberry-clock"
rm -f "$cache_dir/daily-reading.json"
```

下次启动或下次刷新时，程序会回退到内置诗句，并在可联网时重新拉取内容。

### 4.4 看到旧缓存命名时的安全处理

这一步只用于历史版本兼容排障。先确认目录内容，再定点删除已知旧文件名：

```bash
cache_dir="${HOME}/.cache/raspberry-clock"
ls -l "$cache_dir"
rm -f \
  "$cache_dir/daily-quote.json" \
  "$cache_dir/daily-quote.new.tmp" \
  "$cache_dir/daily-quote.jpg" \
  "$cache_dir/daily-quote.jpeg" \
  "$cache_dir/daily-quote.png" \
  "$cache_dir/daily-quote.webp" \
  "$cache_dir/daily-quote.gif" \
  "$cache_dir/daily-quote.new.jpg" \
  "$cache_dir/daily-quote.new.jpeg" \
  "$cache_dir/daily-quote.new.png" \
  "$cache_dir/daily-quote.new.webp" \
  "$cache_dir/daily-quote.new.gif" \
  "$cache_dir/daily-quote.previous.jpg" \
  "$cache_dir/daily-quote.previous.jpeg" \
  "$cache_dir/daily-quote.previous.png" \
  "$cache_dir/daily-quote.previous.webp" \
  "$cache_dir/daily-quote.previous.gif"
```

不要执行针对 `~/.cache/raspberry-clock` 的整目录删除。

## 5. 关闭与重启

设备本机关闭：

```bash
pkill -f raspberry-clock
```

远程关闭：

```bash
ssh <device-host> "pkill -f raspberry-clock"
```

关闭后再次运行 `./run-clock.sh` 即可重启。

## 6. 仓库内验证命令

文档、代码或脚本变更后，至少按需执行下面这些命令：

```bash
cargo fmt --check
cargo test
./tests/install-autostart.sh
./tests/run-clock-config.sh
./tests/watch-clock-config.sh
cargo build --release
PKG_CONFIG_ALLOW_CROSS=1 cargo build --release --target aarch64-unknown-linux-gnu
cargo check --example render-preview
./scripts/render-previews.sh ./tmp/previews
git diff --check
```

如果本次变更涉及当前文档，请额外执行本轮约定的一致性关键词检查。预期结果是：命中只能出现在明确标注的历史兼容或迁移上下文中，不能出现在当前功能描述里。

## 7. 历史兼容与缓存排障

历史版本曾使用不同的缓存命名和远程图片路线。当前仓库仍保留对旧缓存文件的定点清理逻辑，目的是避免迁移后残留文件继续占空间或误导排障判断；这些名字不代表现行产品能力。
