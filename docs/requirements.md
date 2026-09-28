# Sunshine Library Sync — implementation requirements

Source: user-selected ChatGPT conversation `6ab9d790-4af8-83e8-86ce-15875a477b0c`.
The later UI requirements supersede conflicting frontend sections below. First launch scans only; automatic sync defaults to off.

实现一个 Windows 专用的轻量 Sunshine 游戏库自动同步应用。

项目暂定名：
Sunshine Library Sync

技术栈固定：

Backend:
- Rust
- Tauri

Frontend:
- Svelte
- Vite
- TypeScript
- shadcn-svelte

目标平台：
- Windows 10 / Windows 11
- x64

不要使用：
- Electron
- React
- Vue
- Node 后端
- Python runtime
- .NET runtime

最终目标：
构建为正常的 Windows 桌面应用，
Rust/Tauri 负责系统访问、游戏扫描、文件监控、Sunshine 配置修改、代理和网络请求；
Svelte 负责 UI。

这个程序不是游戏启动器，也不是 Playnite 替代品。

不要实现：
- 游戏时长
- 成就
- 社区
- 用户账号系统
- 商店浏览
- 游戏购买
- 好友系统
- 大量 metadata 抓取

核心职责只有：

游戏平台 Provider
→ 自动发现本机游戏平台
→ 扫描本地已安装游戏
→ 合并成统一 Game 模型
→ 与 Sunshine apps.json 同步
→ Artwork
→ 自动监听安装/卸载变化

==================================================
1. 整体架构
==================================================

游戏平台必须设计成可扩展 Provider 架构。

Steam / Epic 不能把业务逻辑直接写死到同步模块。

定义统一 Provider trait。

例如：

trait GameProvider {
    fn id(&self) -> &'static str;

    fn display_name(&self) -> &'static str;

    fn detect(&self) -> Result<ProviderDetection>;

    fn scan_games(&self) -> Result<Vec<Game>>;

    fn watch_paths(&self) -> Vec<PathBuf>;

    fn launch_command(&self, game: &Game) -> Result<LaunchTarget>;

    fn artwork_candidates(
        &self,
        game: &Game
    ) -> Result<Vec<ArtworkCandidate>>;
}

具体签名可以根据 Rust/Tauri 的实际实现优化，
但 Provider 之间必须通过统一接口与上层通信。

第一版实现：

providers/
    steam/
    epic/

未来应能直接添加：

providers/
    ea/
    ubisoft/
    gog/
    xbox/

而无需修改核心 sync engine。

==================================================
2. Provider Detection
==================================================

每个 Provider 自己负责：

- 判断对应 Launcher 是否安装
- 找到 Launcher 安装路径
- 找到游戏 metadata / manifest 路径
- 返回当前状态

定义类似：

ProviderDetection {
    installed: bool,
    launcher_path: Option<PathBuf>,
    data_paths: Vec<PathBuf>,
    version: Option<String>
}

UI 中展示：

Steam
Installed
42 Games

Epic Games
Installed
18 Games

EA App
Not Installed

未来添加 Provider 后，
UI 不应需要硬编码不同逻辑。

==================================================
3. 统一 Game 模型
==================================================

所有 Provider 扫描结果转换成统一结构。

例如：

Game {
    provider_id: String,
    provider_game_id: String,

    name: String,

    install_path: PathBuf,

    launch_target: LaunchTarget,

    artwork: Option<Artwork>,

    metadata: HashMap<String, Value>
}

provider_game_id 必须是平台稳定 ID。

例如：

Steam:
appid

Epic:
AppName

不要使用游戏名称作为唯一 ID。

全局唯一 ID：

GameKey {
    provider_id,
    provider_game_id
}

例如：

steam:1245620
epic:Fortnite

==================================================
4. Steam Provider
==================================================

不得要求：

- Steam 登录
- OAuth
- Steam Web API Key
- Steam 用户账号信息

完全读取本机已有数据。

自动检测 Steam。

优先读取 Windows Registry：

SteamPath

然后读取：

<Steam>/steamapps/libraryfolders.vdf

发现所有 Steam Library。

扫描：

<library>/steamapps/appmanifest_*.acf

解析至少：

appid
name
installdir
StateFlags
LastUpdated
SizeOnDisk

检查：

<library>/steamapps/common/<installdir>

存在后认为游戏已安装。

生成：

provider_id:
steam

provider_game_id:
appid

launch target：

steam://rungameid/<appid>

扫描过程必须完全离线。

==================================================
5. Epic Provider
==================================================

不得要求：

- Epic 登录
- OAuth
- Epic API
- EOS
- Cookie

扫描：

%ProgramData%\Epic\EpicGamesLauncher\Data\Manifests\*.item

.item 是 JSON。

读取：

AppName
DisplayName
InstallLocation
LaunchExecutable
CatalogNamespace
CatalogItemId
AppCategories

过滤：

