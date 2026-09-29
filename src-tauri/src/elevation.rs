#[cfg(windows)]
pub fn restart_as_admin() -> anyhow::Result<()> {
    use anyhow::{bail, Context};
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::UI::{Shell::ShellExecuteW, WindowsAndMessaging::SW_SHOWNORMAL};

    let executable =
        std::env::current_exe().context("Unable to find the application executable")?;
    let operation: Vec<u16> = "runas\0".encode_utf16().collect();
    let file: Vec<u16> = executable
        .as_os_str()
        .encode_wide()
        .chain(Some(0))
        .collect();
    let parameters: Vec<u16> = format!("--wait-for-pid {}\0", std::process::id())
        .encode_utf16()
        .collect();
    let result = unsafe {
        ShellExecuteW(
            std::ptr::null_mut(),
            operation.as_ptr(),
            file.as_ptr(),
            parameters.as_ptr(),
            std::ptr::null(),
            SW_SHOWNORMAL,
        )
    };
    if result as isize <= 32 {
        bail!(
            "Windows did not approve the administrator launch (code {}).",
            result as isize
        );
    }
    Ok(())
}

#[cfg(not(windows))]
pub fn restart_as_admin() -> anyhow::Result<()> {
    anyhow::bail!("Administrator restart is only available on Windows")
}

#[cfg(windows)]
pub fn wait_for_previous_instance() {
    use windows_sys::Win32::{
        Foundation::CloseHandle,
        Storage::FileSystem::SYNCHRONIZE,
        System::Threading::{OpenProcess, WaitForSingleObject},
    };

    let mut arguments = std::env::args();
    while let Some(argument) = arguments.next() {
        if argument != "--wait-for-pid" {
            continue;
        }
        let Some(pid) = arguments.next().and_then(|value| value.parse::<u32>().ok()) else {
            return;
        };
        let handle = unsafe { OpenProcess(SYNCHRONIZE, 0, pid) };
        if !handle.is_null() {
            unsafe {
                WaitForSingleObject(handle, u32::MAX);
                CloseHandle(handle);
            }
        }
        return;
    }
}

#[cfg(not(windows))]
pub fn wait_for_previous_instance() {}
