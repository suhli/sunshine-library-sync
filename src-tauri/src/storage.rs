use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use std::{fs, io::Write, path::Path};

pub fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

/// Same-directory staging + durable flush + atomic replacement. Windows ReplaceFile
/// retains the destination's ACL, unlike remove-then-rename implementations.
pub fn atomic_write(path: &Path, data: &[u8]) -> Result<()> {
    let parent = path.parent().context("File has no parent directory")?;
    fs::create_dir_all(parent)?;
    let mut tmp = tempfile::NamedTempFile::new_in(parent)
        .context("Unable to create temporary file; check folder permissions")?;
    tmp.write_all(data)?;
    tmp.as_file().sync_all()?;
    // ReplaceFileW needs to open the replacement with DELETE access. Close our
    // tempfile handle first; TempPath still removes it on every error path.
    let tmp = tmp.into_temp_path();
    #[cfg(windows)]
    if path.exists() {
        use std::os::windows::ffi::OsStrExt;
        use windows_sys::Win32::Storage::FileSystem::ReplaceFileW;
        let dst: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
        let src: Vec<u16> = tmp.as_os_str().encode_wide().chain(Some(0)).collect();
        // Both NUL-terminated paths remain alive for the entire Win32 call.
        let ok = unsafe {
            ReplaceFileW(
                dst.as_ptr(),
                src.as_ptr(),
                std::ptr::null(),
                0,
                std::ptr::null(),
                std::ptr::null(),
            )
        };
        if ok == 0 {
            return Err(std::io::Error::last_os_error())
                .context("Atomic replacement failed; check permissions or close the other editor");
        }
        return Ok(());
    }
    tmp.persist(path)
        .map_err(|e| e.error)
        .context("Unable to atomically replace file")?;
    Ok(())
}

pub fn write_json(path: &Path, value: &impl serde::Serialize) -> Result<()> {
    let mut bytes = serde_json::to_vec_pretty(value)?;
    bytes.push(b'\n');
    atomic_write(path, &bytes)
}
