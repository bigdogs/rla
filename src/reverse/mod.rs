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

use crate::runtime::rt;

mod pack;
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

pub fn unpack_apk(apk: &str) -> Result<()> {
    debug!("unpack apk: {apk}");

    let apk = Path::new(apk).to_path_buf();
    if !apk.extension().map(|e| e.eq("apk")).eq(&Some(true)) {
        return Err(format_err!("not .apk file"));
    };

    let outdir = apk.with_extension("");
    fs::create_dir(&outdir)?;
    rt().block_on(unpack::run(outdir, apk))?;
    Ok(())
}
