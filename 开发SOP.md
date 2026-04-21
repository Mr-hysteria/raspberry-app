# 开发 SOP — Raspberry Pi 全屏时钟

目标设备：Raspberry Pi Zero 2W · 512MB · Debian 13 (Trixie) 64-bit · 800×480

---

## 0. SSH 连接树莓派

如果你只是想先连上树莓派，推荐先做下面几步。

### 0.1 确认树莓派 IP

如果你记得设备 IP，可以直接跳到下一步。

如果不记得，可以在树莓派本机终端执行：

```bash
hostname -I
```

也可以在路由器后台查看树莓派分配到的局域网 IP。

> 当前历史示例里使用过 `192.168.1.12`，但真实 IP 可能已经变化，建议每次以当前设备实际 IP 为准。

### 0.2 从开发机测试 SSH 是否可连

在 Mac 本机执行：

```bash
ssh raspberry@<树莓派IP>
```

首次连接通常会看到主机指纹确认，输入：

```text
yes
```

随后输入树莓派账号密码即可登录。

### 0.3 登录成功后的常用检查

登录后建议先执行：

```bash
whoami
pwd
uname -a
echo $DISPLAY
```

如果只是远程维护文件或执行命令，`DISPLAY` 为空是正常的。
如果你要远程启动桌面上的全屏程序，则需要在启动命令里显式传入 `DISPLAY=:0` 和 `XAUTHORITY=/home/raspberry/.Xauthority`。

### 0.4 推荐改成 SSH key 连接（可选，但建议）

如果你不想每次输密码，可以在 Mac 本机生成并写入公钥：

```bash
ssh-keygen -t ed25519
ssh-copy-id raspberry@<树莓派IP>
```

如果系统没有 `ssh-copy-id`，也可以手动追加到树莓派的 `~/.ssh/authorized_keys`。

后续即可直接：

```bash
ssh raspberry@<树莓派IP>
```

### 0.5 连接失败时先检查

- 树莓派和开发机是否在同一局域网
- 树莓派是否已经启用 SSH
- IP 是否变化
- 用户名是否仍为 `raspberry`
- 防火墙或路由器是否拦截了 22 端口

如果树莓派本机还没启用 SSH，可在树莓派上执行：

```bash
sudo systemctl enable ssh
sudo systemctl start ssh
sudo systemctl status ssh
```

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

推荐的通用写法：

```bash
ssh raspberry@<树莓派IP> \
  "DISPLAY=:0 XAUTHORITY=/home/raspberry/.Xauthority \
   bash /home/raspberry/Desktop/code/raspberry-app/run-clock.sh &"
```

如果你当前环境仍然依赖密码直连，也可以继续使用历史上的 `sshpass` 方式：

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

推荐的通用写法：

```bash
ssh raspberry@<树莓派IP> \
  "pkill -f raspberry-clock && echo '已关闭'"
```

如果你当前环境仍然依赖密码直连，也可以继续使用历史上的 `sshpass` 方式：

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

### 5.1 推荐保存一个 SSH 别名（可选）

如果你经常连接同一台树莓派，可以在 Mac 的 `~/.ssh/config` 中增加：

```sshconfig
Host raspberry-clock
  HostName <树莓派IP>
  User raspberry
```

之后可以直接：

```bash
ssh raspberry-clock
scp <本地文件> raspberry-clock:<远程路径>
```
