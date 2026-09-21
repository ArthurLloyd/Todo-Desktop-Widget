# Todo Desktop Widget · 涂风改良版

一个与 Microsoft To Do 同步的 Windows 桌面待办组件。它不是 Microsoft To Do 的替代品，而是把常用清单放在桌面上，并保留本地离线操作能力。

本项目基于 [Chenzo1001/Microsoft-Todo-Desktop-Widget](https://github.com/Chenzo1001/Microsoft-Todo-Desktop-Widget) 改编。原项目作者为 Chenzo1001；本改良版本的改良作者为 **涂风**。本仓库是私人维护的改良版本，不代表 Microsoft 或原项目作者的官方发布。

> An independently customized Windows desktop companion for Microsoft To Do. Based on Chenzo1001's project; improved by 涂风.

## 这个版本做了什么

- **桌面窗口模式**：固定模式下，Widget 留在桌面，按 `Win + D` 后仍可见，Chrome、VS Code 等普通窗口可以自然盖住它；普通模式下则按常规窗口规则运行，`Win + D` 会隐藏它。
- **多个 Widget 与分区**：可以创建多个独立窗口；每个窗口可采用单分区、列、行或网格布局。分区可绑定不同的 Microsoft To Do 清单，调整相邻分区的尺寸并持久化。
- **独立主题**：窗口和分区可分别指定主题；提供内置主题、自定义主题编辑器，以及到期、重要、待同步等任务状态的语义颜色。
- **本地优先的任务操作**：创建、编辑、完成和删除任务先写入本地 SQLite，再由全局 Outbox 与 Rust SyncCoordinator 在后台上传。断网时保留待同步操作，恢复连接后继续重试。
- **Microsoft To Do 同步**：通过浏览器完成 OAuth Authorization Code + PKCE 登录，使用 Microsoft Graph 同步清单和任务。手动同步与后台周期同步共用同一个协调层。
- **桌面应用体验**：系统托盘、快捷添加、单实例运行、可选开机启动、任务详情独立窗口，以及简体中文和英文界面。

Windows 桌面固定采用顶级窗口的 Desktop Owner 模式，不把 Tauri/WebView2 窗口通过 `SetParent` 改成 Explorer 子窗口。这样保留了正常的 DPI、输入和 WebView2 行为。不同 Windows 环境下的窗口层级仍应以实机测试为准。

## 当前发布状态

当前版本号为 `0.1.0`。Windows EXE、NSIS 和 MSI 构建流程已经具备，但此私人仓库**没有公开的 GitHub Release**，安装包也不提交到 Git 历史。候选版的测试结果与未完成的验收项目见 [RC 测试报告](docs/RC_TEST_REPORT.md) 和 [发布检查清单](docs/RELEASE_CHECKLIST.md)。

当前 Windows 安装包未签名；公开分发、升级、MSI 生命周期、多 DPI 等事项仍需进一步确认。不要把此仓库视为已完成公开发布的正式产品。

## 技术栈

Tauri v2、Rust、React、TypeScript、Vite、SQLite（`rusqlite`）、Microsoft Graph / Microsoft To Do。

所有 Widget 和分区共享同一份本地任务缓存、Outbox、Microsoft 会话与 Rust SyncCoordinator；不会为每个窗口创建一套同步服务。

## 本地开发

需要 Node.js、Rust 工具链以及 Tauri v2 在当前平台要求的构建环境。安装前端依赖：

```powershell
npm install
```

自行构建和登录时，需要 Microsoft Entra 的桌面公共客户端 Application (client) ID。配置移动和桌面应用的 `http://localhost` 回调，并授予委托权限 `User.Read`、`Tasks.ReadWrite`、`offline_access`。不要创建 Client Secret。Tenant 默认是 `common`。

在 PowerShell 中启动：

```powershell
$env:MICROSOFT_CLIENT_ID = "your application client id"
$env:MICROSOFT_TENANT = "common"
npm run tauri:dev
```

也可以在本地使用被 Git 忽略的 `.env`；不要提交它。仅运行 `npm run dev` 会启动 Vite 前端，不等同于包含 Rust、窗口和系统集成功能的完整 Tauri 应用。

开发检查：

```powershell
npm run build
npm run test:frontend
cargo check --manifest-path src-tauri/Cargo.toml
cargo fmt --check --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml
```

## Windows 构建

使用自己的 Entra Application (client) ID 构建私人候选版：

```powershell
$env:MICROSOFT_CLIENT_ID = "your application client id"
$env:MICROSOFT_TENANT = "common"
npm run release:windows
```

该脚本执行版本与配置预检、前端和 Rust 检查、Windows 打包及产物 smoke test。生成位置：

```text
src-tauri/target/release/ms-todo-desktop-widget.exe
src-tauri/target/release/bundle/nsis/Todo Desktop Widget_<version>_x64-setup.exe
src-tauri/target/release/bundle/msi/Todo Desktop Widget_<version>_x64_en-US.msi
```

NSIS 是当前推荐的个人安装包，使用当前用户安装范围；MSI 面向 Windows Installer / 受管部署。两者均依赖 WebView2 下载引导方案。未签名构建可能触发 Windows SmartScreen 的“未知发布者”提示。

`MICROSOFT_CLIENT_ID` 是公共客户端标识符，不是 Client Secret，但本仓库仍不提交真实的本地构建配置。不要把 Client Secret、Access Token、Refresh Token、证书私钥、用户数据库或安装包加入 Git。

## 数据存储与隐私

当前 Windows 版本的 SQLite 缓存、Widget 配置、主题与待同步操作位于：

```text
%APPDATA%\com.local.ms-todo-desktop-widget\cache.sqlite3
```

**目前数据不会跟随 EXE 或安装位置移动。** 安装到 D 盘也不会自动把数据库迁到 D 盘；“便携数据模式”尚未实现。不要手动移动正在使用的 SQLite 数据库。

Refresh Token 优先通过 Rust `keyring` 写入 Windows Credential Manager；当前兼容路径还可能在 SQLite settings 中保留回退值。卸载程序与清理 AppData 并不等同于清除 Windows 凭据。正常升级通常保留 AppData；NSIS 卸载时可选择是否删除应用数据。

## macOS 说明

仓库保留原项目的 macOS Tauri 构建路径和 `macos-widget/` 中的 WidgetKit 扩展。本改良版的主要开发与实机验收集中在 Windows；上文的 Desktop Owner、`Win + D`、NSIS/MSI 等行为不适用于 macOS，也不应据此推断 macOS 已完成同等验收。

macOS 构建必须在 macOS 上进行。相关命令包括 `npm run tauri:build:mac` 和 `npm run tauri:build:mac:widget`；扩展的 App Group 与签名要求见 [macOS Widget 说明](macos-widget/README.md)。

## 来源与使用边界

- 上游项目：[Chenzo1001/Microsoft-Todo-Desktop-Widget](https://github.com/Chenzo1001/Microsoft-Todo-Desktop-Widget)
- 改良作者：**涂风**
- 本仓库目前保持 **PRIVATE**。上游项目的公开再分发许可尚未明确；私有归档不应被理解为取得了公开发布源码或安装包的授权。
- Microsoft To Do 和 Microsoft Graph 是 Microsoft 的产品与服务；本项目不是 Microsoft 官方应用。
