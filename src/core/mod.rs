//! base.apk
//! output/
//! smalis/
//! unzipped/
//! jadx-src (if jadx is available)
//! .git
//! .gitignore
//! .rla.config.json

use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{format_err, Context, Result};
use tracing::debug;

pub use java_to_smali::java_to_smali;
pub use smali_to_java::smali_to_java;

use crate::runtime::rt;

mod java_to_smali;
mod pack;
mod smali_to_java;
mod unpack;

const RLA_CONFIG: &str = ".rla.config.json";

fn find_rla_root() -> Option<PathBuf> {
    let cur = std::env::current_dir().ok()?;
    let mut cur = Some(&*cur);
    while let Some(dir) = cur {
        if dir.join(RLA_CONFIG).exists() {
            return Some(dir.to_path_buf());
        } else {
            cur = dir.parent()
        }
    }
    None
}

pub fn pack_apk(dir: Option<String>) -> Result<()> {
    let root = dir
        .map(PathBuf::from)
        .or_else(find_rla_root)
        .context("can't find project root")?;
    debug!("pack apk at {root:?}");

    rt().block_on(pack::run(root))?;
    Ok(())
}

pub fn unpack_apk(apk: &str, no_jadx: bool, no_git: bool, force: bool) -> Result<()> {
    debug!("unpack apk: {apk}, no_jadx: {no_jadx}, no_git: {no_git}, force: {force}");

    let apk = Path::new(apk).to_path_buf();
    if !apk.extension().map(|e| e.eq("apk")).eq(&Some(true)) {
        return Err(format_err!("not .apk file"));
    };

    // prepare write directory
    let outdir = apk.with_extension("");
    if outdir.exists() {
        if force {
            debug!("remove {outdir:?}");
            fs::remove_dir_all(&outdir).with_context(|| format!("remove {outdir:?} error"))?
        } else {
            return Err(format_err!(
                "{outdir:?} already exists, delete it or use --force"
            ));
        }
    }
    fs::create_dir(&outdir).with_context(|| format!("{outdir:?} create error"))?;

    rt().block_on(unpack::run(outdir, apk, no_jadx, no_git))?;
    Ok(())
}
