use std::{ffi::OsStr, process::Command};

use crate::{
    deps::{Dep, APK_SIGNER, DEBUG_STORE},
    dir::binarydir,
};
use anyhow::{Context, Result};
use tracing::debug;

/// run a jar file without capture output
pub fn run_jar<T: AsRef<OsStr>>(jar: &'static Dep, args: &[T]) -> Result<()> {
    let jarfile = jar.release_binary(binarydir())?;

    let mut cmd = Command::new("java");
    cmd.arg("-jar").arg(jarfile).args(args);
    debug!("{cmd:?}");
    cmd.status()?.exit_ok()?;
    Ok(())
}

pub fn debugsign(file: &str) -> Result<()> {
    let debug_store = DEBUG_STORE.release_binary(binarydir())?;
    let debug_store = debug_store
        .to_str()
        .with_context(|| format!("path not utf8: {debug_store:?}"))?;
    run_jar(
        APK_SIGNER,
        &[
            "sign",
            "--ks",
            debug_store,
            "--ks-pass",
            "pass:android",
            file,
        ],
    )
}