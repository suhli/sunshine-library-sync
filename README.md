# Sunshine Library Sync

将本机 Steam 和 Epic Games 的已安装游戏同步到 Sunshine，方便在 Moonlight 中浏览和启动。适用于 Windows 10/11 x64。

应用会自动发现游戏平台和游戏库，汇总已安装游戏。你可以在同步前查看将新增、更新或移除的游戏，也可以排除不想显示的项目。

## 下载

在 [Releases](https://github.com/suhli/sunshine-library-sync/releases) 选择适合的版本：

- **安装版**：下载 `_nsis.exe`，按向导安装。
- **免安装版**：下载 `_portable.zip`，解压后运行 `sunshine-library-sync.exe`。

## 使用

1. 启动应用，等待 Steam 和 Epic 游戏扫描完成。首次启动只扫描，不会修改 Sunshine。
2. 如果应用没有找到 Sunshine，在 **设置 → Sunshine** 中选择 Sunshine 的 `apps.json`。
3. 在 **游戏** 页面查看扫描结果，并排除不想同步的游戏。
4. 在 **Sunshine** 页面预览变更并应用同步，或点击右上角的 **立即同步**。
5. 如需让游戏安装、卸载后的变化自动同步，可在 **设置** 中开启自动同步；默认关闭。

同步会保留你在 Sunshine 中手动添加的应用。若新游戏没有立即出现在 Moonlight 中，请重启 Sunshine；也可以在设置中选择同步后自动重启。自动重启可能中断正在进行的串流。
