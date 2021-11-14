use std::path::Path;

use anyhow::Result;
use anyhow::{format_err, Context};
use tracing::warn;

use crate::command;

/// find lib path of android package
pub(crate) fn app_lib_location(pkg: &str) -> Result<String> {
    fn finder(package: &str, path: &str, recursive: bool) -> Result<String> {
        let ls_result = command::run(&format!("adb shell su -c \"ls {}\"", path))?;
        for dir in ls_result.split("\n").filter_map(|s| {
            let trim = s.trim();
            (!trim.is_empty()).then(|| trim)
        }) {
            let dir_path = format!("{}/{}", path, dir);
            if dir.contains(package) {
                return Ok(dir_path);
            }
            if recursive {
                if let Ok(sub_dir) = finder(package, &dir_path, false) {
                    return Ok(sub_dir);
                }
            }
        }
        Err(format_err!("not found"))
    }
    finder(pkg, "/data/app", true)
}

pub(crate) fn push(file: &Path, dest_paths: &[&str]) -> Result<()> {
    let file_name = file.to_str().context(format!("file to str error"))?;
    let base_name = file
        .file_name()
        .and_then(|s| s.to_str())
        .context(format!("file base name error"))?;
    command::run(&format!("adb push {} /sdcard/{}", file_name, base_name))?;

    for p in dest_paths {
        command::run(&format!(
            "adb shell su -c \"cp /sdcard/{} {}\"",
            base_name, p
        ))
        .map_err(|e| {
            warn!("{}", e);
        })
        .ok();
    }
    if let Err(e) = command::run(&format!("adb shell rm /sdcard/{}", base_name)) {
        warn!("adb rm error: {:?}", e);
    }
    Ok(())
}
