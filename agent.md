# Agent Guide

## 1. 项目目标

- 项目名称：`raspberry-clock`
- 当前目标：在 `Raspberry Pi Zero 2 W` 上稳定运行一个全屏每日阅读屏，用时间、古诗文片段和极轻的开始仪式服务书桌场景，而不是任务压力场景。
- 当前技术路线：`Rust + Slint` 原生应用，优先降低运行时占用、减少依赖、方便开机自启。
- 目标设备约束：
  - 设备：`Raspberry Pi Zero 2 W`
  - 内存：`512MB`
  - 系统：`Debian 13 (Trixie) 64-bit`
  - 屏幕分辨率：`800×480`
  - 会话环境：`X11`

## 2. 当前状态

- 当前已完成：
  - 全屏每日阅读屏界面
  - 时间、秒、日期、星期展示
  - 五条白名单今日诗词分类路由与按日期稳定选择
  - 必填字段校验、`40` 字长度限制与负向词过滤
  - `current` / `previous` 双槽 JSON 缓存与内置回退诗句
  - 程序化背景色板与昼夜色系切换
  - 白天轻触开始仪式；夜间轻触仅临时亮屏 `60` 秒
  - `23:30–07:00` 自动息屏
  - 预览示例与三张 `800×480` 截图生成脚本
  - 树莓派初始化脚本、部署脚本、桌面自启动与异常退出守护
- 当前主线：
  - 保持程序在低内存设备上的稳定运行
  - 观察今日诗词接口可用性、过滤结果与一周连续运行表现
- 当前风险：
  - 已完成真机部署、X11 点击、DPMS 关/开、进程存活、字体、缓存与 `800×480` 截图验证；实测记录见 `docs/iteration-log.md`
  - 当前白天远程验收无法替代夜间用实体触摸控制器做一次在场唤醒观察

## 3. 快速上手

### 先读哪些文件

建议按这个顺序建立上下文：

1. `README.md`
2. `agent.md`
3. `docs/architecture.md`
4. `docs/prd-healing-life.md`
5. `开发SOP.md`
6. `src/main.rs`
7. `src/daily_reading.rs`
8. `ui/clock.slint`
9. `run-clock.sh`
10. `docs/decisions.md`
11. `docs/iteration-log.md`

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

预览生成：

```bash
cargo check --example render-preview
./scripts/render-previews.sh ./tmp/previews
```

树莓派初始化：

```bash
chmod +x scripts/bootstrap-pi.sh run-clock.sh
./scripts/bootstrap-pi.sh
```

## 4. 代码结构

```text
Cargo.toml                  Rust 依赖与构建配置
build.rs                    编译 Slint UI 资源
src/main.rs                 应用入口、时钟刷新、后台线程与 Slint 绑定
src/daily_reading.rs        今日阅读请求、双槽缓存、过滤与离线回退
src/background.rs           程序化背景色板与日期稳定选择
src/domain.rs               夜间窗口判断与开始仪式状态
src/display_power.rs        X11 DPMS 协调与 60 秒唤醒逻辑
ui/clock.slint              800×480 固定布局与属性绑定
examples/render-preview.rs  生产组件预览示例
scripts/render-previews.sh  预览图生成脚本
run-clock.sh                树莓派桌面环境下的启动脚本
scripts/bootstrap-pi.sh     树莓派依赖安装与首次构建
scripts/install-autostart.sh 自启动安装
scripts/watch-clock.sh      时钟进程守护
.cargo/config.toml          Cargo 源与交叉编译 linker 配置
```

### 核心模块职责

- `src/main.rs`
  - 设置 Slint 后端与全屏参数
  - 每秒刷新一次时钟快照
  - 通过 `mpsc` 接收后台阅读更新
  - 绑定阅读内容、程序化背景、开始仪式和夜间息屏状态
- `src/daily_reading.rs`
  - 选择当天分类接口
  - 解析 JSON、执行长度与负向词过滤
  - 维护 `daily-reading.json` 的 `current` / `previous` 槽位
  - 清理历史版本遗留缓存文件
- `src/background.rs`
  - 为日期选择确定性的日间或夜间色板
  - 向 Slint 输出画布、洗色块和文字颜色
