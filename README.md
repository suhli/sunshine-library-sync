# Sunshine Library Sync

Windows 10/11 x64 上的轻量 Sunshine 游戏库同步工具。Rust / Tauri 2 后端，Svelte 5 / Vite / TypeScript / shadcn-svelte 界面。

读取本机 Steam、Epic manifest，无需登录、OAuth 或商店 API Key。扫描和 Sunshine 同步可完全离线。首次启动只扫描；点击 **Sync Now** 才写入，**Auto Sync 默认关闭**。

## 使用

1. 打开 `sunshine-library-sync.exe`。首次扫描显示已安装游戏平台及游戏。
2. 如果未找到 Sunshine，在 **Settings → Sunshine → apps.json → Browse** 选择现有配置并保存。
3. 在 **Games** 搜索、筛选或排除游戏。双击查看详情，右键打开文件夹、复制 ID 或同步单个游戏。
4. 在 **Sunshine → View details** 预览新增、修改、删除，然后 **Apply Sync**。也可直接点右上角 **Sync Now**。
5. 在 **Settings** 开启 Auto Sync 后，manifest 变化会等待 3 秒再合并同步。安装 / 卸载或新库目录变更会更新监听路径。

界面提供英文和简体中文。默认跟随 Windows 显示语言，可在 **Settings → General → Language** 手动选择并保存；语言切换会立即应用到界面和托盘菜单。

Reload 默认 **None**，同步后需要手动重启 Sunshine 才能刷新 Moonlight 列表。可改成 **Restart Sunshine** 或自定义命令；只有配置实际变化时才执行，一次批量最多执行一次。重启 Windows 服务可能需要管理员权限，也可能中断正在串流的会话。程序不会主动请求提权。

关闭按钮默认隐藏到托盘；可改成 **Exit application**。托盘的 **Exit** 会取消下载与监听，等待正在提交的同步完成，再退出整个进程。关闭为 Exit 时不保留后台进程。开机启动按用户注册，不安装 Windows 服务。

## 构建与验证

开发依赖：Rust 1.90+ MSVC 工具链、Visual Studio C++ Build Tools / Windows SDK、Node.js 22+、pnpm 11。运行应用无需 Node/Rust/Python/.NET；Tauri 使用 Windows WebView2 Runtime。

```powershell
pnpm install
pnpm tauri dev
```

```powershell
pnpm check
pnpm build
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
pnpm tauri build
```

`pnpm tauri build --no-bundle` 仅生成 Release EXE。完整构建还会生成当前用户安装的 NSIS 安装包。

- EXE：`src-tauri/target/release/sunshine-library-sync.exe`
- 安装包：`src-tauri/target/release/bundle/nsis/`
- `pnpm dev` 只预览前端，明确显示 Desktop preview，不伪造本机数据。

## GitHub Actions 发布

现有 `Windows checks` 工作流在 push / pull request 时运行前端检查、Rust 测试与 Windows 构建。发布时在 GitHub 的 **Actions → Publish Windows release → Run workflow** 中选择 `main`，输入稳定版本号，例如 `0.2.0`（也接受 `v0.2.0`）。版本 tag 已存在时会停止，不覆盖旧发布。

发布工作流会将输入版本写入 `package.json`、Tauri 配置、`Cargo.toml` 和 `Cargo.lock`，完成检查和 Windows x64 构建后创建 `v<version>` tag 与 GitHub Release。版本变更保存在 tag 指向的提交中，不修改 `main` 分支。Release 附带两个文件：

- `Sunshine-Library-Sync_v<version>_win10-win11_x64_portable.zip`：免安装 EXE 和 README。
- `Sunshine-Library-Sync_v<version>_win10-win11_x64_nsis.exe`：当前用户安装的 NSIS 安装包。

Portable 表示无需安装程序；它仍需要 Windows WebView2 Runtime，设置和缓存仍保存在 `%LOCALAPPDATA%\SunshineLibrarySync`。

桌面集成测试先运行 `cargo build --manifest-path src-tauri/Cargo.toml`，再运行 `./scripts/desktop-fixture.ps1 -Launch`。脚本只启动 Debug EXE，在 `.test-data/desktop/` 创建真实格式的测试 manifests 和独立 apps.json；测试窗口会标明“测试数据”。`SUNSHINE_LIBRARY_SYNC_DATA` 仅在 Debug 构建中生效，Release EXE 和安装包始终使用 `%LOCALAPPDATA%\SunshineLibrarySync`，不读取测试数据目录。测试结束后从托盘退出 Debug 应用。两种构建使用同一个单实例标识，启动前请先退出已运行的另一版本。

## 同步的数据保护

