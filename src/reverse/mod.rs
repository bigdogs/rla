//! base.apk
//! output/
//! smalis/
//! unzipped/
//! jadx-src (if jadx is available)
//! .git
//! .gitignore
//! .rla.config.json

use std::{fs, path::Path};

use anyhow::{format_err, Context, Result};
use tracing::debug;

use crate::runtime::rt;

mod unpack;

const RLA_CONFIG: &str = ".rla.config.json";

pub fn pack_apk() -> Result<()> {
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

    let start = std::time::Instant::now();
    rt().block_on(unpack::run(outdir, apk))?;
    tracing::info!("task cost {:.2?}", start.elapsed());
    Ok(())
}
