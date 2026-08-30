# Architecture

## 1. 概览

当前应用是一个固定 `800×480` 的 Slint 全屏窗口，围绕三条主线协同工作：

- 每秒生成一份本地时间快照，驱动时间、日期、昼夜模式与夜间息屏判断。
- 后台线程按本地日期请求当日阅读内容，通过 `mpsc` 渠道把结果送回 UI 线程。
- UI 线程把阅读文本、程序化背景色板、开始仪式状态和 DPMS 状态绑定到 `ui/clock.slint`。

## 2. 模块职责

- `src/main.rs`
  - 应用入口
  - Slint 属性绑定
  - 后台线程启动与 `mpsc` 收发
  - 每秒刷新时钟快照
- `src/daily_reading.rs`
  - 五条白名单路由
  - FNV-1a 日期选路
  - JSON 解析、必填校验、长度限制、负向词过滤
  - `current` / `previous` 双槽缓存
- `src/background.rs`
  - 日间四套色板
  - 夜间四套色板
  - 按日期稳定选择 `scene_variant`
- `src/domain.rs`
  - `23:30–07:00` 夜间窗口判断
  - 白天轻触开始仪式 `StartRitual`
- `src/display_power.rs`
  - 夜间 `60` 秒临时亮屏状态
  - `xset dpms force on/off` 执行
- `ui/clock.slint`
  - 窗口尺寸、字体、布局、底部提示文案
  - `screen-tapped` 回调
- `examples/render-preview.rs`
  - 用真实组件生成预览
- `scripts/render-previews.sh`
  - 批量渲染 `reading-day.png`、`reading-focus.png`、`reading-night.png`

## 3. 时钟与 UI 刷新链路

应用启动后先从 `default_cache_dir()` 读取 `daily-reading.json`，立即选择 `current`、`previous` 或内置诗句填充首屏，然后开始每秒重复的时钟刷新：

1. `read_clock_snapshot()` 通过 `libc::localtime_r` 读取本地时间。
2. 生成 `HH:MM`、秒数、日期星期、`night_mode`、`night_window`、UNIX 时间戳和 `YYYY-MM-DD` 形式的 `date_key`。
3. `background_for_date(date_key, night_mode)` 选择当日场景并更新 Slint 颜色属性。
4. `start_ritual.reconcile(night_window)` 在进入夜间窗口时清除开始状态。
5. `display_power.reconcile(night_window, timestamp)` 计算当前是否需要执行 DPMS 开关。

这条链路始终运行在 UI 线程，但它本身不做网络 I/O。

## 4. 今日阅读获取链路

`AppState` 里保存：

- `reading_sender`
- `reading_receiver`
- `reading_fetch_in_progress`
- `last_reading_fetch`
- `active_reading_date`

每次刷新窗口时，`maybe_start_reading_fetch()` 会根据三个条件决定是否发起请求：

- 当前没有正在进行的请求
- 已显示内容的 `fetched_for_date` 不等于今天
- 距离上次失败尝试至少过去 `15` 分钟

一旦需要刷新：

1. UI 线程记录 `last_reading_fetch = Instant::now()`。
2. 复制 `Sender`、缓存目录和 `local_date`。
3. `std::thread::spawn` 启动后台线程。
4. 后台线程调用 `fetch_and_cache()`，成功或失败都通过 `mpsc` 发回结果。
5. UI 线程在 `apply_reading_updates()` 中轮询 `Receiver`，成功则更新文本和 `active_reading_date`，失败则只清除进行中标记并保留现有显示。

这种设计保证了每秒时钟刷新与网络波动隔离。

## 5. 日期稳定选路与内容过滤

`src/daily_reading.rs` 维护五条固定今日诗词分类路由：

- `rensheng/dushu`
- `rensheng/zheli`
- `shanshui`
- `shenghuo/tianyuan`
- `rensheng/lizhi`

对本地日期字符串 `YYYY-MM-DD` 做 64 位 FNV-1a 哈希，再对路由数量取模，得到当天唯一分类。这样同一天重启不会切换分类，不同日期又会自然轮换。

响应进入展示前必须通过四层过滤：

1. `content` 非空
2. `origin` 非空
3. `author` 非空
4. `category` 非空

之后还要满足：

- `content.trim().chars().count() <= 40`
- 不包含负向词列表中的关键词

不满足时，函数返回错误，调用方继续保留当前显示内容。