- Unreal Engine
- Engine components
- DLC
- Addons
- 非游戏 Launcher Component
- InstallLocation 不存在

但过滤规则应该放在 Epic Provider 内部，
不能放入 Sync Engine。

provider_id：

epic

provider_game_id：

AppName

优先使用 URI 启动：

com.epicgames.launcher://apps/
{CatalogNamespace}%3A{CatalogItemId}%3A{AppName}
?action=launch&silent=true

必须正确 URL encode。

==================================================
6. Provider 扩展设计
==================================================

Provider Registry 统一管理 Provider。

例如：

ProviderRegistry {
    providers: Vec<Box<dyn GameProvider>>
}

应用启动后：

ProviderRegistry
→ Detect All
→ Enable Installed Providers

用户可以在设置里禁用 Provider。

例如：

Steam        ON
Epic         ON
EA App       OFF
Ubisoft      OFF

即使 Provider 已安装，
用户也可以选择不扫描。

配置不能写死 Steam / Epic 字段。

例如使用：

[providers.steam]
enabled = true

[providers.epic]
enabled = true

未来：

[providers.ea]
enabled = true

无需修改配置结构。

==================================================
7. Sunshine 检测
==================================================

自动寻找官方 LizardByte Sunshine。

优先检测：

C:\Program Files\Sunshine

默认：

C:\Program Files\Sunshine\config\sunshine.conf

读取：

file_apps

如果不存在配置：

默认：

C:\Program Files\Sunshine\config\apps.json

允许用户在设置中手动指定。

UI 显示：

Sunshine
Detected

Path:
C:\Program Files\Sunshine

Apps:
C:\Program Files\Sunshine\config\apps.json

Service:
Running

==================================================
8. Sunshine Entry Ownership
==================================================

绝对不能删除用户手动添加的 Sunshine Application。

程序必须维护独立数据库：

%LOCALAPPDATA%\SunshineLibrarySync\state.json

或：

state.db

第一版优先 JSON。

记录：

provider_id
provider_game_id
sunshine_entry_id
name
launch_target
artwork_path
last_seen
hash

只有本工具自己创建过的 Application 才允许：

- 修改
- 删除

Sunshine 中已有：

Desktop
Steam Big Picture
Command Prompt
用户自定义程序

必须完全保留。

==================================================
9. Sync Engine
==================================================

Sync Engine 不能知道 Steam / Epic 的具体格式。

Sync Engine 只能接受：

Vec<Game>

流程：

ProviderRegistry.scan_all()

↓

Vec<Game>

↓

Current State
vs
Managed State

↓

Diff

新增：
Added

更新：
Changed

卸载：
Removed

无变化：
Unchanged

同步规则：

Added
→ 写入 Sunshine

Changed
→ 更新本工具管理字段

Removed
→ 删除对应 managed Sunshine entry

Unchanged
→ 不操作

重复 sync 必须完全幂等。

==================================================
10. apps.json 写入
==================================================

禁止直接覆盖写入。

流程：

Read
→ Parse
→ Merge
→ Serialize
→ Temporary File
→ Flush
→ Atomic Replace

写入前：

apps.json
→ apps.json.bak

只在内容真正变化时写文件。

如果 JSON 无法解析：

禁止继续写。

显示错误：

Sunshine apps.json is invalid

并保留原文件。

==================================================
11. Sunshine Application Schema
==================================================

不要猜 Sunshine apps.json schema。

开发阶段：

检查当前 LizardByte/Sunshine：

- 官方文档
- Web UI Application 实现
- apps.json 示例
- 当前源码

按照当前 schema 生成。

Steam：

使用：

steam://rungameid/<appid>

Epic：

使用官方推荐 Epic URI。

不要直接启动游戏 exe，
除非 Provider 明确判断 URI 不可用。

==================================================
12. Artwork Architecture
==================================================

Artwork 与 Game Scan 完全分离。

游戏同步绝不能等待网络 Artwork。

Game Scan：

必须完全离线。

Artwork Pipeline：

Local
→ Launcher Cache
→ Provider CDN
→ Optional Remote Provider

定义：

trait ArtworkProvider

注意：

GameProvider
和
ArtworkProvider

是两个概念。

Provider 可以提供 Artwork Candidate，
但 Artwork 下载由统一 Artwork Service 负责。

==================================================
13. Steam Artwork
==================================================

优先：

Steam 本地缓存

其次：

根据 appid 访问 Steam CDN。

不允许为了获取：

name
appid
install state

调用 Steam Web API。

Steam API 只能属于可选 metadata 功能，
第一版不要依赖。

==================================================
14. Epic Artwork
==================================================

优先查：

Epic Launcher 本地缓存。

没有则允许尝试网络来源。

失败：

不影响同步。

显示默认 placeholder。

==================================================
15. SteamGridDB
==================================================

支持可选 SteamGridDB。

SteamGridDB API Key：

可选。

无 API Key：

