use crate::config::Settings;

pub fn system_locale() -> &'static str {
    #[cfg(windows)]
    {
        let language = unsafe { windows_sys::Win32::Globalization::GetUserDefaultUILanguage() };
        if language & 0x03ff == 0x04 {
            return "zh-CN";
        }
    }
    "en"
}

pub fn selected_locale(settings: &Settings) -> &str {
    match settings.general.language.as_str() {
        "system" => system_locale(),
        other => other,
    }
}

pub fn tray_label(locale: &str, key: &str) -> &'static str {
    let chinese = locale == "zh-CN";
    match (key, chinese) {
        ("open", true) => "打开 Sunshine Library Sync",
        ("sync", true) => "立即同步",
        ("pause", true) => "暂停自动同步",
        ("exit", true) => "退出",
        ("open", false) => "Open Sunshine Library Sync",
        ("sync", false) => "Sync Now",
        ("pause", false) => "Pause Auto Sync",
        ("exit", false) => "Exit",
        _ => "",
    }
}

pub fn tray_status(locale: &str, status: &str) -> String {
    let label = match (status, locale == "zh-CN") {
        ("running", true) => "运行中",
        ("stopped", true) => "已停止",
        ("pending", true) => "启动或停止中",
        ("not-installed", true) => "未安装",
        ("unknown", true) => "未知",
        ("detecting", true) => "检测中…",
        ("detecting", false) => "detecting…",
        (other, _) => other,
    };
    format!("Sunshine: {label}")
}