- Steam/Epic 只通过 `GameProvider` 提供统一 `Game`；同步引擎不知道具体市场的 manifest 格式。
- 游戏身份使用 `provider_id + provider_game_id`，不使用名称。重名游戏会分配不冲突的 Sunshine 名称。
- 当前 Sunshine 应用配置没有可靠的持久 UUID 字段。本程序只在独立状态文件中记录自己实际写入的应用及字段指纹，不往 Sunshine schema 添加自造字段。
- 只有状态记录与 apps.json 中的管理字段唯一匹配时才更新或删除。手工创建的应用、根级 env / 未知字段、管理条目中的非管理字段都会保留。
- 管理字段被手工编辑、应用被复制、状态文件无效或发现相同启动目标时，停止处理相应条目并在预览中显示冲突。不会按名称认领旧应用。丢失状态文件时不会删除已有条目，但也不会自动认领它们。
- 禁用 Provider、manifest 解析失败、安装尚未完成或某个库无法读取时，不用不完整结果删除该 Provider 的旧条目。其他可用 Provider 仍能扫描和同步。
- 写入前保存 `apps.json.bak`，同目录临时文件 flush 后用 Windows `ReplaceFileW` 原子替换，保留目标 ACL。禁止删旧文件再重命名的非原子退路。
- 两阶段恢复记录避免 apps.json 写入成功而状态落盘失败后重复添加。写入前再次核对源文件。若 Sunshine Web UI 或其他程序同时修改配置，检测到冲突会停止；同步期间请避免同时编辑该文件。
- 无变化时不写 apps.json、不动其备份、不重启 Sunshine。权限不足或 JSON 损坏会报错，原文件保留。

管理字段为 `name`、`cmd`、`detached`、`working-dir`、`image-path`。Steam 使用 `steam://rungameid/<appid>`，Epic 使用正确编码的 Launcher URI，写入 `detached` 数组，不直接启动游戏 EXE。

## 本地文件

默认目录为 `%LOCALAPPDATA%\SunshineLibrarySync`。仅 Debug 构建可用 `SUNSHINE_LIBRARY_SYNC_DATA` 指定隔离的测试目录：

```text
config.toml                    设置（包括可选 API Key）
artwork/<game-key-sha256>.png    独立封面缓存
logs/library-sync.<date>.log    日志，最多保留 7 份
targets/<apps-path-sha256>/
  state.json                   该 Sunshine 配置的管理条目与上次同步
  sync.lock                    进程间同步锁
  pending-sync.json             仅未完成事务存在时保留
```

不同 apps.json 使用独立状态，切换路径不会把另一份配置的应用当作本程序创建。备份位于目标 apps.json 旁边。若事务恢复报告外部修改，先备份并核对原文件、`.bak` 和 `pending-sync.json`；不要删除状态来强行认领应用。

Steam 路径优先读注册表，兼容新旧 `libraryfolders.vdf` 及多个游戏库。Steam 手动路径是 launcher 根目录；Epic 手动路径是 `*.item` 所在的 Manifests 目录。

## 封面与代理

扫描及同步不等待封面。Steam 先读 librarycache，再尝试 Steam CDN；Epic 尝试 manifest 中结构化的本地图片路径，缺失时显示统一占位图。可选 SteamGridDB 只按完整名称匹配，并缓存静态封面。它的 API Key 不是必填，缺失时其余功能可用。不会解析 Epic Chromium 的非公开 HTTP 缓存数据库，因此部分 Epic 游戏需要 SteamGridDB 才能获得封面。

下载统一经过 `NetworkService`，支持超时、有限重试、响应大小限制和图片解码限制。Proxy 有 **System / Direct / Custom**，Custom 接受 HTTP、HTTPS、SOCKS5。System 使用 reqwest 的环境 / 系统代理解析，支持 HTTP_PROXY、HTTPS_PROXY、ALL_PROXY、NO_PROXY；Direct 完全关闭代理。代理凭据和 API Key 不写入日志。**Test connection** 使用 `https://www.gstatic.com/generate_204`，结果及耗时在设置旁显示。

SteamGridDB Key 保存在本机 config.toml 中，请勿将该文件加入版本控制或共享。当前没有远程账户、遥测、云同步、购买、游玩统计或自动更新。

## 扩展 Provider

在 `src-tauri/src/providers/` 新增模块并实现 `GameProvider`：`id`、`display_name`、`detect`、`scan_games`、`watch_paths`、`launch_command`、`artwork_candidates`。在 `providers::registry` 注册即可。设置以 Provider ID 为 key，界面按后端数组自动生成。图标映射之外，不需要修改同步引擎、设置结构或页面布局。返回 `complete = false` 表示本次扫描不允许用于判定卸载。

`ArtworkProvider` 是单独的网络封面接口，不与 GameProvider 混合。所有网络流量经过 NetworkService。

## 上游依据

实现核对了 [Sunshine Windows apps.json 示例](https://github.com/LizardByte/Sunshine/blob/master/src_assets/windows/assets/apps.json)、[应用启动示例](https://github.com/LizardByte/Sunshine/blob/master/docs/app_examples.md)、[Web UI saveApp 实现](https://github.com/LizardByte/Sunshine/blob/master/src/confighttp.cpp) 及 [Windows appdata 路径实现](https://github.com/LizardByte/Sunshine/blob/master/src/platform/windows/misc.cpp)。需求原文保存于 [docs/requirements.md](docs/requirements.md)。

完整验收还需要在已配对 Moonlight 客户端中实际启动 Steam/Epic 游戏；仓库内测试验证配置与 URI 生成，不代替真实串流测试。
