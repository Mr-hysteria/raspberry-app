# Agent Guide

## 1. 项目目标

- 项目名称：`raspberry-clock`
- 目标：在 `Raspberry Pi Zero 2 W` 上稳定运行一个全屏时钟程序，替代历史上的 `HTML + Chromium` 方案。
- 当前技术路线：`Rust + Slint` 原生应用，优先降低内存占用、减少依赖、方便开机自启。
- 目标设备约束：
  - 设备：`Raspberry Pi Zero 2 W`
  - 内存：`512MB`
  - 系统：`Debian 13 (Trixie) 64-bit`
  - 屏幕分辨率：`800x480`

## 2. 当前状态

- 当前已完成：
  - 原生全屏时钟界面
  - 时间、秒、日期、星期展示
  - 中文字体显示
  - `800x480` 小屏布局适配
  - 树莓派初始化脚本与运行脚本
- 当前主线：
  - 保持程序在低内存设备上的稳定运行
  - 继续优化 UI、部署体验和可维护性
- 当前风险：
  - 项目还缺少专门的架构记录与迭代记录，需要持续沉淀
  - `开发SOP.md` 含有设备 IP / 凭据示例，后续整理时要避免继续扩散敏感信息

## 3. 快速上手

### 先读哪些文件

建议按这个顺序建立上下文：

1. `README.md`
2. `agent.md`
3. `开发SOP.md`
4. `src/main.rs`
5. `ui/clock.slint`
6. `run-clock.sh`
7. `docs/decisions.md`
8. `docs/iteration-log.md`

### 常用命令

本地构建：

```bash
cargo build
cargo build --release
```

Mac 上交叉编译：

```bash
PKG_CONFIG_ALLOW_CROSS=1 cargo build --release --target aarch64-unknown-linux-gnu
```

树莓派运行：

```bash
./run-clock.sh
```

树莓派初始化：

```bash
chmod +x scripts/bootstrap-pi.sh run-clock.sh
./scripts/bootstrap-pi.sh
```

## 4. 代码结构

```text
Cargo.toml              Rust 依赖与构建配置
build.rs                编译 Slint UI 资源
src/main.rs             时间读取、状态刷新、应用入口
ui/clock.slint          全屏时钟 UI 布局与样式
run-clock.sh            树莓派桌面环境下的启动脚本
scripts/bootstrap-pi.sh 树莓派依赖安装与首次构建
.cargo/config.toml      Cargo 源与交叉编译 linker 配置
```

### 核心模块职责

- `src/main.rs`
  - 设置 Slint 后端与全屏参数
  - 每秒刷新一次时钟快照
  - 通过 `libc::localtime_r` 读取本地时间
- `ui/clock.slint`
  - 定义窗口尺寸、颜色、文字布局与字体
  - 当前布局按 `800x480` 固定设计
- `run-clock.sh`
  - 负责关闭屏保、关闭 DPMS、隐藏鼠标、避免重复启动
  - 优先运行 `release`，回退到 `debug`

## 5. 关键约束

- 优先保证在 `512MB` 设备上稳定运行，避免为了“功能丰富”显著增加内存占用。
- 默认渲染路径是 `Slint software renderer`，不要假设 GPU 加速存在。
- UI 修改必须考虑 `800x480` 屏幕，不要只按桌面显示器效果判断。
- 启动脚本承担现场运行职责，改动时要特别注意：
  - 是否影响 X11 环境变量
  - 是否影响全屏
  - 是否破坏重复启动时的进程清理逻辑
- 涉及部署说明时，不要在新文档里继续固化明文密码、IP、个人路径等环境特定信息。

## 6. Agent 工作方式

### 接手任务前

- 先确认任务属于哪一类：
  - UI 调整：重点看 `ui/clock.slint`
  - 时间/逻辑调整：重点看 `src/main.rs`
  - 启动/部署问题：重点看 `run-clock.sh`、`scripts/bootstrap-pi.sh`、`开发SOP.md`
  - 构建/交叉编译问题：重点看 `Cargo.toml`、`.cargo/config.toml`、`rust-toolchain.toml`

### 改动时默认策略

- 优先做小步、可验证改动，避免一次重写整套运行链路。
- 优先保留现有“树莓派可直接运行”的能力。
- 新增依赖前先判断是否真的必要，特别是面向树莓派运行时依赖。
- 如果更改会影响部署方式、目录结构、运行命令或业务行为，必须同步更新对应文档。

### 完成任务后

- 至少检查以下文档是否需要更新：
  - `README.md`
  - `agent.md`
  - `开发SOP.md`
  - `docs/decisions.md`
  - `docs/iteration-log.md`

## 7. 文档更新契约

不是每次都要更新所有文档，但每次迭代结束都必须检查以下资产是否需要更新。

### 必检查

- `README.md`
  - 用户可见能力变化
  - 启动方式变化
  - 依赖或目录变化
- `agent.md`
  - 当前状态变化
  - 风险变化
  - 接手顺序或协作规则变化
- `开发SOP.md`
  - 编译、部署、运行、排障流程变化
- `docs/iteration-log.md`
  - 记录本次迭代目标、完成项、遗留项、风险、建议

### 按需新增

- `docs/decisions.md`
  - 出现重要技术取舍时新增记录
  - 要说明“为什么这样做”，而不仅是“做了什么”

## 8. 迭代收尾清单

每次完成一轮功能、修复或部署调整后，按顺序检查：

1. 代码是否已验证最关键路径
2. 本次改动影响了哪些运行命令、脚本或目录
3. `README.md` 是否需要同步
4. `开发SOP.md` 是否需要同步
5. `docs/decisions.md` 是否需要新增决策记录
6. `docs/iteration-log.md` 是否已补本次迭代摘要
7. `agent.md` 中的当前状态、风险、推荐阅读顺序是否仍然准确

## 9. 当前建议补强项

- 补一份 `docs/architecture.md`，记录运行链路与模块关系
- 清理或脱敏 `开发SOP.md` 中的环境专属信息
- 增加最小验证清单，例如：
  - 本地 `cargo build`
  - 树莓派脚本可运行
  - UI 在 `800x480` 下布局未错位
