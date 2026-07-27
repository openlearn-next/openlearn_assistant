# Changelog

所有值得注意的变更记录于此。

格式基于 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.1.0/)，
版本号遵循 [语义化版本](https://semver.org/lang/zh-CN/)。

## [0.1.3] - 2026-07-27

### Fixed
- 替换 1×1px 占位图标为 1024×1024 应用图标
- macOS：跳过代码签名（CI 环境无证书）
- Windows：跳过 MSI 打包（无需 WiX 工具链）
- Linux CI：补充 `libfuse2` 依赖以支持 AppImage 构建

## [0.1.2] - 2026-07-27

### Fixed
- CI workflow 中重复的 `push` 键导致 branches 触发失效
- Release job 缺少 `contents: write` 权限导致无法创建 Release

## [0.1.1] - 2026-07-27

### Changed
- 设置面板中移除 GEMINI_API_KEY 输入框
- CI workflow：main 分支提交自动触发编译，tag 推送自动创建 Release

### Added
- 添加 Cargo.lock 到版本控制

## [0.1.0] - 2026-07-27

### Added
- 首个版本：跨平台桌面 GUI 助手（Tauri v2）
- openlearn-next 的安装 / 卸载 / 升级 / 启动 / 停止 / 状态查看
- 一键安装 Node.js 22 LTS
- 端口、数据库路径等运行时配置
- 实时日志面板
- 在线版与离线版两种发布变体

[0.1.3]: https://github.com/openlearn-next/openlearn_assistant/compare/v0.1.2...v0.1.3
[0.1.2]: https://github.com/openlearn-next/openlearn_assistant/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/openlearn-next/openlearn_assistant/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/openlearn-next/openlearn_assistant/releases/tag/v0.1.0