- `src/domain.rs`
  - 计算 `23:30–07:00` 夜间窗口
  - 管理白天轻触切换的 `StartRitual`
- `src/display_power.rs`
  - 管理夜间触摸唤醒截止时间
  - 调用 `xset dpms force on/off`
- `ui/clock.slint`
  - 定义 `800×480` 固定布局、字体和底部提示切换
- `examples/render-preview.rs`
  - 使用真实组件生成普通、开始、夜间预览
- `scripts/render-previews.sh`
  - 把预览示例输出的 PPM 转成 PNG 并校验尺寸
- `run-clock.sh`
  - 负责关闭屏保、关闭自动 DPMS、隐藏鼠标、避免重复启动
  - 优先运行 `release`，回退到 `debug`

## 5. 关键约束

- 优先保证在 `512MB` 设备上稳定运行，避免为了功能丰富显著增加内存占用。
- 默认渲染路径是 `Slint software renderer`，不要假设 GPU 加速存在。
- 阅读内容请求必须在后台线程运行，不能阻塞每秒时钟刷新。
- 运行时不得请求、解码或缓存远程图片。
- 夜间息屏依赖 X11 的 `xset dpms force on/off`，部署环境必须保留 `x11-xserver-utils`。
- Debian 13 当前安装的是经典版 `unclutter` 8.x，只接受 `-idle`、`-jitter`、`-root` 等单横线参数；不要改成长参数风格。
- UI 修改必须考虑 `800×480` 屏幕，不要只按桌面显示器效果判断。
- 开始仪式只允许白天切换文案与文字对比度，不计时、不计数、不落盘。
- 涉及部署说明时，不要在文档里固化明文密码、固定 IP 或个人开发机路径。

## 6. 协作方式

### 接手任务前

- 先确认任务属于哪一类：
  - UI 调整：重点看 `ui/clock.slint`、`src/background.rs`
  - 阅读链路调整：重点看 `src/daily_reading.rs`、`src/main.rs`
  - 触摸/息屏问题：重点看 `src/domain.rs`、`src/display_power.rs`、`run-clock.sh`
  - 启动/部署问题：重点看 `run-clock.sh`、`scripts/*.sh`、`开发SOP.md`
  - 构建/交叉编译问题：重点看 `Cargo.toml`、`.cargo/config.toml`、`rust-toolchain.toml`

### 改动时默认策略

- 优先做小步、可验证改动，避免一次重写整套运行链路。
- 优先保留现有树莓派可直接运行的能力。
- 新增依赖前先判断是否真的必要，特别是面向树莓派运行时依赖。
- 如果更改会影响部署方式、目录结构、运行命令、缓存格式或用户可见行为，必须同步更新文档。

### 完成任务后

- 至少检查以下文档是否需要更新：
  - `README.md`
  - `agent.md`
  - `docs/architecture.md`
  - `开发SOP.md`
  - `docs/prd-healing-life.md`
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
  - 推荐阅读顺序或协作规则变化
- `docs/architecture.md`
  - 模块关系、状态流、缓存或交互链路变化
- `开发SOP.md`
  - 编译、预览、部署、运行、排障流程变化
- `docs/prd-healing-life.md`
  - 产品目标、范围、交互、视觉方向变化
- `docs/iteration-log.md`
  - 本次迭代目标、完成项、已知证据、遗留项与建议

### 按需新增或更新

- `docs/decisions.md`
  - 出现重要技术取舍时新增记录
  - 已被替代的历史决策要明确标记状态

## 8. 迭代收尾清单

每次完成一轮功能、修复或部署调整后，按顺序检查：

1. 关键路径是否有对应验证
2. 本次改动影响了哪些运行命令、脚本、目录或缓存
3. `README.md` 是否需要同步
4. `docs/architecture.md` 是否需要同步
5. `docs/prd-healing-life.md` 是否需要同步
6. `开发SOP.md` 是否需要同步
7. `docs/decisions.md` 是否需要新增或调整状态
8. `docs/iteration-log.md` 是否已补本次迭代摘要
9. `agent.md` 中的当前状态、风险、推荐阅读顺序是否仍然准确