程序完整功能仍然可用。

用户可以在 UI 设置：

Artwork Source

Auto
Steam
Epic
SteamGridDB
Local Only

==================================================
16. Network / Proxy
==================================================

所有联网功能必须统一通过 NetworkService。

任何 Provider 都不能自行创建 HTTP Client。

统一处理：

Proxy
Timeout
Retry
User-Agent
Connection Error

支持：

System Proxy
Direct
HTTP
HTTPS
SOCKS5

UI：

Proxy

Mode:
System
Direct
Custom

Custom Proxy:

http://127.0.0.1:7897

支持：

http://
https://
socks5://

同时读取环境变量：

HTTP_PROXY
HTTPS_PROXY
ALL_PROXY
NO_PROXY

优先级：

UI / config
>
environment
>
system proxy
>
direct

提供：

Test Proxy

例如请求一个可靠的 HTTPS endpoint，
显示：

Connected
Latency

或者：

Proxy connection failed

重要：

代理只能影响网络模块。

Steam / Epic Provider 本地扫描必须完全不依赖网络。

==================================================
17. Watch Service
==================================================

实现自动监控。

每个 Provider 通过：

watch_paths()

返回需要监听的文件/目录。

Steam：

libraryfolders.vdf

各 Library：

appmanifest_*.acf

Epic：

%ProgramData%\Epic\EpicGamesLauncher\Data\Manifests

统一 WatchService：

Provider
→ watch paths
→ filesystem watcher
→ event
→ debounce
→ scan provider
→ sync

Debounce：

3 秒。

连续事件只触发一次扫描。

不要为每个 manifest 变化立刻重启 Sunshine。

==================================================
18. Sunshine Reload
==================================================

配置：

Reload mode

None
Restart Sunshine
Custom Command

apps.json 内容真实变化后才执行。

一次 sync：

无论新增多少游戏，

最多 restart 一次。

不要一个游戏 restart 一次。

自动检测 Sunshine Service Name。

不要写死 service name。

==================================================
19. Tauri Backend
==================================================

Rust Backend 使用清晰模块：

src-tauri/src/

providers/
    mod.rs

    steam/
        mod.rs
        detector.rs
        scanner.rs
        manifest.rs
        artwork.rs

    epic/
        mod.rs
        detector.rs
        scanner.rs
        manifest.rs
        artwork.rs

provider_registry.rs

models/
    game.rs
    provider.rs

sunshine/
    detector.rs
    config.rs
    apps.rs
    service.rs

sync/
    engine.rs
    diff.rs
    state.rs

artwork/
    service.rs
    cache.rs

network/
    client.rs
    proxy.rs

watcher/
    mod.rs

config/
    mod.rs

commands/
    providers.rs
    games.rs
    sync.rs
    settings.rs

app_state.rs

main.rs

main.rs 不得包含业务逻辑。

==================================================
20. Tauri Commands
==================================================

前端只能通过 Tauri Commands 调用系统功能。

例如：

get_providers()

detect_providers()

scan_provider(provider_id)

scan_all_games()

get_games()

sync_sunshine()

get_sync_preview()

get_settings()

save_settings()

get_sunshine_status()

restart_sunshine()

test_proxy()

open_path()

不要让 Svelte 直接访问 Windows 文件系统。

==================================================
21. Frontend
==================================================

Frontend：

Svelte
Vite
TypeScript
shadcn-svelte

目录：

src/

lib/
    components/
    stores/
    api/
    types/

routes/
或 views/

页面：

Dashboard
Games
Providers
Sunshine
Settings

不要做复杂路由系统。

桌面应用优先。

==================================================
22. UI Style
==================================================

整体风格：

现代 Windows Desktop Utility。

不要做成：

游戏平台商城
Playnite
Steam
Epic Launcher

避免：

大量渐变
巨大 Banner
游戏商城首页
复杂 Sidebar

采用：

shadcn-svelte
Card
Table
Badge
Dialog
Switch
Tabs
Dropdown
Toast
Skeleton

整体视觉：

扁平
紧凑
信息密度适中

==================================================
23. Dashboard
==================================================

Dashboard：

Sunshine

Status:
Running

Applications:
58

Last Sync:
2 minutes ago

Provider Overview:

Steam
Installed
42 Games

Epic
Installed
16 Games

EA App
Not Supported Yet

Recent Sync:

+ Cyberpunk 2077
- Old Game
Updated Elden Ring artwork

右上：

Sync Now

==================================================
24. Providers 页面
==================================================

每个 Provider 使用 Card：

Steam

Status:
Installed

Path:
C:\Program Files (x86)\Steam

Libraries:
3

Games:
42

Enabled:
ON

按钮：

Scan

Epic Games

Status:
Installed

Games:
16

Enabled:
ON

未来新增 Provider 后，
UI 必须通过 backend 返回的 ProviderInfo 自动生成。

不能：

if steam
if epic

