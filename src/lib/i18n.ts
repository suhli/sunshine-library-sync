import { derived, get, writable } from 'svelte/store';
import { settings } from '$lib/stores/settings';

export type AppLocale = 'en' | 'zh-CN';
export const systemLocale = writable(typeof navigator === 'undefined' ? 'en' : navigator.language);
export const fixtureMode = writable(false);

export function resolveLocale(value: string): AppLocale {
  return /^zh(?:-|$)/i.test(value) ? 'zh-CN' : 'en';
}

export const locale = derived([settings, systemLocale], ([$settings, $systemLocale]) =>
  $settings.general.language === 'system' ? resolveLocale($systemLocale) : resolveLocale($settings.general.language)
);

// English source strings also serve as the fallback when a new UI message has not been translated.
const zh: Record<string, string> = {
  'Overview': '概览', 'Games': '游戏', 'Providers': '游戏平台', 'Sunshine': 'Sunshine', 'Settings': '设置',
  'On': '开启', 'Off': '关闭', 'Local games. Simply synced.': '本地游戏，轻松同步。', 'Version 0.1.0': '版本 0.1.0',
  'Main navigation': '主导航', 'More actions': '更多操作', 'Sync Now': '立即同步', 'Syncing…': '正在同步…',
  'Refresh': '刷新', 'Preview Sync': '预览同步', 'Restart Sunshine': '重启 Sunshine', 'Open Logs': '打开日志',
  'Desktop preview': '桌面预览', 'Open the Windows app to detect local games and connect Sunshine.': '请打开 Windows 应用，以检测本地游戏并连接 Sunshine。',
  'Test fixture': '测试数据', 'This Debug session uses isolated test manifests and a separate Sunshine configuration.': '当前 Debug 会话使用隔离的测试清单和 Sunshine 配置。',
  'Dismiss error': '关闭错误提示', 'Dismiss notification': '关闭通知',
  'Sunshine running': 'Sunshine 运行中', 'Sunshine detected': '已检测到 Sunshine', 'Sunshine not detected': '未检测到 Sunshine',
  'Scanning libraries…': '正在扫描游戏库…', 'Ready': '就绪', 'Auto Sync paused': '自动同步已暂停', 'Auto Sync on': '自动同步已开启', 'Auto Sync off': '自动同步已关闭',
  'Your installed games, connected to Sunshine.': '将已安装游戏连接到 Sunshine。', 'View details': '查看详情', 'Local installation detected': '已检测到本机安装',
  'Version {version}': '版本 {version}', 'Applications': '应用', '{count} total': '共 {count} 个', '· {count} managed': '· 本程序管理 {count} 个',
  'Last sync': '上次同步', 'Configuration': '配置', 'Sunshine was not detected.': '未检测到 Sunshine。',
  'Locate its apps.json file to connect your library.': '请选择 Sunshine 的 apps.json 文件以连接游戏库。', 'Locate Sunshine': '定位 Sunshine',
  'Manage': '管理', '{count} games': '{count} 个游戏', 'Enable {name}': '启用 {name}',
  'No providers available.': '没有可用的游戏平台。', 'Local providers are available in the desktop app.': '请在桌面应用中检测本地游戏平台。',
  'Steam and Epic are discovered from their local installation files.': 'Steam 和 Epic 将根据本机安装文件自动识别。',
  'Library changes': '游戏库变更', 'Preview sync': '预览同步', '+ {count} added': '+ 新增 {count} 个', '~ {count} changed': '~ 更改 {count} 个', '− {count} removed': '− 移除 {count} 个',
  'Ready to sync. Review the changes or choose Sync Now.': '可以同步。请检查变更，或点击“立即同步”。',
  'No sync yet. Your first scan leaves Sunshine unchanged.': '尚未同步。首次扫描不会更改 Sunshine。',
  'Some entries need attention. Open the sync preview for details.': '部分条目需要处理。请打开同步预览查看详情。',
  'Your library is up to date.': '游戏库已是最新状态。', 'Changes will appear here after your first sync.': '首次同步后，变更将显示在这里。',
  'Scan again': '重新扫描',
  'Search games': '搜索游戏', 'Search games…': '搜索游戏…', 'Filter by provider': '按游戏平台筛选', 'All providers': '所有游戏平台',
  'Filter by sync status': '按同步状态筛选', 'All statuses': '所有状态', 'Installed games': '已安装游戏', 'Artwork': '封面', 'Name': '名称',
  'Provider': '游戏平台', 'Install path': '安装路径', 'Sync status': '同步状态', 'Scanning local libraries…': '正在扫描本地游戏库…',
  'No games match these filters.': '没有符合筛选条件的游戏。', 'No games found.': '未找到游戏。',
  'Try another name, provider or status.': '请尝试其他名称、游戏平台或状态。', 'Steam and Epic will be scanned automatically in the desktop app.': '桌面应用会自动扫描 Steam 和 Epic。',
  'Clear filters': '清除筛选', 'Sync': '同步', 'Include in Sunshine': '包含在 Sunshine 中', 'Exclude from Sunshine': '从 Sunshine 中排除',
  'Open install directory': '打开安装目录', 'Copy game ID': '复制游戏 ID', 'Open provider folder': '打开游戏平台文件夹', 'Details': '详情',
  'Include': '包含', 'Exclude': '排除', 'Double-click a game for details. Right-click for actions.': '双击游戏查看详情，右键点击查看更多操作。',
  'Local library': '本地游戏库', 'Game details': '游戏详情', 'Select a game from your library.': '请从游戏库中选择一款游戏。',
  'Game ID': '游戏 ID', 'Install': '安装位置', 'Launch': '启动目标', 'Cached locally': '已在本机缓存', 'No cover available': '暂无封面',
  'Open folder': '打开文件夹', 'Copy ID': '复制 ID', 'Game ID copied.': '游戏 ID 已复制。',
  'Discover installed games from local launcher files. No account access is needed.': '通过本地启动器文件查找已安装游戏，无需登录账号。',
  'Scanning paused · existing entries kept': '扫描已暂停 · 保留现有条目', 'Launcher path': '启动器路径', 'Not available': '不可用',
  'Library locations': '游戏库位置', 'Watched locations': '监听位置',
  'No local {name} installation was detected.': '未检测到本机的 {name}。', '{count} scan warnings · automatic removals paused': '{count} 条扫描警告 · 已暂停自动移除',
  'Retry scan': '重试扫描', 'Scan': '扫描', 'Configure path': '配置路径', 'Detecting local providers…': '正在检测本地游戏平台…',
  'Open the desktop app to detect providers.': '请打开桌面应用以检测游戏平台。', 'Installed Steam and Epic libraries will appear here automatically.': '已安装的 Steam 和 Epic 游戏库会自动显示在这里。',
  'Connection, configuration and sync status for your Sunshine installation.': '查看 Sunshine 的连接、配置与同步状态。',
  'Status': '状态', 'Edit configuration': '编辑配置', 'Service': '服务', 'Version': '版本', 'Not detected': '未检测到', 'Open config folder': '打开配置文件夹',
  'Actions': '操作', 'Restarting Sunshine may interrupt an active stream.': '重启 Sunshine 可能会中断正在进行的串流。',
  'Managed applications': '本程序管理的应用', 'Reload mode': '重新加载方式', 'Automatic sync': '自动同步', 'Enabled': '已启用',
  'None · restart Sunshine manually after changes': '无 · 修改后手动重启 Sunshine', 'Restart Sunshine service': '重启 Sunshine 服务', 'Custom command': '自定义命令',
  'Sync preview': '同步预览', 'Locate a valid apps.json to preview changes.': '请选择有效的 apps.json 以预览变更。',
  'Review changes to applications managed by Library Sync.': '检查 Library Sync 管理的应用变更。', 'Added': '新增', 'Updated': '更新', 'Removed': '移除',
  'No changes': '无变更', '{count} unchanged · Manual Sunshine applications are preserved.': '{count} 个未变化 · 手动添加的 Sunshine 应用会保留。',
  'Refresh preview': '刷新预览', 'Cancel': '取消', 'Apply Sync': '应用同步',
  'Backup failed': '备份失败', 'Sunshine apps.json was not changed. Choose whether to continue this sync without a backup.': 'Sunshine apps.json 尚未更改。请选择是否在没有备份的情况下继续这次同步。',
  'Backup destination': '备份位置', 'Continuing will update apps.json without a backup from this sync. Cancel keeps it unchanged.': '继续会在没有本次备份的情况下更新 apps.json；取消则保持文件不变。',
  'Cancel sync': '取消同步', 'Continue without backup': '无备份继续同步',
  'Close': '关闭',
  'Running': '运行中', 'Stopped': '已停止', 'Not installed': '未安装', 'Installed': '已安装', 'Synced': '已同步', 'New': '新增', 'Changed': '已修改',
  'Excluded': '已排除', 'Error': '错误', 'Disabled': '已禁用', 'Unknown': '未知', 'Starting / stopping': '启动或停止中',
  'Preferences for this computer.': '此电脑的偏好设置。', 'Open data folder': '打开数据文件夹', 'General': '常规',
  'Appearance': '外观', 'Follow Windows or choose a theme.': '跟随 Windows 或手动选择主题。', 'System': '跟随系统', 'Light': '浅色', 'Dark': '深色',
  'Language': '语言', 'Follow the Windows display language or choose one.': '跟随 Windows 显示语言，或手动选择。', 'English': '英语', 'Simplified Chinese': '简体中文',
  'Start with Windows': '开机启动', 'Start for your Windows account.': '登录 Windows 账号时启动。',
  'Start minimized': '启动时最小化', 'Hide to the tray when started with Windows.': '开机启动时隐藏到托盘。',
  'Close behavior': '关闭窗口时', 'Exit stops all background activity.': '退出会停止全部后台活动。', 'Minimize to tray': '最小化到托盘', 'Exit application': '退出应用',
  'Auto Sync': '自动同步', 'Sync when installed games change. Off until you enable it.': '已安装游戏变化时同步，默认关闭。',
  'Optional location overrides. Leave blank for automatic detection.': '可选的路径设置；留空则自动检测。', 'Launcher folder or manifest location': '启动器文件夹或清单位置',
  'Automatic': '自动检测', 'Provider settings appear after desktop detection.': '检测到游戏平台后会显示相关设置。',
  'Artwork source': '封面来源', 'Local covers first. Downloads run in the background.': '优先使用本地封面，下载在后台进行。',
  'Auto': '自动', 'Local only': '仅本地', 'SteamGridDB API key': 'SteamGridDB API 密钥',
  'Optional · saved in your local configuration file.': '可选 · 保存在本机配置文件中。', 'Not configured': '未配置',
  'Network': '网络', 'Proxy': '代理', 'Local scans and sync work without a connection.': '本地扫描和同步无需网络连接。',
  'Direct': '直连', 'Custom': '自定义', 'Proxy URL': '代理地址', 'HTTP, HTTPS or SOCKS5': '支持 HTTP、HTTPS 或 SOCKS5',
  'Test connection': '测试连接', 'Connected · {ms} ms': '连接成功 · {ms} 毫秒',
  'Install folder': '安装文件夹', 'Optional path override.': '可选的路径设置。', 'Browse': '浏览',
  'Locate Sunshine apps.json': '选择 Sunshine apps.json', 'Locate Sunshine folder': '选择 Sunshine 文件夹', 'JSON configuration': 'JSON 配置',
  'Reads file_apps from sunshine.conf by default.': '默认从 sunshine.conf 读取 file_apps。',
  'Reload after changes': '变更后重新加载', 'Runs once only when apps.json changes.': '仅在 apps.json 发生变化时执行一次。',
  'None': '无', 'Restart Sunshine manually after syncing to refresh the Moonlight list.': '同步后请手动重启 Sunshine，以刷新 Moonlight 应用列表。',
  'Restarting may interrupt a stream and may require administrator rights.': '重启可能中断串流，并且可能需要管理员权限。',
  'Reload command': '重新加载命令', 'Windows command to run after a successful sync.': '同步成功后运行的 Windows 命令。',
  'You have unsaved changes.': '有未保存的更改。', 'Settings are saved locally.': '设置已保存在本机。',
  'Discard': '放弃更改', 'Save changes': '保存更改', 'Settings saved.': '设置已保存。',
  'Library saved. {error}': '游戏库已保存，但重新加载失败：{error}',
  'Sync completed · {added} added, {updated} updated, {removed} removed': '同步完成 · 新增 {added} 个，更新 {updated} 个，移除 {removed} 个',
  'Library is already up to date.': '游戏库已是最新状态。',
  'Excluded. Apply Sync to remove its managed entry.': '已排除。应用同步后将移除对应的托管条目。', 'Included in the next sync.': '将在下次同步中包含。',
  'Sunshine restarted.': 'Sunshine 已重启。', 'Never': '从未',
};

export function translate(language: AppLocale, key: string, parameters: Record<string, string | number> = {}): string {
  const template = language === 'zh-CN' ? zh[key] ?? key : key;
  return template.replace(/\{(\w+)\}/g, (match, name: string) => String(parameters[name] ?? match));
}

export const t = derived(locale, language => (key: string, parameters?: Record<string, string | number>) => translate(language, key, parameters));
export function formatTimestamp(value: number | null | undefined, language: AppLocale): string {
  return value ? new Date(value * 1000).toLocaleString(language === 'zh-CN' ? 'zh-CN' : 'en-US') : translate(language, 'Never');
}
export const tr = (key: string, parameters?: Record<string, string | number>) => translate(get(locale), key, parameters);
