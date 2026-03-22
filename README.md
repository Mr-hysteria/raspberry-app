# 🚀 Raspberry Pi Zero 2 W 初始化与开发指南

## 1. 当前硬件配置清单 (Hardware Specs)
* **核心板：** Raspberry Pi Zero 2 W (Quad-core 64-bit Arm Cortex-A53 @ 1GHz)。
* **内存：** 512MB LPDDR2（资源极其宝贵，需精简运行）。
* **存储：** Micro SD Card (建议 Class 10 U3 以上)。
* **显示：** 5 寸 HDMI 触控屏 (物理分辨率 **$800 \times 480$**)。
* **网络：** 2.4GHz IEEE 802.11b/g/n 无线网卡（不支持 5G 频段）。
* **系统版本：** Debian 13 (Trixie) 64-bit，采用最新的 **DEB822** 软件源格式。

---

## 2. 初始化踩坑与配置教程

### ⚠️ 三大核心“天坑”总结
1.  **Wi-Fi 隔离：** Zero 2 W 只认 2.4G 信号。如果 Mac 在 5G 频段且路由器开启了 AP 隔离，`ping` 不通是物理级别的。
2.  **APT 自动更新锁：** 首次开机联网，系统会静默占用 `apt` 进程进行安全同步。此时手动安装会报 `Could not get lock`。
3.  **软件源格式更迭：** Debian 13 抛弃了旧的 `.list` 格式，改为 `/etc/apt/sources.list.d/*.sources`，传统的换源教程不再适用。

### 🛠️ 完整初始化流程

#### 第一步：物理级免密连接 (Mac 端执行)
```bash
# 1. 生成密钥 (一路回车)
ssh-keygen -t rsa -b 4096
# 2. 将公钥分发给树莓派
ssh-copy-id raspberry@raspberry.local
```

#### 第二步：国内镜像换源 (Debian Trixie 专属)
1. **修改 Debian 主源：** `sudo nano /etc/apt/sources.list.d/debian.sources`
   * 将 `URIs` 改为：`https://mirrors.tuna.tsinghua.edu.cn/debian/`
   * 将 `debian-security` 的 `URIs` 改为：`https://mirrors.tuna.tsinghua.edu.cn/debian-security/`
2. **修改树莓派专属源：** `sudo nano /etc/apt/sources.list.d/raspi.sources`
   * 将 `URIs` 改为：`https://mirrors.tuna.tsinghua.edu.cn/raspberrypi/`

#### 第三步：环境基座安装
```bash
sudo apt update
# fonts-wqy-zenhei: 中文字体，防止中文乱码
# unclutter: 自动隐藏鼠标
# x11-xserver-utils: 屏幕控制工具 (xset)
# nginx: 静态资源服务器
sudo apt install fonts-wqy-zenhei unclutter x11-xserver-utils nginx -y
# 安装浏览器相关环境
sudo apt update && sudo apt install chromium x11-xserver-utils -y
```

#### 第四步：锁定 5 寸屏分辨率
编辑 `/boot/firmware/config.txt`，末尾添加：
```text
hdmi_group=2
hdmi_mode=87
hdmi_cvt=800 480 60 6 0 0 0
hdmi_drive=1
```