写一堆前端特殊逻辑。

==================================================
25. Games 页面
==================================================

Table：

Cover
Name
Provider
Install Path
Sync Status

支持：

Search

Provider Filter

状态：

Synced
New
Excluded
Error

允许：

Exclude from Sunshine

该排除项写入统一配置：

excluded_games

key 使用：

provider_id + provider_game_id

==================================================
26. Sunshine 页面
==================================================

显示：

Sunshine Status
Version
Install Path
apps.json Path
Service Status

按钮：

Open Config Folder

Restart Sunshine

Sync Now

Preview Sync

Sync Preview：

Added 5
Removed 2
Updated 3

展开查看具体游戏。

==================================================
27. Settings 页面
==================================================

Sections：

General

Start with Windows
Start Minimized
Auto Sync

Providers

动态生成所有 Provider。

Artwork

Artwork Source
SteamGridDB API Key

Network

Proxy Mode

System
Direct
Custom

Proxy URL

Test Proxy

Sunshine

Sunshine Path

apps.json

Reload Mode

==================================================
28. Tray
==================================================

使用 Tauri Tray。

菜单：

Open Sunshine Library Sync

Sync Now

Pause Auto Sync

Sunshine:
Running

Exit

关闭窗口默认行为：

隐藏到托盘。

Settings 允许：

Close behavior

Minimize to Tray
Exit Application

如果选择 Exit：

必须彻底退出所有：

watcher
network task
background worker
Tauri process

不能出现类似 Playnite：

关闭 UI 后进程仍然残留。

==================================================
29. Background Task
==================================================

程序只有在以下情况允许后台运行：

用户明确：

Minimize to Tray

或者：

Start with Windows

如果用户点击：

Exit

必须停止所有异步任务。

使用 cancellation token / graceful shutdown。

==================================================
30. 配置
==================================================

路径：

%LOCALAPPDATA%\SunshineLibrarySync\config.toml

例如：

[general]
auto_sync = true
start_with_windows = false
close_behavior = "tray"

[providers.steam]
enabled = true

[providers.epic]
enabled = true

[sunshine]
apps_path = "..."
reload_mode = "restart"

[network]
proxy_mode = "system"
proxy_url = ""

[artwork]
provider = "auto"

结构必须允许未来任意 Provider。

==================================================
31. 错误处理
==================================================

错误必须用户可理解。

例如：

Steam manifest could not be parsed

Epic manifest directory not found

Sunshine apps.json permission denied

Sunshine service restart failed

Proxy unavailable

Artwork download failed

Artwork 下载失败：

warning

不能阻止同步。

apps.json 写失败：

error

禁止更新 state。

==================================================
32. 日志
==================================================

路径：

%LOCALAPPDATA%\SunshineLibrarySync\logs

格式：

[Provider:Steam] Found 42 games
[Provider:Epic] Found 16 games

[Sync] Added Cyberpunk 2077
[Sync] Removed Game XYZ

[Sunshine] apps.json updated
[Sunshine] service restarted

[Artwork] Download failed ...

实现：

log rotation。

UI 提供：

Open Logs

==================================================
33. 测试
==================================================

为以下模块编写测试：

Steam VDF parser

Steam library detection

Epic manifest parser

Provider Registry

GameKey equality

Sync Diff

Sunshine Merge

Managed Entry deletion

Atomic write

Proxy config

必须使用 fixture。

不能要求 CI 环境真的安装：

Steam
Epic
Sunshine

测试：

多个 Steam Library

Steam 游戏卸载

Epic DLC 排除

同名游戏来自 Steam + Epic

Sunshine 手动应用保留

重复同步幂等

Provider disabled

网络完全离线

Artwork 下载失败

Proxy 无效

==================================================
34. 第一版完成标准
==================================================

程序启动：

自动检测 Steam / Epic / Sunshine。

Providers 页面：

正确显示 Installed / Not Installed。

扫描：

无需任何账号授权。

Steam：
从本地 manifest 找到游戏。

Epic：
从本地 manifest 找到游戏。

Games 页面：

显示统一游戏库。

Sync：

正确更新 Sunshine apps.json。

卸载游戏：

watcher 检测后自动删除 managed entry。

新装游戏：

自动添加。

Sunshine 用户手动 Application：

永远不受影响。

网络断开：

扫描和同步仍正常。

Proxy：

Artwork 请求走代理。

关闭应用并选择 Exit：

进程完全退出。

==================================================
35. 开发约束
==================================================

优先实现完整可运行产品。

不要停留在：

README
架构设计
伪代码
Mock UI

必须实际实现：

Rust backend
Tauri commands
Svelte UI
Provider architecture
Steam Provider
Epic Provider
Sunshine Sync
Watcher
Proxy
Artwork
Settings
Tests

不要分 Phase 1 / Phase 2。

一次性完成第一版。

如果某些外围功能暂时无法可靠实现，
优先保证：

