# Raspberry Pi 全屏时钟

这是一个针对 `Raspberry Pi Zero 2 W` 优化的全屏时钟项目，现已切换为 `Rust + Slint` 原生实现，不再依赖 `Chromium`。

## 目标设备

- 树莓派型号：`Raspberry Pi Zero 2 W`
- 内存：`512MB`
- 屏幕分辨率：`800x480`
- 系统：`Debian 13 (Trixie) 64-bit`

## 为什么改成 Slint

原先的 `HTML + Chromium` 方案虽然实现快，但对 `512MB` 内存设备不够友好。这个项目现在改成了 `Slint` 原生界面：

- 常驻内存更低
- 无浏览器首次运行弹窗
- 不依赖 Node.js
- 更适合后续做开机直启

## 项目结构

```text
Cargo.toml            Rust 依赖与构建配置
build.rs              编译 Slint UI
src/main.rs           时钟逻辑
ui/clock.slint        全屏界面
run-clock.sh          运行脚本
scripts/bootstrap-pi.sh  树莓派初始化脚本
```

## 文档入口

如果是第一次接手这个仓库，建议优先看下面这些文档：

- `README.md`：项目概览、启动方式、当前特性
- `agent.md`：面向后续 agent / 开发者的协作入口、上下文索引、文档更新契约
- `开发SOP.md`：开发、交叉编译、部署到树莓派的操作说明
- `docs/decisions.md`：关键技术决策记录
- `docs/iteration-log.md`：每次迭代的目标、完成项、遗留项与后续建议

每次迭代结束后，建议至少检查上述文档是否需要同步更新，避免项目知识只留在对话或提交记录里。

## 首次安装

在树莓派项目目录执行：

```bash
cd /home/raspberry/Desktop/code/raspberry-app
chmod +x scripts/bootstrap-pi.sh run-clock.sh
./scripts/bootstrap-pi.sh
```

这个脚本会完成：

```text
1. 通过 rsproxy 安装 stable Rust 工具链
2. 安装 Slint 所需的 X11 / fontconfig 依赖
3. 优先尝试 release 编译
4. 如果 release 因内存不足失败，则自动回退到 debug 编译
```

## 启动时钟

```bash
cd /home/raspberry/Desktop/code/raspberry-app
./run-clock.sh
```

`run-clock.sh` 会：

```text
1. 关闭屏保、自动黑屏和 DPMS
2. 尝试隐藏鼠标光标
3. 优先启动 target/release/raspberry-clock
4. 若 release 不存在，则自动启动 target/debug/raspberry-clock
5. 使用 Slint software renderer 全屏显示
```

## 当前特性

- 原生全屏时钟界面
- 显示时、分、秒
- 显示日期与星期
- 中文字体显示
- 针对 `800x480` 小屏布局优化
- 软件渲染，无需 GPU 加速

## 可清理的旧方案

项目已经不再需要以下旧路线组件：

- `Chromium`
- `Nginx`
- `src/index.html`

如果系统中仍保留这些包，可以在确认新程序运行正常后卸载。

推荐清理命令：

```bash
sudo apt purge -y chromium chromium-common chromium-l10n chromium-sandbox rpi-chromium-mods nginx nginx-common
sudo apt autoremove -y
```
