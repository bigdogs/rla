use std::{ffi::OsStr, path::Path, process::Command};

use crate::{
    deps::{Dep, APK_SIGNER, DEBUG_STORE},
    dir::binarydir,
};
use anyhow::Result;

/// run a jar file without capture output
pub(crate) fn run_jar<T: AsRef<OsStr>>(jar: &'static Dep, args: &[T]) -> Result<()> {
    let jarfile = jar.release_binary(binarydir())?;
    let mut cmd = Command::new("java");
    cmd.arg("-jar").arg(jarfile).args(args);
    crate::cmd::run_interit(cmd)
}

pub(crate) fn debugsign(file: &Path) -> Result<()> {
    let debug_store = DEBUG_STORE.release_binary(binarydir())?;
    let apk_signer = APK_SIGNER.release_binary(binarydir())?;

    let mut c = Command::new("java");
    c.arg("-jar")
        .arg(apk_signer)
        .arg("sign")
        .arg("--ks")
        .arg(debug_store)
        .arg("--ks-pass")
        .arg("pass:android")
        .arg(file);
    super::run(c).map(|_| ())
}

pub(crate) fn git_init(workdir: &Path) -> Result<String> {
    let mut git = Command::new("git");
    git.current_dir(workdir).arg("init");
    super::run(git)
}

pub(crate) fn git_commit(workdir: &Path, msg: &str) -> Result<String> {
    let mut git = Command::new("git");
    git.current_dir(workdir).args(&["commit", "-m", msg]);
    super::run(git)
}

pub(crate) fn git_add(workdir: &Path) -> Result<String> {
    let mut git = Command::new("git");
    git.current_dir(workdir).args(&["add", "."]);
    super::run(git)
}

pub(crate) fn jadx_extract_src(apk: &Path, outdir: &Path) -> Result<String> {
    let mut jadx = Command::new("jadx");
    jadx.arg("-e").arg(apk).arg("-d").arg(outdir);
    super::run(jadx)
}

pub(crate) fn baksmali(dex: &Path, outdir: &Path, baksmali_jar: &Path) -> Result<String> {
    let mut c = Command::new("java");
    c.arg("-jar")
        .arg(baksmali_jar)
        .arg("d")
        .arg(dex)
        .arg("-o")
        .arg(outdir);
    super::run(c)
}

pub(crate) fn smali(smali_dir: &Path, dex: &Path, smali_jar: &Path) -> Result<String> {
    let mut c = Command::new("java");
    c.arg("-jar")
        .arg(smali_jar)
        .arg("a")
        .arg(smali_dir)
        .arg("-o")
        .arg(dex);
    super::run(c)
}