Provider Architecture
Steam / Epic Local Scan
Sunshine Sync
Watch
Proxy
Tauri UI

代码应方便未来接入：

EA App
Ubisoft Connect
GOG Galaxy
Xbox / Microsoft Store
Battle.net

新增 Provider 时，
不应修改 Sync Engine。

# Superseding UI requirements

==================================================
21. UI / UX 总体约束
==================================================

UI 技术栈固定：

- Svelte
- Vite
- TypeScript
- shadcn-svelte
- lucide-svelte

这是 Windows 桌面管理工具，不是网站，不是游戏商城。

整体视觉参考：

- Windows 11 Settings
- GitHub Desktop
- Raycast Settings
- Linear 的克制感
- shadcn 的基础视觉语言

但不要直接复制任何产品。

设计目标：

- 扁平
- 紧凑
- 清晰
- 信息密度偏高
- 低装饰
- 桌面工具感
- 快速扫描状态
- 重要操作始终容易找到

避免：

- Landing Page 风格
- SaaS Dashboard 风格
- 游戏商城风格
- 大面积渐变
- 玻璃拟态
- 毛玻璃背景
- 巨大的 Hero
- 大标题占据大量空间
- 每块内容都放 Card
- 夸张圆角
- 大量阴影
- 彩色渐变图标
- 大量动画
- Hover 后元素明显位移
- 大面积空白
- 一页只有几个巨大组件

UI 应该感觉像：

“专门管理 Sunshine 游戏库的 Windows Utility”

而不是：

“一个做游戏同步的网页后台”。

==================================================
22. 窗口布局
==================================================

默认窗口尺寸：

1100 × 720

最小窗口：

900 × 600

支持自由缩放。

主布局：

┌────────────────────────────────────────────┐
│ Titlebar / App Header                       │
├──────────────┬─────────────────────────────┤
│ Navigation   │ Main Content                │
│              │                             │
│              │                             │
└──────────────┴─────────────────────────────┘

左侧导航宽度：

180px ~ 210px

不要做 260px 以上的大 Sidebar。

导航：

Overview
Games
Providers
Sunshine
Settings

底部可以放：

Version
GitHub / About

不要在 Sidebar 塞统计信息。

==================================================
23. App Header
==================================================

顶部高度：

48px ~ 52px

左侧：

当前页面标题

右侧固定：

[ Sync Now ]

以及一个更多操作菜单：

⋯

菜单：

Refresh
Restart Sunshine
Open Logs

不要做：

巨大搜索框
账号头像
通知中心
商城入口

因为本程序不存在用户账号体系。

==================================================
24. 视觉尺寸系统
==================================================

整体使用紧凑 Desktop Density。

推荐：

页面 Padding：
20px ~ 24px

Section 间距：
20px ~ 24px

控件高度：

Button:
32px ~ 36px

Input:
32px ~ 36px

Table Row:
40px ~ 44px

Navigation Row:
36px ~ 40px

Card Padding:
14px ~ 16px

禁止：

48px+ 的普通按钮

禁止：

60px+ 的 Table Row

圆角：

6px ~ 8px

不要使用夸张的 16px / 24px 圆角。

阴影：

尽量不用。

优先依靠：

border
background contrast
spacing

完成层级划分。

==================================================
25. Typography
==================================================

使用系统字体：

font-family:
system-ui

Windows 优先：

Segoe UI Variable
Segoe UI

页面标题：

20px ~ 22px
SemiBold

Section Title：

14px ~ 16px
SemiBold

正文：

13px ~ 14px

辅助文字：

12px ~ 13px

不要使用：

32px+
40px+
48px+

的大标题。

这是工具软件，不是宣传页面。

==================================================
26. Color
==================================================

支持：

Light
Dark
System

默认：

System

颜色尽量由 shadcn token 控制。

状态色只用于状态：

Success
Warning
Error
Info

不要使用大量品牌色。

Provider 图标可以使用各自原始 Logo，
但 UI 其它部分保持中性。

Accent Color：

只用于：

- Primary Button
- Selected Navigation
- Focus Ring
- Active Toggle

不要到处使用 Accent。

==================================================
27. Overview 页面
==================================================

Overview 不能做成一堆巨大统计 Card。

布局：

┌─────────────────────────────────────────────┐
│ Overview                         Sync Now    │
├─────────────────────────────────────────────┤
│ Sunshine                                    │
│ ● Running    Version xxx                    │
│ Applications 58      Last Sync 2 min ago    │
├─────────────────────────────────────────────┤
│ Providers                                   │
│                                             │
│ Steam       ● Installed     42 games        │
│ Epic        ● Installed     16 games        │
│ EA          ○ Not installed                 │
├─────────────────────────────────────────────┤
│ Last Sync                                   │
│ + Cyberpunk 2077                     Steam  │
│ - Game XYZ                           Epic   │
│ ~ Elden Ring artwork                 Steam  │
└─────────────────────────────────────────────┘

