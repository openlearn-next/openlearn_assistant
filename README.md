# OpenLearn-Next 助手

跨平台桌面 GUI 助手，用于管理 [openlearn-next](https://www.npmjs.com/package/openlearn-next) 教学平台的
**安装 / 卸载 / 升级 / 启动 / 停止 / 状态查看**。

基于 [Tauri v2](https://v2.tauri.app/)（Rust 后端 + 系统 WebView）+ 原生 TypeScript 前端，产物体积小。

## 功能

- **安装**：`npm install -g openlearn-next`（online 版从 npm 拉取；offline 版使用内嵌 tarball）
- **一键安装 Node 22**：系统级安装 Node.js 22 LTS（openlearn-next 依赖的 `better-sqlite3` 要求 Node ≥ 22），按需联网下载
- **卸载**：`npm uninstall -g openlearn-next`，可选「保留 / 彻底删除」用户数据（`~/openlearn-next`）
- **升级**：停机 → `npm update -g openlearn-next` → 重新启动
- **启动 / 停止**：以后台进程方式运行 `openlearn-next -p <port>`，向其注入配置环境变量，并记录 PID
- **状态**：检测 Node 版本、已装 openlearn-next 版本、运行态与端口，附实时日志面板
- **配置**：端口（默认 9000）、数据库路径（默认 `~/openlearn-next/data.db`）、`GEMINI_API_KEY`，启动时注入子进程（不写 `.env`）

> 注：SQLite 已内嵌于 openlearn-next 包中，无需单独安装数据库。

## 两种发布变体

| 变体 | 体积 | 说明 |
|------|------|------|
| **online** | ~5–15 MB | 安装/升级/Node 配置时联网下载，二进制极小 |
| **offline** | ~60 MB+ | 内嵌 Node 22 + openlearn-next tarball，适配无网机房 |

## 开发环境准备

### Linux（以 Arch 为例）
```bash
sudo pacman -S webkit2gtk-4.1 gtk3 libsoup3 cairo pango atk gdk-pixbuf2 glib2 pkgconf
```
Ubuntu/Debian：
```bash
sudo apt-get install libwebkit2gtk-4.1-dev build-essential curl wget file \
  libssl-dev libayatana-appindicator3-dev librsvg2-dev
```

### macOS
```bash
brew install gtk+3 webkit2gtk-4.1 libsoup@3 pkg-config
```
Xcode 命令行工具：`xcode-select --install`

### Windows
安装 [Visual Studio 2022](https://visualstudio.microsoft.com/)（含「使用 C++ 的桌面开发」工作负载）与
[WebView2 Runtime](https://developer.microsoft.com/microsoft-edge/webview2/)（Win11 通常已内置）。

### 通用
- Rust 工具链（[rustup](https://rustup.rs/)）
- Node.js 22 LTS
- Tauri CLI：`npm install` 后通过 `npm run tauri` 调用

## 本地运行 / 构建

```bash
npm install

# 开发模式（需图形界面）
npm run tauri dev

# 编译并打包（online 版）
npm run tauri build

# offline 版：先抓取内嵌资源，再带 offline feature 构建
./scripts/fetch-offline-assets.sh linux x86_64
npm run tauri -- build --features offline
```

产物位于 `src-tauri/target/release/bundle/`。

## CI 发布矩阵

`.github/workflows/release.yml` 自动构建 4 平台 × 2 变体：
Windows x64 · macOS universal (x64+arm64) · Linux x64 · Linux arm64，online / offline 各一份。
在 GitHub 上推送 `v*` 标签或手动 `workflow_dispatch` 触发。

## 实现说明

- Rust 后端（`src-tauri/src/`）通过 Tauri IPC 暴露命令：`detect_node` / `provision_node` /
  `install_pkg` / `uninstall_pkg` / `upgrade_pkg` / `start_service` / `stop_service` / `status` /
  `get_logs` / `load_settings` / `save_settings`。
- **提权**：系统级安装/升级/Node 配置会触发提权（Windows UAC / macOS `osascript` / Linux `pkexec`/`sudo`）。
  非管理员环境会失败——这是「系统级安装」方案的已知取舍。
- **进程托管**：启动后以独立会话/进程组运行（`setsid` / `DETACHED_PROCESS`），停止时杀整组（含 Worker Threads）。
- **offline 资源**：`scripts/fetch-offline-assets.sh` 抓取 Node 22 与 openlearn-next tarball 到
  `src-tauri/resources/`，由 `cargo` 的 `offline` feature 经 `include_bytes!` 内嵌。

## 目录结构

```
.
├── index.html
├── src/                     # 前端（Vanilla TS）
├── src-tauri/
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── resources/           # 仅 offline 变体：node-asset.* / openlearn-next.tgz
│   └── src/
│       ├── main.rs
│       ├── commands.rs      # Tauri 命令入口
│       ├── settings.rs      # 配置读写 + 注入 env 构造
│       ├── elevate.rs       # 跨平台提权
│       ├── node.rs          # Node 检测/下载/安装
│       ├── pkg.rs           # npm 安装/卸载/升级
│       └── service.rs       # 启动/停止/状态/日志
├── scripts/fetch-offline-assets.sh
└── .github/workflows/release.yml
```
