use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use anyhow::{format_err, Context, Result};
use once_cell::sync::Lazy;
use structopt::{clap::ArgGroup, StructOpt};
use tracing::debug;
use walkdir::WalkDir;

/// sopush --pkg xx --lib xxx.so
/// sopush --pkg xx --aar --aar-name
#[derive(StructOpt, Debug)]
#[structopt(group = ArgGroup::with_name("so_file").required(true))]
struct SoPushOpt {
    /// android package to push
    #[structopt(long = "pkg", env = "SO_PUSH_PKG")]
    package_name: String,
    /// target app arch
    #[structopt(long, env = "SO_PUSH_ARCH")]
    arch: String,
    /// lib (e.g xx.so) to push
    #[structopt(long = "lib", group = "so_file")]
    so_path: Option<String>,
    #[structopt(long = "aar", group = "so_file")]
    is_aar: bool,
    /// local aar name
    #[structopt(long = "aar-name", env = "SO_PUSH_AAR_NAME")]
    aar_name: Option<String>,
}

pub(crate) enum Arch {
    Arm,
    Arm64,
}

impl Arch {
    fn jni_name(&self) -> &str {
        match self {
            Arch::Arm => "armeabi-v7a",
            Arch::Arm64 => "arm64-v8a",
        }
    }
}

struct Inner {
    package_name: String,
    lib_path: PathBuf,
    arch: Arch,
}

static INNER: Lazy<Inner> = Lazy::new(|| {
    let opt: SoPushOpt = SoPushOpt::from_args();
    debug!("{:?}", opt);

    let arch = match opt.arch.as_str() {
        "arm" => Arch::Arm,
        "arm64" => Arch::Arm64,
        _ => {
            panic!("--arch can be either \"arm\" or \"arm64\"");
        }
    };
    let lib = if opt.is_aar {
        let aar_name = opt.aar_name.expect("--aar-name is missing?");
        let aar_path = find_file_at_cur_dir(&aar_name)
            .expect(&format!("{} is not found at current dir", aar_name));
        extract_so(&aar_path, arch.jni_name()).unwrap()
    } else {
        PathBuf::from(opt.so_path.expect("so file is not specified"))
    };
    Inner {
        package_name: opt.package_name.clone(),
        lib_path: lib,
        arch,
    }
});

fn find_file_at_cur_dir(aar_name: &str) -> Option<PathBuf> {
    let aar_name = OsStr::new(aar_name);
    WalkDir::new(".")
        .into_iter()
        .filter_map(|e| e.ok())
        .find(|entry| entry.file_type().is_file() && entry.file_name() == aar_name)
        .map(|f| f.path().to_path_buf())
}

fn extract_so(aar_path: &Path, arch: &str) -> Result<PathBuf> {
    let file = std::fs::File::open(&aar_path)?;
    let mut archive = zip::ZipArchive::new(file)?;

    for i in 0..archive.len() {
        let mut zf = archive.by_index(i)?;
        if zf.is_file() && zf.name().ends_with(".so") && zf.name().contains(arch) {
            // need to keep file name
            let so_name = Path::new(zf.name())
                .file_name()
                .with_context(|| format!("invalid file name? {:?}", zf.name()))?;
            // need to keep handle if use tempfile
            // not work on windows
            let temp_dir = Path::new("/tmp");
            let temp_so_name = temp_dir.join(so_name);
            let mut temp_so = std::fs::File::create(&temp_so_name)?;
            std::io::copy(&mut zf, &mut temp_so)?;
            return Ok(temp_so_name);
        }
    }
    Err(format_err!(
        "{}/xxx.so is not found in file {:?}",
        arch,
        aar_path
    ))
}

pub(crate) fn target_package() -> &'static str {
    &INNER.package_name
}

pub(crate) fn local_lib() -> &'static Path {
    &INNER.lib_path
}

pub(crate) fn jni_name() -> &'static str {
    &INNER.arch.jni_name()
}