Sunshine 状态使用一个普通 section。

不要做：

┌─────────────┐
│ 58          │
│ Games       │
└─────────────┘

这种大型 KPI Card。

Providers 使用紧凑 List。

每一行：

Provider Logo
Provider Name
Status
Games Count

右侧可以：

Enabled Switch

==================================================
28. Games 页面
==================================================

Games 页面是整个应用主要页面。

必须使用 Table / DataGrid。

不要默认使用游戏封面 Card Grid。

布局：

┌─────────────────────────────────────────────┐
│ Games                          58 games      │
│                                             │
│ [ Search... ] [All Providers ▼] [Status ▼] │
├─────────────────────────────────────────────┤
│   Name             Provider    Status        │
│ ─────────────────────────────────────────── │
│ ▣ Cyberpunk 2077   Steam       Synced       │
│ ▣ Alan Wake 2      Epic        Synced       │
│ ▣ Elden Ring       Steam       Excluded     │
└─────────────────────────────────────────────┘

Columns：

Artwork
Name
Provider
Install Path
Sync Status

Artwork：

32×42
或
36×48

不要显示巨大封面。

Install Path：

过长时 ellipsis。

Hover 后 Tooltip 显示完整路径。

Provider：

使用：

Logo + Name

Sync Status：

Badge：

Synced
New
Changed
Excluded
Error

不要把 Synced 做成鲜艳绿色大 Badge。

只需：

绿色 dot + text

或者轻量 Badge。

==================================================
29. Game Row 交互
==================================================

单击一行：

选中。

双击：

默认打开游戏详情 Dialog 或 Side Panel。

右键：

Context Menu：

Sync
Exclude
Open Install Directory
Copy Game ID
Open Provider

不要默认启动游戏。

因为这个程序不是 Launcher。

选中游戏后右侧可以出现 Detail Sheet：

Cyberpunk 2077

Steam

App ID:
1091500

Install:
D:\SteamLibrary\steamapps\common\Cyberpunk 2077

Launch:
steam://rungameid/1091500

Sunshine:
Synced

Artwork:
...

[Open Folder]

[Exclude]

不要为详情跳转新的全屏页面。

==================================================
30. Games 搜索与过滤
==================================================

顶部工具栏固定保持紧凑。

Search：

宽度：

240px ~ 300px

Provider：

All
Steam
Epic
...

Status：

All
Synced
New
Changed
Excluded
Error

可选：

Installed

不要添加十几个过滤器。

不要做高级 Query Builder。

游戏数量显示：

58 games

而不是：

TOTAL GAMES
58

==================================================
31. Providers 页面
==================================================

Providers 页面使用普通列表，不要做巨大 Card Grid。

例如：

Providers

Steam
────────────────────────────────────────────
● Installed

Path
C:\Program Files (x86)\Steam

Libraries
3

Games
42

Auto Sync                       [ ON ]

                               [ Scan ]

Epic Games
────────────────────────────────────────────
● Installed

Path
C:\Program Files (x86)\Epic Games

Games
16

Auto Sync                       [ ON ]

                               [ Scan ]

Provider 之间使用：

Separator

而不是每一个都塞进巨大 Card。

Provider Logo：

24×24 或 28×28

Provider Status：

小状态点。

Installed：
绿色 dot

Not Installed：
灰色 dot

Error：
红色 dot

==================================================
32. Provider 未安装状态
==================================================

未安装 Provider：

EA App

○ Not installed

No local EA App installation was detected.

不要：

显示巨大 Error
弹 Toast
显示红色警告

未安装游戏平台是正常状态。

==================================================
33. Sunshine 页面
==================================================

布局按 Setting / Inspector 形式。

Sunshine

Status
────────────────────────────

Service
● Running

Version
2026.xxxxx

Install Path
C:\Program Files\Sunshine

apps.json
C:\Program Files\Sunshine\config\apps.json

[ Open Config Folder ]

Actions
────────────────────────────

[ Sync Now ]
[ Restart Sunshine ]

Sync
────────────────────────────

Managed Applications     58

Last Sync
2026-09-28 10:42

Reload Mode
[ Restart Sunshine ▼ ]

下面可以显示：

Sync Preview

+ 5 Added
~ 2 Changed
- 1 Removed

点击：

View Details

再打开 Dialog。

不要直接把整个 diff 永久铺满页面。

==================================================
34. Sync 操作 UX
==================================================

Sync Now 点击后：

Button：

Syncing...

显示 spinner。

不要弹全屏 Loading。

扫描与 Sync 应该尽量在后台完成。

完成后显示 Toast：

Sync completed

5 added
2 updated
1 removed

如果没有变化：

Library is already up to date.

不要重启 Sunshine：

如果 apps.json 没变化。

==================================================
35. Sync Preview
==================================================

Preview Sync 打开 Dialog：

Sync Preview