## 6. 缓存模型

缓存文件固定为：

```text
~/.cache/raspberry-clock/daily-reading.json
```

结构是：

- `version`
- `current`
- `previous`

更新规则：

- 首次成功获取时，只填充 `current`
- 新日期成功获取时，旧 `current` 旋转到 `previous`
- 同一天再次成功获取时，只替换 `current`，不转动 `previous`
- 启动显示顺序固定为 `current` → `previous` → 内置诗句

写盘方式是同目录临时文件加原子重命名：

1. 写入 `daily-reading.json.tmp`
2. `sync_all()`
3. `fs::rename()` 覆盖正式文件

因此失败写入不会破坏上一份可用缓存。

## 7. 程序化背景链路

`src/background.rs` 为白天和夜间各维护四套静态色板，每套包含：

- `canvas`
- `wash_primary`
- `wash_secondary`
- `text_primary`
- `text_muted`
- `accent`
- `variant`

背景模块也按日期字符串做 FNV-1a 哈希，但使用的是 32 位实现，并对四套场景取模。输出结果再转换为 Slint `Color`，绑定到：

- `canvas-color`
- `wash-primary-color`
- `wash-secondary-color`
- `text-primary-color`
- `text-muted-color`
- `accent-color`
- `scene-variant`

`ui/clock.slint` 根据 `scene-variant` 调整三个半透明圆角块的位置和尺寸，从而在保持确定性的同时避免每天完全同构。

## 8. Slint 属性绑定

`AppWindow` 暴露的关键属性有：

- `time-text`
- `seconds-text`
- `date-weekday-text`
- `reading-content`
- `reading-source`
- `focus-active`
- 六个颜色属性
- `scene-variant`
- `night-mode`
- `ui-font-family`

运行时绑定策略：

- 时间快照更新 `time-text`、`seconds-text`、`date-weekday-text`
- 阅读更新只改 `reading-content` 和 `reading-source`
- 程序化背景更新六个颜色属性和 `scene-variant`
- 开始仪式只改 `focus-active`
- `night-mode` 反映当前是否处于夜色方案

底部提示文案并不在 Rust 端拼接，而是由 Slint 根据 `focus-active` 直接选择：

- 普通：`轻触一下，先读一页`
- 开始：`已经开始了，先读一页就好`

## 9. 触摸、开始仪式与 DPMS

`ui/clock.slint` 用全屏 `TouchArea` 把点击转为 `screen-tapped()`。Rust 侧回调执行顺序是：

1. 读取当前本地时间，判断是否处于 `night_window`
2. `display_power.touch(night_window, timestamp)`
3. `display_power.reconcile(night_window, timestamp)`
4. `start_ritual.tap(night_window)`
5. 若 DPMS 状态发生变化，则调用 `apply_screen_power()`
6. 将 `focus-active` 回写 UI

这里的关键边界是：

- 白天轻触会切换开始仪式
- 夜间轻触不会切换开始仪式
- 夜间轻触只延长亮屏截止时间
- 进入夜间窗口时，`start_ritual.reconcile(true)` 自动清除开始状态

DPMS 具体命令由 `src/display_power.rs` 执行：

- 亮屏：`xset dpms force on`
- 熄屏：`xset dpms force off`

命令失败只会写 stderr，不会终止应用。

## 10. 预览与人工验视

`examples/render-preview.rs` 复用真实 `AppWindow`，用固定示例数据生成三种状态截图。它还通过测试锁定预览字体默认值为 `WenQuanYi Zen Hei`。

`scripts/render-previews.sh` 的职责是：

- 调用示例生成三份 PPM
- 用 `sips` 或 ImageMagick `convert` 转成 PNG
- 校验输出尺寸必须是 `800×480`

这条链路的作用不是替代真机验证，而是让布局、字体和提示切换在仓库内就可复现、可对比。

## 11. 历史兼容

当前运行时只维护 `daily-reading.json`。之所以仍保留对历史文件名的清理，是为了迁移后自动回收旧版缓存占用，并减少排障歧义。相关清理目标只存在于兼容逻辑中，例如：

- `daily-quote.json`
- `daily-quote.new.tmp`
- `daily-quote.*`
- `daily-quote.new.*`
- `daily-quote.previous.*`

这些名字只代表历史遗留文件，不代表当前产品仍在使用旧路线。
