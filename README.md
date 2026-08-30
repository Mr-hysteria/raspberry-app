# Raspberry Pi 每日阅读屏

这是一个针对 `Raspberry Pi Zero 2 W` 优化的全屏阅读摆件项目，采用 `Rust + Slint` 原生实现，不依赖 `Chromium`。当前版本把时间保留为稳定视觉锚点，用每日古诗文片段、确定性的程序化背景和极轻的轻触开始仪式，降低开始阅读的心理门槛。

## 目标设备

- 树莓派型号：`Raspberry Pi Zero 2 W`
- 内存：`512MB`
- 屏幕分辨率：`800×480`
- 系统：`Debian 13 (Trixie) 64-bit`
- 显示环境：`X11`

## 当前体验

- 显示 `HH:MM` 大时钟、弱化秒数、完整日期与星期
- 显示“今日一页”中文诗句，以及“作者《作品》 · 分类”来源行
- 白天轻触切换开始仪式文案；夜间轻触只负责临时亮屏 `60` 秒
- 每天 `23:30–07:00` 通过 X11 DPMS 自动息屏
- 使用程序化色板和圆角形状绘制背景，不请求、不解码、不缓存远程图片
- 每天只请求一个今日诗词接口；同日成功后不再重复请求，失败后至少 `15` 分钟再试
- 本地缓存保留 `current` / `previous` 两条阅读内容；无缓存或缓存损坏时回退到内置诗句
- 提供与生产 UI 共用组件的预览命令，可生成普通、开始、夜间三张 `800×480` 截图
- LightDM/LXDE 桌面登录后自动启动，并由守护脚本负责异常退出恢复

## 内容来源

程序只会从下面五个白名单接口里，按本地日期稳定选择当天分类：

- `https://v1.jinrishici.com/rensheng/dushu.json`
- `https://v1.jinrishici.com/rensheng/zheli.json`
- `https://v1.jinrishici.com/shanshui.json`
- `https://v1.jinrishici.com/shenghuo/tianyuan.json`
- `https://v1.jinrishici.com/rensheng/lizhi.json`

响应必须同时包含非空的 `content`、`origin`、`author` 和 `category`。内容超过 `40` 个 Unicode 字符，或命中负向词过滤时会被拒绝展示，并保留当前内容。

## 项目结构

```text
Cargo.toml                    Rust 依赖与构建配置
build.rs                      编译 Slint UI
src/main.rs                   应用入口、时钟刷新、后台线程与 Slint 绑定
src/daily_reading.rs          今日阅读请求、过滤、双槽缓存与离线回退
src/background.rs             程序化背景色板与日期稳定选择
src/domain.rs                 夜间窗口判断与开始仪式状态
src/display_power.rs          X11 DPMS 状态协调与 60 秒唤醒逻辑
ui/clock.slint                800×480 固定布局与视觉样式
examples/render-preview.rs    生产组件预览示例
scripts/render-previews.sh    生成三张 800×480 PNG 预览
run-clock.sh                  树莓派桌面环境启动脚本
scripts/bootstrap-pi.sh       树莓派初始化脚本
scripts/install-autostart.sh  安装桌面自启动项
scripts/watch-clock.sh        异常退出自动恢复
scripts/deploy-and-run-pi.sh  本地交叉编译、上传并远程启动
tests/*.sh                    启动、自启动与守护脚本检查
docs/architecture.md          当前架构与状态流
```

## 快速开始

初始化树莓派环境：

```bash
chmod +x scripts/bootstrap-pi.sh run-clock.sh
./scripts/bootstrap-pi.sh
```

本地构建：

```bash
cargo build --release
```

在树莓派桌面环境运行：

```bash
./run-clock.sh
```

生成三张预览图：

```bash
cargo check --example render-preview
./scripts/render-previews.sh ./tmp/previews
```

输出目录中会得到：

- `reading-day.png`
- `reading-focus.png`
- `reading-night.png`

## 部署

如果你在本地开发机上交叉编译并上传到树莓派，可以在仓库根目录执行：

```bash
./scripts/deploy-and-run-pi.sh
```

常用覆盖参数：

- `PI_SSH_HOST=<ssh-alias> ./scripts/deploy-and-run-pi.sh`
- `PI_HOST=<device-host-or-ip> ./scripts/deploy-and-run-pi.sh`
- `PI_REMOTE_GIT_PULL=1 ./scripts/deploy-and-run-pi.sh`

更完整的开发、交叉编译、部署与排障说明见 `开发SOP.md`。

## 文档入口

建议按下面顺序建立上下文：

- `README.md`：项目概览、启动方式、当前特性
- `agent.md`：后续协作者的上下文入口、约束和文档更新契约
- `docs/architecture.md`：模块职责、状态流、缓存和交互链路
- `开发SOP.md`：开发、预览、部署和缓存排障
- `docs/prd-healing-life.md`：当前产品需求文档 `v3`
- `docs/decisions.md`：关键技术决策记录
- `docs/iteration-log.md`：迭代历史与已知证据

## 历史迁移说明

当前运行时只维护 `daily-reading.json`。如果历史版本遗留了旧缓存命名或旧界面认知，请优先查看 `docs/architecture.md` 和 `开发SOP.md` 中的历史兼容章节，再决定是否清理旧文件。