Added
──────────────────
Cyberpunk 2077
Alan Wake 2

Updated
──────────────────
Elden Ring

Removed
──────────────────
Old Game

底部：

Cancel

Apply Sync

危险操作：

Removed

用轻量 warning 色。

不要整个 Dialog 红色。

==================================================
36. Settings 页面
==================================================

不要做：

每个设置一个 Card。

Settings 使用 Windows Settings / VSCode Settings 类似布局：

General
────────────────────────────

Start with Windows          [ OFF ]

Close behavior
[ Minimize to Tray ▼ ]

Auto Sync                   [ ON ]

Artwork
────────────────────────────

Artwork Source
[ Auto ▼ ]

SteamGridDB API Key
[ **************** ]

Network
────────────────────────────

Proxy
[ System ▼ ]

Proxy URL
[ http://127.0.0.1:7897 ]

[ Test Connection ]

Sunshine
────────────────────────────

apps.json
[ C:\...\apps.json            ][ Browse ]

Reload
[ Restart Sunshine ▼ ]

Section 之间：

24px spacing + Separator。

==================================================
37. Proxy UI
==================================================

Proxy Mode：

System
Direct
Custom

如果：

System

隐藏 Proxy URL。

如果：

Direct

隐藏 Proxy URL。

如果：

Custom

显示：

Proxy URL

输入 Placeholder：

http://127.0.0.1:7897

下面：

[ Test Connection ]

测试结果直接显示在按钮右侧：

● Connected  42 ms

或：

● Failed

不要每次 Test 都弹 Toast。

==================================================
38. Artwork UI
==================================================

Artwork 失败不是核心错误。

如果没有 Artwork：

使用统一 Placeholder。

不要显示 Broken Image Icon。

Placeholder：

普通灰色背景
Gamepad icon

Artwork 加载：

异步。

不能阻塞 Games Table。

不要因为 Artwork API 很慢导致：

Games 页面 Loading。

==================================================
39. Loading State
==================================================

启动应用时：

Shell 必须立即显示。

不要：

白屏数秒。

页面：

Overview

可以先显示 Skeleton。

例如：

Sunshine
██████

Providers
██████

Backend 完成扫描后替换。

Provider Scan 不能阻止 UI 创建。

==================================================
40. Empty State
==================================================

没有扫描到游戏：

Games

No games found.

Steam and Epic will be scanned automatically.

[ Scan Again ]

不要使用：

大型插画
营销文案
几十行解释。

Sunshine 未检测：

Sunshine was not detected.

[ Locate Sunshine ]

Provider 未安装：

正常显示 Not Installed。

==================================================
41. Error State
==================================================

局部错误局部显示。

例如：

Steam
⚠ Scan failed

[ Retry ]

不要因为 Steam Provider 出错：

阻止 Epic 扫描。

不要出现：

Something went wrong :(

这种没有信息量的错误。

显示具体但简洁的错误：

Unable to read Steam libraryfolders.vdf.

Details

Details 可以展开技术错误。

==================================================
42. Toast 规范
==================================================

Toast 只用于：

操作成功
操作失败
同步完成

不要用于：

Provider 未安装
Artwork 缺失
扫描开始
扫描结束的每一个步骤

例如：

✓ Sync completed

而不是连续弹：

Scanning Steam
Steam scanned
Scanning Epic
Epic scanned
Updating Sunshine
Restarting Sunshine
Done

这种 Toast spam。

==================================================
43. Dialog 使用规范
==================================================

Dialog 只用于：

- Sync Preview
- 确认删除/清除
- 高风险修改
- 游戏详情
- 设置 API Key

不要每个操作都开 Dialog。

普通设置直接在页面修改。

==================================================
44. Context Menu
==================================================

Games Table 支持 Windows 风格右键菜单：

Sync

Exclude from Sunshine

Open Install Directory

Copy Game ID

Open in Steam / Epic

菜单项保持：

32px 左右高度。

不要做巨大菜单。

==================================================
45. Animation
==================================================

动画非常克制。

允许：

Opacity
100~150ms

Dropdown
100~150ms

Dialog
150ms

禁止：

Spring animation
Bounce
大幅 Slide
Card Hover 上浮
缩放动画

用户开启系统 Reduce Motion 时：

禁用非必要动画。

==================================================
46. Sidebar
==================================================

Sidebar：

宽度约：

192px

项目：

Overview
Games
Providers
Sunshine
Settings

图标：

16px ~ 18px

文字：

13px ~ 14px

Selected：

轻背景色

不要：

左侧彩色粗条
巨型图标
复杂折叠菜单

目前只有 5 个页面，
不需要 Nested Navigation。

==================================================
47. Status Bar
==================================================

窗口底部允许一个非常轻量的 Status Bar。

高度：

24px ~ 28px

例如：

Sunshine ● Running

Steam 42
Epic 16

Last Sync 10:42

右边：

v0.1.0

如果实现 Status Bar：

不要同时在多个地方重复大量相同状态。

==================================================
48. Titlebar
==================================================

优先使用 Tauri 自定义 Titlebar，
但外观必须克制。

Windows：

最小化
最大化
关闭

必须符合 Windows 常规交互习惯。

关闭按钮：

Hover 红色。

拖动区域：

不能覆盖 Button/Input。

如果自定义标题栏导致复杂度明显上升，
第一版可以保留系统原生标题栏。

功能稳定优先。

==================================================
49. Responsive
==================================================

这是 Desktop-first。

不需要 Mobile Layout。

窗口较窄时：

Sidebar 可以缩成 Icon Sidebar。

Table：

允许横向滚动。

不要为了响应式把 Desktop UI 做成手机卡片列表。

==================================================
50. UI Component Rules
==================================================

优先使用 shadcn-svelte：

Button
Input
Select
Switch
Badge
Table
Dropdown Menu
Context Menu
Dialog
Sheet
Tabs
Tooltip
Separator
Skeleton
Toast

Card 只在真正存在独立视觉容器时使用。

原则：

能用 Section + Separator
就不要使用 Card。

能用 Table
就不要用一堆 Card。

能用 Inline Status
就不要做 KPI Widget。

==================================================
51. Provider 图标
==================================================

Provider 数据模型增加：

icon

但 Backend 不应该返回任意网络 URL。

Frontend 内置 Provider icon mapping。

例如：

steam
epic

未来：

ea
ubisoft
gog
xbox
battlenet

Provider Registry 返回：

id
display_name

Frontend 根据 ID 查找对应本地图标。

找不到：

使用 Generic Gamepad Icon。

==================================================
52. Provider UI 必须数据驱动
==================================================

禁止：

{#if provider.id === "steam"}
...
{:else if provider.id === "epic"}
...
{/if}

这样的 Provider 专用 UI。

必须：

{#each providers as provider}

渲染通用 ProviderRow / ProviderSection。

Provider 新增：

EAProvider

后端注册后，

UI 自动显示：

EA App

而不需要增加新的页面布局代码。

只有图标 mapping 可以按 provider_id 配置。

==================================================
53. UI State Architecture
==================================================

不要把所有状态塞在 App.svelte。

至少拆分：

providers store

games store

sunshine store

settings store

sync store

例如：

src/lib/stores/

providers.ts
games.ts
sunshine.ts
settings.ts
sync.ts

Tauri API：

src/lib/api/

providers.ts
games.ts
sunshine.ts
settings.ts

Svelte Component 不能散落 invoke()。

所有 Tauri 调用统一封装。

==================================================
54. 首次启动
==================================================

不要设计复杂 Setup Wizard。

第一次打开：

自动 detect：

Steam
Epic
Sunshine

直接进入 Overview。

如果 Sunshine 没找到：

顶部显示一个轻量提示：

Sunshine was not detected.

[ Locate Sunshine ]

如果 Steam/Epic 没安装：

正常显示 Not Installed。

不阻止用户进入应用。

==================================================
55. 默认交互原则
==================================================

默认行为必须安全。

第一次启动：

不要直接修改 apps.json。

先：

Scan

展示结果。

用户第一次点击：

Sync Now

之后再同步。

设置中允许：

Auto Sync

开启后才自动执行。

这样避免第一次安装程序：

突然向 Sunshine 加几十个游戏。

==================================================
56. 不要生成这些 UI
==================================================

明确禁止生成：

- 巨型 Dashboard 数字卡
- Dashboard 折线图
- Pie Chart
- 游戏时长图表
- 最近玩过
- Trending Games
- News
- Welcome Hero
- Login Page
- 用户头像
- Profile
- Notification Center
- 云同步
- 商店页面
- 社区页面

这些全部不属于项目目标。

==================================================
57. 最终 UI 验收标准
==================================================

最终 UI 打开时应该让用户第一眼看到：

Sunshine 是否正常。

Steam / Epic 是否检测到。

扫描到了多少游戏。

有没有未同步变化。

并且最多两次点击就能：

同步游戏库。

用户进入 Games 后：

可以快速搜索、
筛选 Provider、
检查 Sync 状态、
排除某个游戏。

用户进入 Settings 后：

可以直接配置：

Proxy
Artwork
Auto Sync
Sunshine Path

而不需要进入多层设置页面。

整个 UI：

不能有明显网页 Landing Page 感。

不能看起来像游戏平台。

不能看起来像典型 AI 生成 SaaS Dashboard。

优先像一个成熟、克制、原生感较强的 Windows Utility。

## 2026-09-28 补充要求

- Release EXE 与安装包不能使用测试清单或模拟游戏数据。测试数据只在 Debug 集成测试中启用，测试窗口应有明显标识。
- 界面支持英文和简体中文；首次启动及语言设置为“跟随系统”时采用 Windows 显示语言。设置中可手动切换，语言选择保存在本机。
