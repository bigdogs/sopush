use std::process::Command;

use anyhow::{format_err, Context, Result};
use once_cell::sync::Lazy;
use regex::Regex;
use tracing::debug;

static SPLITTER: Lazy<Regex> = Lazy::new(|| Regex::new(r#"[^\s"']+|"([^"]*)"|'([^']*)'"#).unwrap());

pub(crate) fn run(command: &str) -> Result<String> {
    debug!("[+] {}", command);
    let mut iter = SPLITTER.find_iter(command).map(|m| m.as_str());
    let cmd = iter.next().context("no command")?;
    let args = iter.collect::<Vec<_>>();
    let output = Command::new(cmd).args(&args).output()?;

    return if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        let s = format!(
            "{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        Err(format_err!(
            "{:?} execute error, exit code: {:?}, msg: {:?}",
            command,
            output.status.code(),
            s
        ))
    };
}
