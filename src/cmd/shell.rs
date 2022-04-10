use std::{ffi::OsStr, fs, path::Path, process::Command};

use crate::{
    deps::{Dep, APK_SIGNER, DEBUG_STORE},
    dir::binarydir,
};
use anyhow::{Context, Result};

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
    super::run(c).map(|_| {
        // Latest apksigner use v4 algorithm, which will create a extra file(xxx.idsig),
        // and I have no idea what is used for, just delete it :(
        // https://source.android.com/security/apksigning/v4
        let mut s = file.to_string_lossy().to_string();
        s.push_str(".idsig");
        if Path::new(&s).exists() {
            fs::remove_file(s).ok();
        }
    })
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

pub(crate) fn jadx_compile_smali(smali: &Path, outdir: &Path) -> Result<String> {
    let mut jadx = Command::new("jadx");
    jadx.arg("-d").arg(outdir).arg(smali);
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

pub(crate) fn unzip(apk: &Path, dir: &Path) -> Result<String> {
    let mut c = Command::new("unzip");
    c.arg("-n").arg("-d").arg(dir).arg(apk);
    super::run(c)
}

/// compile java requires working dir to java root
pub(crate) fn compile_java<P: AsRef<OsStr>>(java_files: &[P], work_dir: &Path) -> Result<String> {
    let mut c = Command::new("javac");
    c.current_dir(work_dir)
        .arg("--release")
        .arg("8")
        .args(java_files);
    super::run(c)
}

/// dx compile requires working to java root
pub(crate) fn dx_class_to_dex<P: AsRef<OsStr>>(
    class_files: &[P],
    work_dir: &Path,
    dx_jar: &Path,
    out_dex: &Path,
) -> Result<String> {
    let class_files = class_files
        .iter()
        .map(|p| {
            Path::new(p.as_ref())
                .strip_prefix(work_dir)
                .with_context(|| format!("{:?} not at dir {:?}", p.as_ref(), work_dir))
        })
        .collect::<Result<Vec<_>>>()?;

    let mut c = Command::new("java");
    c.current_dir(work_dir)
        .arg("-jar")
        .arg(dx_jar)
        .arg("--dex")
        .arg("--output")
        .arg(out_dex)
        .args(class_files);
    super::run(c)
}

pub(crate) fn zip_update_files<P: AsRef<OsStr>>(
    apk: &Path,
    workdir: &Path,
    files: &[P],
) -> Result<String> {
    let mut c = Command::new("zip");
    c.current_dir(workdir).arg(apk).args(files);
    super::run(c)
}
